pub mod client;
pub mod schema;

#[cfg(feature = "server")]
pub mod server;

pub mod fetch;
#[cfg(not(target_arch = "wasm32"))]
pub mod fetch_native;
#[cfg(target_arch = "wasm32")]
pub mod fetch_web;
