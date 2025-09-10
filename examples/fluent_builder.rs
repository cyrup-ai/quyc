//! Example usage of Http3 builder with exact user syntax patterns

use std::collections::HashMap;

use axum::http::Request;
use axum::{
    Router,
    body::Body,
    extract::{Form, Json},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next, map_response},
    response::{Json as ResponseJson, Response},
    routing::{get, post, put},
};

use cyrup_sugars::prelude::*;
use ystream::prelude::MessageChunk;
use quyc::{Http3, ContentType, HttpChunk, BadChunk};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Serialize, Deserialize, Debug)]
struct SerdeRequestType {
    message: String,
    data: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct SerdeResponseType {
    result: String,
    count: u32,
}

impl From<BadChunk> for SerdeResponseType {
    fn from(_bad_chunk: BadChunk) -> Self {
        SerdeResponseType {
            result: "error".to_string(),
            count: 0,
        }
    }
}

impl ystream::prelude::MessageChunk for SerdeResponseType {
    fn bad_chunk(_error: String) -> Self {
        Self {
            result: "error".to_string(),
            count: 0,
        }
    }
    
    fn error(&self) -> Option<&str> {
        if self.result == "error" {
            Some(&self.result)
        } else {
            None
        }
    }
}

// JSON request/response types
#[derive(Serialize, Deserialize, Debug, Clone)]
struct JsonRequest {
    user_id: u64,
    username: String,
    permissions: Vec<String>,
    metadata: std::collections::HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct JsonResponse {
    success: bool,
    user_id: u64,
    created_at: String,
    roles: Vec<String>,
    settings: std::collections::HashMap<String, i32>,
}

impl From<BadChunk> for JsonResponse {
    fn from(_bad_chunk: BadChunk) -> Self {
        JsonResponse {
            success: false,
            user_id: 0,
            created_at: String::new(),
            roles: Vec::new(),
            settings: std::collections::HashMap::new(),
        }
    }
}

impl ystream::prelude::MessageChunk for JsonResponse {
    fn bad_chunk(_error: String) -> Self {
        Self {
            success: false,
            user_id: 0,
            created_at: String::new(),
            roles: Vec::new(),
            settings: std::collections::HashMap::new(),
        }
    }
    
    fn error(&self) -> Option<&str> {
        if !self.success {
            Some("Request failed")
        } else {
            None
        }
    }
}

// Form request/response types
#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(dead_code)]
struct FormRequest {
    product_id: String,
    quantity: i32,
    price: f64,
    category: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct FormResponse {
    order_id: String,
    total_cost: f64,
    estimated_delivery: String,
    items: Vec<String>,
    discount_applied: bool,
}

impl From<BadChunk> for FormResponse {
    fn from(_bad_chunk: BadChunk) -> Self {
        FormResponse {
            order_id: String::new(),
            total_cost: 0.0,
            estimated_delivery: String::new(),
            items: Vec::new(),
            discount_applied: false,
        }
    }
}

impl ystream::prelude::MessageChunk for FormResponse {
    fn bad_chunk(_error: String) -> Self {
        Self {
            order_id: String::new(),
            total_cost: 0.0,
            estimated_delivery: String::new(),
            items: Vec::new(),
            discount_applied: false,
        }
    }
    
    fn error(&self) -> Option<&str> {
        if self.order_id.is_empty() {
            Some("Order processing failed")
        } else {
            None
        }
    }
}

// Binary/Text request/response types
#[derive(Serialize, Deserialize, Debug, Clone)]
struct BinaryRequest {
    file_name: String,
    file_size: u64,
    checksum: String,
    mime_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct BinaryResponse {
    upload_id: String,
    status: String,
    bytes_processed: u64,
    validation_result: bool,
}

impl From<BadChunk> for BinaryResponse {
    fn from(_bad_chunk: BadChunk) -> Self {
        BinaryResponse {
            upload_id: String::new(),
            status: "error".to_string(),
            bytes_processed: 0,
            validation_result: false,
        }
    }
}

impl ystream::prelude::MessageChunk for BinaryResponse {
    fn bad_chunk(_error: String) -> Self {
        Self {
            upload_id: String::new(),
            status: "error".to_string(),
            bytes_processed: 0,
            validation_result: false,
        }
    }
    
