extern crate alloc;
pub mod app;
pub mod arguments;
pub mod config;
pub mod direct;
pub mod error;
pub mod mcp;
pub mod models;
pub mod net;
pub mod page;
pub mod search;
use mimalloc::MiMalloc;
#[global_allocator]
static GLOBAL_ALLOCATOR: MiMalloc = MiMalloc;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const HTTP_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.0.0 Safari/537.36";
pub type Result<T> = core::result::Result<T, error::AppError>;
