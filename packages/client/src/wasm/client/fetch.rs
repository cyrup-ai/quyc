//! WASM fetch implementation for HTTP requests

use std::convert::TryInto;
use std::fmt;

use bytes::Bytes;
use http::header::{HeaderMap, HeaderName, HeaderValue};
use http::{Method, Request, Response, StatusCode, Uri, Version};
use serde::{Deserialize, Serialize};
use serde_json;
use url::Url;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{Headers, RequestInit, RequestMode, Window};

#[cfg(target_arch = "wasm32")]
use js_sys::{Array, Object, Uint8Array};

use crate::error::{Error, Result};
use crate::prelude::HttpResponse;
use crate::wasm::body::Body;
use crate::wasm::request::Request as WasmRequest;
// ResponseExt not available - removed
use crate::wasm::{handle_error, AbortController, AbortSignal};
use ystream::AsyncStream;

#[cfg(target_arch = "wasm32")]
use web_sys::{Request as WebRequest, RequestCredentials, RequestRedirect};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
unsafe extern "C" {
    #[wasm_bindgen(js_name = fetch)]
    fn fetch_with_request(input: &web_sys::Request) -> Promise;
}

#[cfg(target_arch = "wasm32")]
fn js_fetch(req: &web_sys::Request) -> Promise {
    use wasm_bindgen::{JsCast, JsValue};
    let global = js_sys::global();

    if let Ok(true) = js_sys::Reflect::has(&global, &JsValue::from_str("ServiceWorkerGlobalScope"))
    {
        global
            .unchecked_into::<web_sys::ServiceWorkerGlobalScope>()
            .fetch_with_request(req)
    } else {
        // browser
        fetch_with_request(req)
    }
}

