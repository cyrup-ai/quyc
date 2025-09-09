use ystream::AsyncStream;

use super::{Client, Request, RequestBuilder, Response};

impl RequestBuilder {
    /// Build a `Request`, which can be inspected, modified and executed with
    /// `Client::execute()`. Returns unwrapped Request per fluent-ai architecture.
    pub fn build(self) -> Request {
        match self.request {
            Ok(req) => req,
            Err(e) => Request::bad_chunk(e.to_string()),
        }
    }

    /// Build a `Request`, which can be inspected, modified and executed with
    /// `Client::execute()`.
    ///
    /// This is similar to [`RequestBuilder::build()`], but also returns the
    /// embedded `Client`. Returns unwrapped Request per fluent-ai architecture.
    pub fn build_split(self) -> (Client, Request) {
        let request = match self.request {
            Ok(req) => req,
            Err(e) => Request::bad_chunk(e.to_string()),
        };
        (self.client, request)
    }

    /// Constructs the Request and sends it to the target URL, returning a
    /// pure Response stream - NO Result wrapping per fluent-ai architecture.
    ///
    /// # Pure Streaming
    ///
    /// Returns unwrapped Response stream. Errors become Response::bad_chunk() 
    /// items in the stream, not Result::Err values.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ystream::prelude::*;
    /// #
    /// # fn run() {
    /// let client = crate::wasm::client::Client::new();
    /// let request = crate::http::HttpRequest::get("https://example.com");
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

    /// Attempt to clone the RequestBuilder.
    ///
    /// `None` is returned if the RequestBuilder can not be cloned.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use crate::error::Error;
    /// #
    /// # fn run() -> Result<(), Error> {
    /// let client = crate::wasm::client::Client::new();
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