    fn error(&self) -> Option<&str> {
        if self.status == "error" {
            Some(&self.status)
        } else {
            None
        }
    }
}

// Handler for test server that logs received payload and headers
async fn handle_post(
    headers: HeaderMap,
    Json(payload): Json<SerdeRequestType>,
) -> ResponseJson<Vec<SerdeResponseType>> {
    println!("🚀 Server received payload: {:#?}", payload);
    println!("📋 Server received headers:");
    for (name, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            println!("   {}: {}", name, value_str);
        } else {
            println!("   {}: <binary_data>", name);
        }
    }
    println!();

    let response = SerdeResponseType {
        result: format!("Processed: {}", payload.message),
        count: payload.data.len() as u32,
    };

    println!("📤 Server responding with: {:#?}", response);
    ResponseJson(vec![response])
}

// Handler for CSV download
async fn handle_csv_download() -> Response<String> {
    let csv_data = "name,age,city\nJohn,30,NYC\nJane,25,LA\nBob,35,Chicago";

    Response::builder()
        .header("content-type", "text/csv")
        .header("content-disposition", "attachment; filename=\"test.csv\"")
        .body(csv_data.to_string())
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Error building response".to_string())
                .unwrap()
        })
}

// PUT handler for JSON content - JsonRequest -> JsonResponse
async fn handle_put_json(
    headers: HeaderMap,
    Json(payload): Json<JsonRequest>,
) -> Result<ResponseJson<Vec<JsonResponse>>, StatusCode> {
    println!("🔄 PUT JSON received: {:#?}", payload);
    println!("📋 Headers: {:?}", headers.get("content-type"));

    // Transform JsonRequest -> JsonResponse to prove different serialization/deserialization
    let mut settings = std::collections::HashMap::new();
    settings.insert("notifications".to_string(), 1);
    settings.insert("theme".to_string(), 2);
    settings.insert("language".to_string(), 3);

    let response = JsonResponse {
        success: true,
        user_id: payload.user_id + 1000,
        created_at: chrono::Utc::now().to_rfc3339(),
        roles: payload
            .permissions
            .into_iter()
            .map(|p| format!("role_{}", p))
            .collect(),
        settings,
    };

    println!("📤 PUT JSON responding: {:#?}", response);
    Ok(ResponseJson(vec![response]))
}

// PUT handler for form-urlencoded content - FormRequest -> FormResponse
async fn handle_put_form(
    headers: HeaderMap,
    Form(params): Form<HashMap<String, String>>,
) -> Result<ResponseJson<Vec<FormResponse>>, StatusCode> {
    println!("🔄 PUT Form received: {:#?}", params);
    println!("📋 Headers: {:?}", headers.get("content-type"));

    // Parse form params into FormRequest-like data, respond with FormResponse
    let product_id = params
        .get("product_id")
        .map_or("unknown", |s| s.as_str())
        .to_string();
    let quantity: i32 = params
        .get("quantity")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    let price: f64 = params
        .get("price")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);

    let response = FormResponse {
        order_id: format!("ORD-{}-{}", product_id, chrono::Utc::now().timestamp()),
        total_cost: price * quantity as f64,
        estimated_delivery: "2025-01-30".to_string(),
        items: vec![format!("{} x{}", product_id, quantity)],
        discount_applied: quantity > 5,
    };

    println!("📤 PUT Form responding: {:#?}", response);
    Ok(ResponseJson(vec![response]))
}

// PUT handler for binary/text content - BinaryRequest -> BinaryResponse
async fn handle_put_binary(
    headers: HeaderMap,
    Json(payload): Json<BinaryRequest>,
) -> Result<ResponseJson<Vec<BinaryResponse>>, StatusCode> {
    println!("🔄 PUT Binary received: {:#?}", payload);
    println!("📋 Headers: {:?}", headers.get("content-type"));

    // Transform BinaryRequest -> BinaryResponse
    let response = BinaryResponse {
        upload_id: format!("UPLOAD-{}", chrono::Utc::now().timestamp()),
        status: "processed".to_string(),
        bytes_processed: payload.file_size,
        validation_result: payload.checksum.len() > 10,
    };

    println!("📤 PUT Binary responding: {:#?}", response);
    Ok(ResponseJson(vec![response]))
}

