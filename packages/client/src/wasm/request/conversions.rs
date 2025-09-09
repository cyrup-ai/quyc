use std::convert::TryFrom;
use std::fmt;

use bytes::Bytes;
use http::{Request as HttpRequest, request::Parts};
use url::Url;

use super::{Body, Request, RequestBuilder};

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt_request_fields(&mut f.debug_struct("Request"), self).finish()
    }
}

impl fmt::Debug for RequestBuilder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut builder = f.debug_struct("RequestBuilder");
        match self.request {
            Ok(ref req) => fmt_request_fields(&mut builder, req).finish(),
            Err(ref err) => builder.field("error", err).finish(),
        }
    }
}

fn fmt_request_fields<'a, 'b>(
    f: &'a mut fmt::DebugStruct<'a, 'b>,
    req: &Request,
) -> &'a mut fmt::DebugStruct<'a, 'b> {
    f.field("method", &req.method)
        .field("url", &req.url)
        .field("headers", &req.headers)
}

impl<T> TryFrom<HttpRequest<T>> for Request
where
    T: Into<Body>,
{
    type Error = crate::Error;

    fn try_from(req: HttpRequest<T>) -> std::result::Result<Self, crate::Error> {
        let (parts, body) = req.into_parts();
        let Parts {
            method,
            uri,
            headers,
            ..
        } = parts;
        let url = Url::parse(&uri.to_string()).map_err(crate::error::builder)?;
        Ok(Request {
            method,
            url,
            headers,
            body: Some(body.into()),
            timeout: None,
            cors: true,
            credentials: None,
            cache: None,
        })
    }
}

impl TryFrom<Request> for HttpRequest<Body> {
    type Error = crate::Error;

    fn try_from(req: Request) -> std::result::Result<Self, crate::Error> {
        let Request {
            method,
            url,
            headers,
            body,
            ..
        } = req;

        let mut req = HttpRequest::builder()
            .method(method)
            .uri(url.as_str())
            .body(body.unwrap_or_else(|| Body::from(Bytes::default())))
            .map_err(crate::error::builder)?;

        *req.headers_mut() = headers;
        Ok(req)
    }
}
