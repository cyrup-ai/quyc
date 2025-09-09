// Use latest web-sys methods for optimal performance and future compatibility
// Updated to use non-deprecated APIs for production quality

use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use quyc_client::Body;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: String);
}

#[wasm_bindgen_test]
fn test_body() {
    let body = Body::from("TEST");
    assert_eq!(
        [84, 69, 83, 84],
        body.as_bytes().expect("body should have bytes")
    );
}

#[wasm_bindgen_test]
fn test_body_js_static_str() {
    let body_value = "TEST";
    let body = Body::from(body_value);

    let mut init = web_sys::RequestInit::new();
    init.method("POST");
    init.body(Some(
        body.to_js_value()
            .expect("could not convert body to JsValue")
            .as_ref(),
    ));

    let js_req = web_sys::Request::new_with_str_and_init("", &init)
        .expect("could not create JS request");
    let text_promise = js_req.text().expect("could not get text promise");
    let text = quyc_client::wasm::promise::<JsValue>(text_promise)
        .collect()
        .into_iter()
        .find(|v| !v.is_error())
        .expect("could not get request body as text");

    assert_eq!(text.as_string().expect("text is not a string"), body_value);
}

#[wasm_bindgen_test]
fn test_body_js_string() {
    let body_value = "TEST".to_string();
    let body = Body::from(body_value.clone());

    let mut init = web_sys::RequestInit::new();
    init.method("POST");
    init.body(Some(
        body.to_js_value()
            .expect("could not convert body to JsValue")
            .as_ref(),
    ));

    let js_req = web_sys::Request::new_with_str_and_init("", &init)
        .expect("could not create JS request");
    let text_promise = js_req.text().expect("could not get text promise");
    let text = quyc_client::wasm::promise::<JsValue>(text_promise)
        .collect()
        .into_iter()
        .find(|v| !v.is_error())
        .expect("could not get request body as text");

    assert_eq!(text.as_string().expect("text is not a string"), body_value);
}

#[wasm_bindgen_test]
fn test_body_js_static_u8_slice() {
    let body_value: &'static [u8] = b"\x00\x42";
    let body = Body::from(body_value);

    let mut init = web_sys::RequestInit::new();
    init.method("POST");
    init.body(Some(
        body.to_js_value()
            .expect("could not convert body to JsValue")
            .as_ref(),
    ));

    let js_req = web_sys::Request::new_with_str_and_init("", &init)
        .expect("could not create JS request");

    let array_buffer_promise = js_req
        .array_buffer()
        .expect("could not get array_buffer promise");
    let array_buffer = quyc_client::wasm::promise::<JsValue>(array_buffer_promise)
        .collect()
        .into_iter()
        .find(|v| !v.is_error())
        .expect("could not get request body as array buffer");

    let v = Uint8Array::new(&array_buffer).to_vec();

    assert_eq!(v, body_value);
}

#[wasm_bindgen_test]
fn test_body_js_vec_u8() {
    let body_value = vec![0u8, 42];
    let body = Body::from(body_value.clone());

    let mut init = web_sys::RequestInit::new();
    init.method("POST");
    init.body(Some(
        body.to_js_value()
            .expect("could not convert body to JsValue")
            .as_ref(),
    ));

    let js_req = web_sys::Request::new_with_str_and_init("", &init)
        .expect("could not create JS request");

    let array_buffer_promise = js_req
        .array_buffer()
        .expect("could not get array_buffer promise");
    let array_buffer = quyc_client::wasm::promise::<JsValue>(array_buffer_promise)
        .collect()
        .into_iter()
        .find(|v| !v.is_error())
        .expect("could not get request body as array buffer");

    let v = Uint8Array::new(&array_buffer).to_vec();

    assert_eq!(v, body_value);
}