// Requestbin-style middleware to log ALL incoming requests with full details
async fn requestbin_logger(
    req: Request<Body>,
    next: Next,
) -> Result<axum::response::Response, StatusCode> {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let headers = req.headers().clone();
    let version = req.version();

    println!("\n🔍 REQUESTBIN: Incoming {} {} {:?}", method, uri, version);
    println!("📋 Headers ({} total):", headers.len());

    for (name, value) in headers.iter() {
        match value.to_str() {
            Ok(value_str) => println!("   {}: {}", name, value_str),
            Err(_) => println!("   {}: <binary_data>", name),
        }
    }

    // Extract and log the body
    let (parts, body) = req.into_parts();
    let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => {
            println!("❌ Error reading body: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if !body_bytes.is_empty() {
        println!("📤 Body ({} bytes):", body_bytes.len());
        match std::str::from_utf8(&body_bytes) {
            Ok(body_str) => println!("{}", body_str),
            Err(_) => println!("<binary_data>"),
        }
    } else {
        println!("📤 Body: <empty>");
    }

    println!(""); // Empty line for readability

    // Reconstruct request with the consumed body
    let reconstructed_req = Request::from_parts(parts, Body::from(body_bytes));

    // Continue to the next middleware/handler
    let response = next.run(reconstructed_req).await;

    Ok(response)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize rustls crypto provider
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    
    // Initialize env_logger for http3's native debug logging
    unsafe {
        std::env::set_var("RUST_LOG", "http3=debug,hyper=debug,quyc=debug");
    }
    env_logger::init();
    println!("✨ Enabled http3's native HTTP debug logging");

    // Start local test server on random port with requestbin logging
    let app = Router::new()
        .route("/test", post(handle_post))
        .route("/put/json", put(handle_put_json))
        .route("/put/form", put(handle_put_form))
        .route("/put/binary", put(handle_put_binary))
        .route("/download/file.csv", get(handle_csv_download))
        .layer(middleware::from_fn(requestbin_logger))
        .layer(map_response(add_alt_svc_header));

    // Use thread-based runtime for server infrastructure  
    let rt = tokio::runtime::Runtime::new()?;
    let (local_addr, server_handle) = rt.block_on(async {
        // Use a fixed port for HTTP
        let addr = SocketAddr::from(([127, 0, 0, 1], 3030));
        
        println!("🌍 Test server starting on http://{}", addr);

        // Spawn HTTP server in background thread
        let listener = tokio::net::TcpListener::bind(addr).await?;
        let server_handle = tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .unwrap();
        });

        // Give server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        Ok::<(std::net::SocketAddr, tokio::task::JoinHandle<()>), Box<dyn std::error::Error>>((
            addr,
            server_handle,
        ))
    })?;

    // Create test request
    let request = SerdeRequestType {
        message: "Hello HTTP3 Builder!".to_string(),
        data: vec!["test".to_string(), "data".to_string()],
    };

    let server_url = format!("http://{}/test", local_addr);

    println!("📡 Testing Http3 builder with local server...");
    println!("📝 Sending request payload: {:#?}", request);
    println!("🌐 Server URL: {}", server_url);

    println!("🧪 Testing Http3 builder with EXACT syntax patterns...\n");

    // Stream of HttpChunk or mixed BadHttpChunk
    let _response: SerdeResponseType = Http3::json()
        .debug() // Enable debug logging
        .headers([("x-api-key", "abc123")])
        .body(&request)
        .post(&server_url).collect_one();

    // collect to Serde mapped type
    let response_data: SerdeResponseType = Http3::json()
        .accept_content_type(ContentType::ApplicationJson)
        .headers([("x-api-key", "abc123")])
        .body(&request)
        .post(&server_url)
        .collect_one();
    println!("📥 Received response: {:?}", response_data);

    // shorthand
    let response_data2: SerdeResponseType = Http3::json()
        .api_key("abc123")
        .body(&request)
        .post(&server_url)
        .collect_one();
    println!("📥 Received response 2: {:?}", response_data2);

    // shorthand - form urlencoded with development config
    let _serde_response_type: SerdeResponseType = Http3::json()
        .basic_auth([("user", "password")])
        .body(&request)
        .post(&server_url)
        .collect_one();

    // Demonstrate cyrup_sugars on_chunk beautiful syntax
    println!("🧪 Testing cyrup_sugars on_chunk beautiful syntax...");

    // Note: This demonstrates the beautiful on_chunk syntax with raw HTTP chunks
    let _stream_with_error_handling: ystream::AsyncStream<SerdeResponseType, 1024> = Http3::json()
        .headers([("foo", "bar"), ("fizz", "buzz")])
        .body(&request)
        .on_chunk(|result| match result {
            Ok(chunk) => {
                println!("Received HTTP chunk: {:?}", chunk);
                chunk
            }
            Err(e) => {
                println!("HTTP error occurred: {:?}", e);
                HttpChunk::bad_chunk(e.to_string())
            }
        })
        .post::<SerdeResponseType>(&server_url);

    println!("✅ cyrup_sugars on_chunk syntax demonstrated successfully!");

    // Stream of HttpChunk may have mixed BadHttpChunk
    let error_response = Http3::json()
        .headers([("foo", "bar"), ("fizz", "buzz")])
        .body(&request)
        .post(&server_url)
        .collect_one_or_else(|_e| SerdeResponseType {
            result: "error".to_string(),
            count: 0,
        });
    println!("📥 Error response: {:?}", error_response);

    // Download file example with proper URL
    let csv_url = format!("http://{}/download/file.csv", local_addr);
    let download_result = Http3::json()
        .headers([("x-api-key", "abc123")])
        .download_file(&csv_url)
        .save("/tmp/some.csv")
        .collect(); // fluent-ai AsyncStream collection pattern

    println!("📥 Download result: {:?}", download_result);

    // Test comprehensive PUT endpoints with different serialization/deserialization types
    println!("\n🧪 Testing PUT endpoints with different content types...\n");

    // PUT JSON test - JsonRequest -> JsonResponse
    let json_request = JsonRequest {
        user_id: 42,
        username: "test_user".to_string(),
        permissions: vec!["read".to_string(), "write".to_string()],
        metadata: {
            let mut map = std::collections::HashMap::new();
            map.insert("department".to_string(), "engineering".to_string());
            map.insert("location".to_string(), "remote".to_string());
            map
        },
    };

    let json_url = format!("http://{}/put/json", local_addr);
    let json_response: JsonResponse = Http3::json()
        .debug()
        .body(&json_request)
        .put(&json_url)
        .collect_one();
    println!("📤 PUT JSON Response: {:#?}", json_response);

    // PUT Form test - Send actual form-urlencoded data, not JSON
    let form_params = std::collections::HashMap::from([
        ("product_id".to_string(), "LAPTOP_001".to_string()),
        ("quantity".to_string(), "3".to_string()),
        ("price".to_string(), "999.99".to_string()),
        ("category".to_string(), "electronics".to_string()),
    ]);

    let form_url = format!("http://{}/put/form", local_addr);
    let form_response: FormResponse = Http3::json()
        .debug()
        .body(&form_params)
        .put(&form_url)
        .collect_one();
    println!("📤 PUT Form Response: {:#?}", form_response);

    // PUT Binary test - BinaryRequest -> BinaryResponse
    let binary_request = BinaryRequest {
        file_name: "document.pdf".to_string(),
        file_size: 1024000,
        checksum: "sha256:abc123def456".to_string(),
        mime_type: "application/pdf".to_string(),
    };

    let binary_url = format!("http://{}/put/binary", local_addr);
    let binary_response: BinaryResponse = Http3::json()
        .debug()
        .body(&binary_request)
        .put(&binary_url)
        .collect_one();
    println!("📤 PUT Binary Response: {:#?}", binary_response);

    // Give the server a moment to process all requests
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Properly shut down the server
    server_handle.abort();
    println!("🛑 Server shutdown initiated");

    println!("\n✅ All PUT endpoint tests completed successfully!");
    println!("🎯 HTTP3 builder example completed!");
    Ok(())
}

// Alt-Svc header middleware for HTTP/3 discovery (RFC 7838)
async fn add_alt_svc_header<B>(mut response: Response<B>) -> Response<B> {
    // Advertise HTTP/3 support on port 3031 (where HTTP/3 server would run)
    response.headers_mut().insert(
        "alt-svc", 
        "h3=\":3031\"; ma=86400".parse().unwrap()
    );
    response
}
