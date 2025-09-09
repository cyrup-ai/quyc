//! Redirect Handling
//!
//! By default, a `Client` will automatically handle HTTP redirects, having a
//! maximum redirect chain of 10 hops. To customize this behavior, a
//! `redirect::Policy` can be used with a `ClientBuilder`.

mod attempt;
mod headers;
mod policy;
mod tower;



// Re-export main types for backward compatibility
// Re-export internal types for module coordination
pub(crate) use attempt::ActionKind;
pub use attempt::{Action, Attempt};
pub(crate) use headers::{make_referer, remove_sensitive_headers};
pub use policy::Policy;
pub(crate) use tower::TowerRedirectPolicy;
