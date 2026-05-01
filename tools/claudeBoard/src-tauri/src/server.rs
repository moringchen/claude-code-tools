use std::sync::{Arc, Mutex};

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};

use crate::{
    model::{HookEvent, TaskSnapshot},
    store::TaskStore,
};

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<Mutex<TaskStore>>,
}

pub fn build_router(store: Arc<Mutex<TaskStore>>) -> Router {
    Router::new()
        .route("/events", post(post_event))
        .route("/snapshot", get(get_snapshot))
        .with_state(AppState { store })
}

async fn post_event(
    State(state): State<AppState>,
    Json(event): Json<HookEvent>,
) -> axum::http::StatusCode {
    state.store.lock().unwrap().apply(event);
    axum::http::StatusCode::ACCEPTED
}

async fn get_snapshot(State(state): State<AppState>) -> Json<TaskSnapshot> {
    Json(state.store.lock().unwrap().snapshot())
}
