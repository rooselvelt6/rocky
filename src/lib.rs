// Re-export the uci module so it's available to both frontend and backend
pub mod uci;

// Frontend components (only compiled for wasm/csr)
#[cfg(feature = "csr")]
pub mod frontend;
