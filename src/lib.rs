// Re-export the uci module so it's available to both frontend and backend
// Re-export the uci module so it's available to both frontend and backend
pub mod uci;

// Frontend components (only compiled for wasm/csr)
#[cfg(feature = "csr")]
pub mod frontend;

pub mod models; // Also useful to share DB types if needed, but DB logic is backend only usually.
                // pub mod db; // Removed to avoid WASM issues. db is backend only.
                // Actually db.rs uses tokio/surreal which might not compile on wasm easily if not careful.
                // Let's keep db in main for now or check if it's safe.
                // models is definitely safe (structs).

#[cfg(feature = "ssr")]
pub mod services;
