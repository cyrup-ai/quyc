//! Tests for multipart/form-data functionality
//!
//! WASM-specific tests for multipart form handling, including text parts,
//! binary parts, and form data conversion for browser environments.

use wasm_bindgen_test::*;
use quyc_client::wasm::multipart::{Form, Part};

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_multipart_js() {
    use js_sys::Uint8Array;
    use wasm_bindgen::JsValue;
    use web_sys::{File, FormData};

    let text_file_name = "test.txt";
    let text_file_type = "text/plain";
    let text_content = "TEST";
    let text_part = Part::text(text_content)
        .file_name(text_file_name)
        .mime_str(text_file_type)
        .expect("invalid mime type");

    let binary_file_name = "binary.bin";
    let binary_file_type = "application/octet-stream";
    let binary_content = vec![0u8, 42];
    let binary_part = Part::bytes(binary_content.clone())
        .file_name(binary_file_name)
        .mime_str(binary_file_type)
        .expect("invalid mime type");

    let string_name = "string";
    let string_content = "CONTENT";
    let string_part = Part::text(string_content);

    let text_name = "text part";
    let binary_name = "binary part";
    let form = Form::new()
        .part(text_name, text_part)
        .part(binary_name, binary_part)
        .part(string_name, string_part);

    let mut init = web_sys::RequestInit::new();
    init.method("POST");
    init.body(Some(
        form.to_form_data()
            .expect("could not convert to FormData")
            .as_ref(),
    ));

    let js_req = web_sys::Request::new_with_str_and_init("", &init)
        .expect("could not create JS request");

    let form_data_promise = js_req.form_data().expect("could not get form_data promise");

    let form_data = quyc_client::wasm::promise::<FormData>(form_data_promise)
        .collect()
        .into_iter()
        .find(|v| !v.is_error())
        .expect("could not get body as form data");

    // check text part
    let text_file = File::from(form_data.get(text_name));
    assert_eq!(text_file.name(), text_file_name);
    assert_eq!(text_file.type_(), text_file_type);

    let text_promise = text_file.text();
    let text = quyc_client::wasm::promise::<JsValue>(text_promise)
        .collect()
        .into_iter()
        .find(|v| !v.is_error())
        .expect("could not get text body as text");
    assert_eq!(
        text.as_string().expect("text is not a string"),
        text_content
    );

    // check binary part
    let binary_file = File::from(form_data.get(binary_name));
    assert_eq!(binary_file.name(), binary_file_name);
    assert_eq!(binary_file.type_(), binary_file_type);

    // check string part
    let string = form_data
        .get(string_name)
        .as_string()
        .expect("content is not a string");
    assert_eq!(string, string_content);

    let binary_array_buffer_promise = binary_file.array_buffer();
    let array_buffer = quyc_client::wasm::promise::<JsValue>(binary_array_buffer_promise)
        .collect()
        .into_iter()
        .find(|v| !v.is_error())
        .expect("could not get request body as array buffer");

    let binary = Uint8Array::new(&array_buffer).to_vec();

    assert_eq!(binary, binary_content);
}