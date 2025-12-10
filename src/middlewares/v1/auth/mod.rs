#[allow(clippy::module_inception)]
pub mod auth;
pub mod authenticated;
pub(crate) mod internal;

// Re-export the public Auth type (without token ID) for external use
pub use auth::Auth as PublicAuth;

// Re-export Authenticated and its internal Auth type for caching
pub use authenticated::{Authenticated, Auth};