// Using current web-sys API for maximum compatibility, ignore their deprecation.
#[allow(deprecated)]
pub(super) fn fetch(req: Request<crate::wasm::body::Body>) -> AsyncStream<Response<crate::wasm::body::Body>, 1024> {
    use ystream::emit;
    
    AsyncStream::with_channel(move |sender| {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::prelude::*;
            use wasm_bindgen::JsCast;
            
            // Build the request setup 
            let setup_result = (|| -> Result<(web_sys::Request, web_sys::AbortController), Error> {
                let mut init = web_sys::RequestInit::new();
                init.method(req.method().as_str());

                // convert HeaderMap to Headers
                let js_headers = web_sys::Headers::new()
                    .map_err(|e| Error::from(std::io::Error::new(std::io::ErrorKind::Other, format!("WASM error: {:?}", e))))?;

                for (name, value) in req.headers() {
                    js_headers
                        .append(
                            name.as_str(),
                            value.to_str().map_err(crate::error::builder)?,
                        )
                        .map_err(|e| Error::from(std::io::Error::new(std::io::ErrorKind::Other, format!("WASM error: {:?}", e))))?;
                }
                init.headers(&js_headers.into());

                // When req.cors is true, do nothing because the default mode is 'cors'
                if !req.cors {
                    init.mode(web_sys::RequestMode::NoCors);
                }

                if let Some(creds) = req.credentials {
                    init.credentials(creds);
                }

                if let Some(cache) = req.cache {
                    init.set_cache(cache);
                }

                if let Some(body) = req.body() {
                    if !body.is_empty() {
                        init.body(Some(body.to_js_value()?.as_ref()));
                    }
                }

                let abort_controller = web_sys::AbortController::new()
                    .map_err(|e| Error::from(std::io::Error::new(std::io::ErrorKind::Other, format!("WASM error: {:?}", e))))?;
                
                // Implement WASM timeout handling with JavaScript setTimeout
                let (timeout_id, timeout_callback_handle) = if let Some(timeout) = req.timeout() {
                    let timeout_ms = timeout.as_millis() as u32;
                    let abort_controller_clone = abort_controller.clone();
                    
                    // Create timeout callback and store handle to prevent GC
                    let timeout_callback = Closure::once_into_js(move || {
                        // Abort the request when timeout is reached
                        abort_controller_clone.abort();
                    });
                    
                    // Store callback handle to prevent garbage collection
                    let callback_handle = timeout_callback.as_ref().clone();
                    
                    // Set timeout using JavaScript setTimeout API
                    let window = match web_sys::window() {
                        Some(w) => w,
                        None => return Err(Error::from(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "No window object available for timeout setup"
                        )))
                    };
                    
                    let timeout_id = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                        timeout_callback.as_ref().unchecked_ref(),
                        timeout_ms as i32
                    ).map_err(|e| Error::from(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to set timeout: {:?}", e)
                    )))?;
                    
                    (Some(timeout_id), Some(callback_handle))
                } else {
                    (None, None)
                };
                init.signal(Some(&abort_controller.signal()));

                let js_req = web_sys::Request::new_with_str_and_init(req.url().as_str(), &init)
                    .map_err(|e| Error::from(std::io::Error::new(std::io::ErrorKind::Other, format!("WASM error: {:?}", e))))?;

                Ok((js_req, abort_controller))
            })();

            match setup_result {
                Ok((js_req, abort_controller)) => {
                    // Use fetch with promise callbacks instead of Future
                    let fetch_promise = match web_sys::window() {
                        Some(window) => window.fetch_with_request(&js_req),
                        None => {
                            emit!(sender, Err(Error::from(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "No window object available for fetch"
                            ))));
                            return;
                        }
                    };
                    
                    // Helper function to clear timeout and callback
                    let clear_timeout = |timeout_id: Option<i32>, callback_handle: Option<JsValue>| {
                        if let Some(timeout_id) = timeout_id {
                            if let Some(window) = web_sys::window() {
                                window.clear_timeout_with_handle(timeout_id);
                            }
                        }
                        // Callback handle will be dropped, allowing GC
                        drop(callback_handle);
                    };
                    
                    let timeout_id_clone = timeout_id;
                    let callback_handle_clone = timeout_callback_handle.clone();
                    let success_callback = Closure::once_into_js(move |js_resp: JsValue| {
                        // Clear timeout on successful response
                        clear_timeout(timeout_id_clone, callback_handle_clone);
                        let web_response: web_sys::Response = match js_resp.dyn_into() {
                            Ok(resp) => resp,
                            Err(_) => {
                                emit!(sender, Err(Error::from(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    "Failed to convert JS response to web_sys::Response"
                                ))));
                                return;
                            }
                        };
                        
                        // Convert from the js Response
                        let mut resp = http::Response::builder().status(web_response.status());
                        let url = match Url::parse(&web_response.url()) {
                            Ok(url) => url,
                            Err(e) => {
                                emit!(sender, Err(Error::from(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    format!("Failed to parse response URL: {}", e)
                                ))));
                                return;
                            }
                        };
                        let js_headers = web_response.headers();

                        let js_iter = match js_sys::try_iter(&js_headers) {
                            Ok(Some(iter)) => iter,
                            Ok(None) => {
                                emit!(sender, Err(Error::from(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    "Headers object is not iterable"
                                ))));
                                return;
                            }
                            Err(e) => {
                                emit!(sender, Err(Error::from(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    format!("Failed to iterate headers: {:?}", e)
                                ))));
                                return;
                            }
                        };

                        for item in js_iter {
                            let item = match item {
                                Ok(item) => item,
                                Err(e) => {
                                    emit!(sender, Err(Error::from(std::io::Error::new(
                                        std::io::ErrorKind::Other,
                                        format!("Header iterator error: {:?}", e)
                                    ))));
                                    return;
                                }
                            };
                            
                            let serialized_headers: String = match js_sys::JSON::stringify(&item) {
                                Ok(json) => json.into(),
                                Err(e) => {
                                    emit!(sender, Err(Error::from(std::io::Error::new(
                                        std::io::ErrorKind::Other,
                                        format!("Failed to serialize header: {:?}", e)
                                    ))));
                                    return;
                                }
                            };
                            
                            let [name, value]: [String; 2] = match serde_json::from_str(&serialized_headers) {
                                Ok(header_pair) => header_pair,
                                Err(e) => {
                                    emit!(sender, Err(Error::from(std::io::Error::new(
                                        std::io::ErrorKind::Other,
                                        format!("Failed to deserialize header: {}", e)
                                    ))));
                                    return;
                                }
                            };
                            resp = resp.header(&name, &value);
                        }

                        // Complete the response building and create our Response type
                        match resp.body(web_response) {
                            Ok(http_response) => {
                                let response = Response::new(http_response, url, abort_controller);
                                emit!(sender, Ok(response));
                            }
                            Err(e) => {
                                emit!(sender, Err(Error::from(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    format!("Response build error: {}", e)
                                ))));
                            }
                        }
                    });
                    
                    let timeout_id_clone_error = timeout_id;
                    let callback_handle_clone_error = timeout_callback_handle;
                    let error_callback = Closure::once_into_js(move |error: JsValue| {
                        // Clear timeout on error response
                        clear_timeout(timeout_id_clone_error, callback_handle_clone_error);
                        emit!(sender, Err(Error::from(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Fetch error: {:?}", error)
                        ))));
                    });
                    
                    // Use native JavaScript Promise.then() method
                    let _ = fetch_promise.then2(&success_callback, &error_callback);
                }
                Err(setup_error) => {
                    emit!(sender, Err(setup_error));
                }
            }
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            emit!(sender, Err(Error::from(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "WASM functionality not available on this platform"
            ))));
        }
    })
}
