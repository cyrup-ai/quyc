//! WASM-specific HTTP client implementation
//!
//! This module provides WebAssembly-compatible HTTP functionality using the Fetch API.
//! It's designed to work in browser environments and web workers.

use std::convert::TryInto;
use std::time::Duration;

#[cfg(target_arch = "wasm32")]
use js_sys::Function;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::{Closure, wasm_bindgen};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};
#[cfg(target_arch = "wasm32")]
// Removed JsFuture and spawn_local - using AsyncStream only
#[cfg(target_arch = "wasm32")]
use web_sys::{AbortController, AbortSignal};

// WASM utility functions
#[cfg(target_arch = "wasm32")]
fn js_fetch(request: web_sys::Request) -> js_sys::Promise {
    match web_sys::window() {
        Some(window) => window.fetch_with_request(&request),
        None => {
            // Create a rejected promise for environments without window
            let promise = js_sys::Promise::reject(&JsValue::from_str("No window object available"));
            promise
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn set_timeout(callback: &js_sys::Function, delay: i32) -> i32 {
    match web_sys::window() {
        Some(window) => {
            window.set_timeout_with_callback_and_timeout_and_arguments_0(callback, delay)
                .unwrap_or_else(|_| -1) // Return -1 on timeout creation failure
        }
        None => -1 // Return -1 when no window available
    }
}

#[cfg(target_arch = "wasm32")]
fn clear_timeout(id: i32) {
    if let Some(window) = web_sys::window() {
        window.clear_timeout_with_handle(id);
        // Ignore errors - timeout may have already fired
    }
}

// Re-export WASM types from web-sys
#[cfg(target_arch = "wasm32")]
pub use web_sys::{AbortController, AbortSignal};

// For non-WASM targets, provide full implementation
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Mutex;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

/// AbortController for non-WASM environments
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub struct AbortController {
    inner: Arc<AbortControllerInner>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
struct AbortControllerInner {
    aborted: AtomicBool,
    signal: AbortSignal,
    callbacks: Mutex<Vec<Box<dyn Fn() + Send + Sync>>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl AbortController {
    pub fn new() -> Self {
        let aborted = Arc::new(AtomicBool::new(false));
        let callbacks = Arc::new(Mutex::new(Vec::new()));
        
        let signal = AbortSignal {
            aborted: aborted.clone(),
            callbacks: callbacks.clone(),
        };
        
        let inner = Arc::new(AbortControllerInner {
            aborted: AtomicBool::new(false),
            signal: signal.clone(),
            callbacks: Mutex::new(Vec::new()),
        });
        
        Self { inner }
    }
    
    pub fn abort(&self) {
        self.inner.aborted.store(true, Ordering::SeqCst);
        self.inner.signal.trigger_abort();
        
        // Call all registered callbacks
        if let Ok(callbacks) = self.inner.callbacks.lock() {
            for callback in callbacks.iter() {
                callback();
            }
        }
    }
    
    pub fn signal(&self) -> AbortSignal {
        self.inner.signal.clone()
    }
    
    pub fn is_aborted(&self) -> bool {
        self.inner.aborted.load(Ordering::Acquire)
    }
}

/// AbortSignal for non-WASM environments
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub struct AbortSignal {
    aborted: Arc<AtomicBool>,
    callbacks: Arc<Mutex<Vec<Box<dyn Fn() + Send + Sync>>>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl AbortSignal {
    pub fn aborted(&self) -> bool {
        self.aborted.load(Ordering::Acquire)
    }
    
    pub fn on_abort<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        if let Ok(mut callbacks) = self.callbacks.lock() {
            callbacks.push(Box::new(callback));
        }
        
        // If already aborted, call immediately
        if self.aborted() {
            callback();
        }
    }
    
    fn trigger_abort(&self) {
        self.aborted.store(true, Ordering::SeqCst);
        
        if let Ok(callbacks) = self.callbacks.lock() {
            for callback in callbacks.iter() {
                callback();
            }
        }
    }
    
    pub async fn wait_for_abort(&self) {
        while !self.aborted() {
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for AbortController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_arch = "wasm32")]
use std::fmt;

#[cfg(target_arch = "wasm32")]
use bytes::Bytes;
#[cfg(target_arch = "wasm32")]
use http::{HeaderMap, HeaderName, HeaderValue, Method, Request, Response, StatusCode, Uri};
#[cfg(target_arch = "wasm32")]
use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use url::Url;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
// Removed JsFuture - using AsyncStream only
use js_sys::Promise;
use ystream::prelude::MessageChunk;

use crate::error::Error;

// MessageChunk implementation for JsValue
impl MessageChunk for JsValue {
    fn bad_chunk(error: String) -> Self {
        JsValue::from_str(&format!("ERROR: {}", error))
    }

    fn error(&self) -> Option<&str> {
        if let Some(s) = self.as_string() {
            if s.starts_with("ERROR: ") {
                return Some(&s[7..]);
            }
        }
        None
    }

    fn is_error(&self) -> bool {
        if let Some(s) = self.as_string() {
            s.starts_with("ERROR: ")
        } else {
            false
        }
    }
}

impl Default for JsValue {
    fn default() -> Self {
        JsValue::NULL
    }
}

// MessageChunk implementation for FormData
#[cfg(target_arch = "wasm32")]
impl MessageChunk for web_sys::FormData {
    fn bad_chunk(error: String) -> Self {
        match web_sys::FormData::new() {
            Ok(form_data) => {
                let _ = form_data.append_with_str("error", &error);
                form_data
            }
            Err(_) => {
                // If we can't create FormData, return a minimal FormData-like object
                // This should never happen in practice, but provides a safe fallback
                let obj = js_sys::Object::new();
                obj.unchecked_into::<web_sys::FormData>()
            }
        }
    }

    fn error(&self) -> Option<&str> {
        // FormData doesn't have a direct way to check for errors
        // We check if it has an "error" field
        if let Ok(error_value) = self.get("error").as_string().ok_or("") {
            if !error_value.is_empty() {
                // This is a static string that we can't return a reference to
                // Return None since we can't provide a &str reference
                return None;
            }
        }
        None
    }

    fn is_error(&self) -> bool {
        // Check if FormData has an "error" field
        if let Ok(error_value) = self.get("error").as_string().ok_or("") {
            !error_value.is_empty()
        } else {
            false
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl MessageChunk for () {
    fn bad_chunk(_error: String) -> Self {
        ()
    }

    fn error(&self) -> Option<&str> {
        None
    }

    fn is_error(&self) -> bool {
        false
    }
}

pub mod body;
pub mod client;
pub mod request;
pub mod response;

// WASM-specific utilities
#[cfg(target_arch = "wasm32")]
use js_sys::{Array, Object, Promise, Uint8Array};

/// Handle JavaScript errors in WASM context
#[cfg(target_arch = "wasm32")]
pub fn handle_error(error: JsValue) -> Error {
    if let Some(error_str) = error.as_string() {
        Error::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            error_str,
        ))
    } else {
        Error::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Unknown JavaScript error",
        ))
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn handle_error(_error: String) -> Error {
    Error::from(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "WASM functionality not available on this platform",
    ))
}

/// Convert JavaScript Promise to AsyncStream using callback pattern
#[cfg(target_arch = "wasm32")]
pub fn promise_to_stream(promise: Promise) -> ystream::AsyncStream<JsValue, 1024> {
    use ystream::{AsyncStream, emit};
    use wasm_bindgen::prelude::*;
    
    AsyncStream::with_channel(move |sender| {
        // Use JavaScript Promise.then() instead of Future-based approach
        let success_callback = Closure::once_into_js(move |js_val: JsValue| {
            emit!(sender, js_val);
        });
        
        let error_callback = Closure::once_into_js(move |error: JsValue| {
            emit!(sender, JsValue::bad_chunk(format!("Promise error: {:?}", error)));
        });
        
        // Use native JavaScript Promise.then() method
        let _ = promise.then2(&success_callback, &error_callback);
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn promise_to_stream(_promise: ()) -> ystream::AsyncStream<JsValue, 1024> {
    use ystream::{AsyncStream, emit};
    
    AsyncStream::with_channel(move |sender| {
        emit!(sender, JsValue::bad_chunk(
            "WASM functionality not available on this platform".to_string()
        ));
    })
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
unsafe extern "C" {
    #[wasm_bindgen(js_name = "setTimeout")]
    fn set_timeout(handler: &Function, timeout: i32) -> JsValue;

    #[wasm_bindgen(js_name = "clearTimeout")]
    fn clear_timeout(handle: JsValue) -> JsValue;
}

fn promise<T>(
    promise: js_sys::Promise,
) -> ystream::AsyncStream<T, 1024>
where
    T: JsCast + MessageChunk,
{
    use ystream::{AsyncStream, emit};

    AsyncStream::with_channel(move |sender| {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::prelude::*;
            
            // Use JavaScript Promise.then() instead of Future-based approach
            let success_callback = Closure::once_into_js(move |js_val: JsValue| {
                match wasm_bindgen::JsCast::dyn_into::<T>(js_val) {
                    Ok(result) => emit!(sender, result),
                    Err(_js_val) => {
                        emit!(sender, T::bad_chunk("promise resolved to unexpected type".to_string()))
                    }
                }
            });
            
            let error_callback = Closure::once_into_js(move |error: JsValue| {
                emit!(sender, T::bad_chunk(format!("WASM error: {:?}", error)))
            });
            
            // Use native JavaScript Promise.then() method
            let _ = promise.then2(&success_callback, &error_callback);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            emit!(sender, T::bad_chunk("WASM functionality not available on this platform".to_string()));
        }
    })
}

/// A guard that cancels a fetch request when dropped.
#[cfg(target_arch = "wasm32")]
struct AbortGuard {
    ctrl: AbortController,
    timeout: Option<(JsValue, Closure<dyn FnMut()>)>,
}

#[cfg(not(target_arch = "wasm32"))]
struct AbortGuard;

impl AbortGuard {
    fn new() -> std::result::Result<Self, crate::HttpError> {
        Ok(AbortGuard {
            ctrl: AbortController::new()
                .map_err(|e| crate::Error::from(std::io::Error::new(std::io::ErrorKind::Other, format!("WASM error: {:?}", e))))
                .map_err(crate::client::core::ClientBuilder)?,
            timeout: None,
        })
    }

    fn signal(&self) -> AbortSignal {
        self.ctrl.signal()
    }

    #[cfg(target_arch = "wasm32")]
    fn timeout(&mut self, timeout: Duration) {
        let ctrl = self.ctrl.clone();
        let abort =
            Closure::once(move || ctrl.abort_with_reason(&"crate::client::HttpClient::errors::TimedOut".into()));
        let timeout = set_timeout(
            abort.as_ref().unchecked_ref::<js_sys::Function>(),
            match timeout.as_millis().try_into() {
                Ok(millis) => millis,
                Err(_) => return, // Skip if timeout conversion fails
            },
        );
        if let Some((id, _)) = self.timeout.replace((timeout, abort)) {
            clear_timeout(id);
        }
    }
}

impl Drop for AbortGuard {
    fn drop(&mut self) {
        self.ctrl.abort();
        if let Some((id, _)) = self.timeout.take() {
            #[cfg(target_arch = "wasm32")]
            if let Some(timeout_id) = id.as_f64() {
                clear_timeout(timeout_id as i32);
            }
        }
    }
}
