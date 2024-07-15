use std::sync::Arc;

use axum::{handler::HandlerWithoutStateExt, http::StatusCode, Router};
use tower_http::services::ServeDir;

use crate::server::MyExtensionHub;

pub fn wrap_files_router(state: Arc<MyExtensionHub>, router: Router) -> Router {
    async fn handle_404() -> (StatusCode, &'static str) {
        (StatusCode::NOT_FOUND, "Not found")
    }

    let service = handle_404.into_service();

    let server_dir = ServeDir::new(&state.config.base_dir).not_found_service(service);
    router.fallback_service(server_dir)
}
