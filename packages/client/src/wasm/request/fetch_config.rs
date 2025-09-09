//! WASM-specific fetch configuration methods
//!
//! This module contains methods for configuring WebAssembly fetch API options
//! such as CORS mode, credentials, and cache settings.

use web_sys::{RequestCache, RequestCredentials};
use super::types::RequestBuilder;

impl RequestBuilder {
    /// Disable CORS on fetching the request.
    ///
    /// # WASM
    ///
    /// This option is only effective with WebAssembly target.
    ///
    /// The [request mode][mdn] will be set to 'no-cors'.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Request/mode
    pub fn fetch_mode_no_cors(mut self) -> RequestBuilder {
        if let Ok(ref mut req) = self.request {
            req.cors = false;
        }
        self
    }

    /// Set fetch credentials to 'same-origin'
    ///
    /// # WASM
    ///
    /// This option is only effective with WebAssembly target.
    ///
    /// The [request credentials][mdn] will be set to 'same-origin'.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Request/credentials
    pub fn fetch_credentials_same_origin(mut self) -> RequestBuilder {
        if let Ok(ref mut req) = self.request {
            req.credentials = Some(RequestCredentials::SameOrigin);
        }
        self
    }

    /// Set fetch credentials to 'include'
    ///
    /// # WASM
    ///
    /// This option is only effective with WebAssembly target.
    ///
    /// The [request credentials][mdn] will be set to 'include'.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Request/credentials
    pub fn fetch_credentials_include(mut self) -> RequestBuilder {
        if let Ok(ref mut req) = self.request {
            req.credentials = Some(RequestCredentials::Include);
        }
        self
    }

    /// Set fetch credentials to 'omit'
    ///
    /// # WASM
    ///
    /// This option is only effective with WebAssembly target.
    ///
    /// The [request credentials][mdn] will be set to 'omit'.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Request/credentials
    pub fn fetch_credentials_omit(mut self) -> RequestBuilder {
        if let Ok(ref mut req) = self.request {
            req.credentials = Some(RequestCredentials::Omit);
        }
        self
    }

    /// Set fetch cache mode to 'default'.
    ///
    /// # WASM
    ///
    /// This option is only effective with WebAssembly target.
    ///
    /// The [request cache][mdn] will be set to 'default'.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Request/cache
    pub fn fetch_cache_default(mut self) -> RequestBuilder {
        if let Ok(ref mut req) = self.request {
            req.cache = Some(RequestCache::Default);
        }
        self
    }

    /// Set fetch cache mode to 'no-store'.
    ///
    /// # WASM
    ///
    /// This option is only effective with WebAssembly target.
    ///
    /// The [request cache][mdn] will be set to 'no-store'.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Request/cache
    pub fn fetch_cache_no_store(mut self) -> RequestBuilder {
        if let Ok(ref mut req) = self.request {
            req.cache = Some(RequestCache::NoStore);
        }
        self
    }

    /// Set fetch cache mode to 'reload'.
    ///
    /// # WASM
    ///
    /// This option is only effective with WebAssembly target.
    ///
    /// The [request cache][mdn] will be set to 'reload'.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Request/cache
    pub fn fetch_cache_reload(mut self) -> RequestBuilder {
        if let Ok(ref mut req) = self.request {
            req.cache = Some(RequestCache::Reload);
        }
        self
    }

    /// Set fetch cache mode to 'no-cache'.
    ///
    /// # WASM
    ///
    /// This option is only effective with WebAssembly target.
    ///
    /// The [request cache][mdn] will be set to 'no-cache'.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Request/cache
    pub fn fetch_cache_no_cache(mut self) -> RequestBuilder {
        if let Ok(ref mut req) = self.request {
            req.cache = Some(RequestCache::NoCache);
        }
        self
    }

    /// Set fetch cache mode to 'force-cache'.
    ///
    /// # WASM
    ///
    /// This option is only effective with WebAssembly target.
    ///
    /// The [request cache][mdn] will be set to 'force-cache'.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Request/cache
    pub fn fetch_cache_force_cache(mut self) -> RequestBuilder {
        if let Ok(ref mut req) = self.request {
            req.cache = Some(RequestCache::ForceCache);
        }
        self
    }

    /// Set fetch cache mode to 'only-if-cached'.
    ///
    /// # WASM
    ///
    /// This option is only effective with WebAssembly target.
    ///
    /// The [request cache][mdn] will be set to 'only-if-cached'.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/API/Request/cache
    pub fn fetch_cache_only_if_cached(mut self) -> RequestBuilder {
        if let Ok(ref mut req) = self.request {
            req.cache = Some(RequestCache::OnlyIfCached);
        }
        self
    }
}