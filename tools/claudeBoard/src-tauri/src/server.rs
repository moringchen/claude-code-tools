use std::io;
use std::sync::{Arc, Mutex};

use axum::{
    extract::State,
    http::{header, Method},
    routing::{get, post},
    Json, Router,
};
use tower_http::cors::{Any, CorsLayer};

use crate::{
    model::{HookEvent, TaskSnapshot},
    scan::rebuild_tasks_from_rows,
    store::TaskStore,
};

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<Mutex<TaskStore>>,
    pub scan_refresh: Arc<dyn Fn() -> io::Result<()> + Send + Sync>,
}

pub fn build_router<F, N>(store: Arc<Mutex<TaskStore>>, scan_rows: F, now: N) -> Router
where
    F: Fn() -> io::Result<Vec<String>> + Send + Sync + 'static,
    N: Fn() -> String + Send + Sync + 'static,
{
    let refresh_store = Arc::clone(&store);
    let scan_refresh = Arc::new(move || {
        eprintln!("[claudeBoard] refresh start source=router");
        let rows = scan_rows()?;
        eprintln!("[claudeBoard] refresh scan_rows source=router row_count={}", rows.len());
        let timestamp = now();
        eprintln!("[claudeBoard] refresh timestamp source=router value={timestamp}");
        let tasks = rebuild_tasks_from_rows(&rows, &timestamp);
        eprintln!("[claudeBoard] refresh rebuilt source=router task_count={}", tasks.len());
        refresh_store.lock().unwrap().replace_scanned_tasks(tasks);
        Ok(())
    });

    Router::new()
        .route("/events", post(post_event))
        .route("/refresh", post(post_refresh))
        .route("/snapshot", get(get_snapshot))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_headers([header::CONTENT_TYPE]),
        )
        .with_state(AppState {
            store,
            scan_refresh,
        })
}

pub fn refresh_scan<F, N>(store: &Arc<Mutex<TaskStore>>, scan_rows: F, now: N) -> io::Result<()>
where
    F: Fn() -> io::Result<Vec<String>>,
    N: Fn() -> String,
{
    eprintln!("[claudeBoard] refresh start source=function");
    let rows = scan_rows()?;
    eprintln!("[claudeBoard] refresh scan_rows source=function row_count={}", rows.len());
    let timestamp = now();
    eprintln!("[claudeBoard] refresh timestamp source=function value={timestamp}");
    let tasks = rebuild_tasks_from_rows(&rows, &timestamp);
    eprintln!("[claudeBoard] refresh rebuilt source=function task_count={}", tasks.len());
    store.lock().unwrap().replace_scanned_tasks(tasks);
    Ok(())
}

async fn post_event(
    State(state): State<AppState>,
    Json(event): Json<HookEvent>,
) -> axum::http::StatusCode {
    state.store.lock().unwrap().apply(event);
    axum::http::StatusCode::ACCEPTED
}

async fn post_refresh(State(state): State<AppState>) -> axum::http::StatusCode {
    match (state.scan_refresh)() {
        Ok(()) => axum::http::StatusCode::ACCEPTED,
        Err(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn get_snapshot(State(state): State<AppState>) -> Json<TaskSnapshot> {
    let snapshot = state.store.lock().unwrap().snapshot();
    eprintln!(
        "[claudeBoard] snapshot served total={} running={} needs_user={} completed={}",
        snapshot.counts.total,
        snapshot.counts.running,
        snapshot.counts.needs_user,
        snapshot.counts.completed
    );
    Json(snapshot)
}
