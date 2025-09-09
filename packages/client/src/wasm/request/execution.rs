//! Request execution and building methods
//!
//! This module contains methods for building and executing HTTP requests,
//! including streaming response handling and request cloning.

use ystream::AsyncStream;
use super::{http::HttpRequest, RequestBuilder};
use super::{Client, Response};

impl RequestBuilder {
    /// Build a `Request`, which can be inspected, modified and executed with
    /// `Client::execute()`.
    pub fn build(self) -> std::result::Result<Request, crate::HttpError> {
        self.request
    }

    /// Build a `Request`, which can be inspected, modified and executed with
    /// `Client::execute()`.
    ///
    /// This is similar to [`RequestBuilder::build()`], but also returns the
    /// embedded `Client`.
    pub fn build_split(self) -> (Client, std::result::Result<Request, crate::HttpError>) {
        (self.client, self.request)
    }

    /// Constructs the Request and sends it to the target URL, returning a
    /// pure Response stream - NO Result wrapping per fluent-ai architecture.
    ///
    /// # Pure Streaming
    ///
    /// This method returns unwrapped Response stream. Errors become 
    /// Response::bad_chunk() items in the stream, not Result::Err values.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ystream::prelude::*;
    /// #
    /// # fn run() {
    /// let client = crate::client::HttpClient::new();
    /// let request = crate::http::HttpRequest::get("https://hyper.rs");
    /// let response_stream = client.execute(request);
    /// 
    /// for response in response_stream {
    ///     if response.is_error() {
    ///         if let Some(error_msg) = response.error() {
    ///             eprintln!("Error: {}", error_msg);
    ///         } else {
    ///             eprintln!("Unknown error occurred");
    ///         }
    ///     } else {
    ///         println!("Success response received");
    ///     }
    /// }
    /// # }
    /// ```
    pub fn send(self) -> AsyncStream<Response, 1024> {
        use ystream::emit;
        use ystream::prelude::MessageChunk;
        
        AsyncStream::with_channel(move |sender| {
            let req = match self.request {
                Ok(req) => req,
                Err(e) => {
                    emit!(sender, Response::bad_chunk(e.to_string()));
                    return;
                }
            };
            
            let result_stream = self.client.execute_request(req);
            for result in result_stream {
                match result {
                    Ok(response) => emit!(sender, response),
                    Err(error) => emit!(sender, Response::bad_chunk(error.to_string())),
                }
            }
        })
    }

    /// Attempt to crate::client::core::ClientBuilder.
    ///
    /// `None` is returned if the RequestBuilder can not be cloned.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use crate::prelude::HttpError;
    /// #
    /// # fn run() -> Result<(), Error> {
    /// let client = crate::client::HttpClient::new();
    /// let request = crate::http::HttpRequest::post("http://httpbin.org/post")
    ///     .body("from a &str!");
    /// let clone = builder.try_clone();
    /// assert!(clone.is_some());
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_clone(&self) -> Option<RequestBuilder> {
        self.request
            .as_ref()
            .ok()
            .and_then(|req| req.try_clone())
            .map(|req| RequestBuilder {
                client: self.client.clone(),
                request: Ok(req),
            })
    }
}