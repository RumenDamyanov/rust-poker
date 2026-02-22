//! Axum router with all routes and middleware.

use axum::Router;
use axum::routing::{get, post};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use super::handlers;
use super::state::SharedState;

/// Build the Axum router with all routes and middleware.
pub fn create_router(state: SharedState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Health check
        .route("/health", get(handlers::health))
        // Rulesets
        .route("/api/rulesets", get(handlers::list_rulesets))
        // Game CRUD
        .route(
            "/api/games",
            post(handlers::create_game).get(handlers::list_games),
        )
        .route(
            "/api/games/{id}",
            get(handlers::get_game).delete(handlers::delete_game),
        )
        // Player management
        .route("/api/games/{id}/join", post(handlers::join_game))
        .route("/api/games/{id}/leave", post(handlers::leave_game))
        // Game flow
        .route("/api/games/{id}/start", post(handlers::start_hand))
        .route("/api/games/{id}/action", post(handlers::player_action))
        // Player view
        .route(
            "/api/games/{id}/player/{player_id}",
            get(handlers::player_view),
        )
        // History
        .route("/api/games/{id}/history", get(handlers::history))
        // Middleware
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
