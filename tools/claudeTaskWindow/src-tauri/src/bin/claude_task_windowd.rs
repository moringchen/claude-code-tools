use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use claude_task_window::{server::build_router, store::TaskStore};

#[tokio::main]
async fn main() {
    let store = Arc::new(Mutex::new(TaskStore::default()));
    let app = build_router(store);
    let addr = SocketAddr::from(([127, 0, 0, 1], 46123));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
