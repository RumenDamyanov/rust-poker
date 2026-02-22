//! Integration tests for the Poker REST API.
//!
//! Uses Axum's test client to make HTTP requests without starting a real server.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use rumenx_poker::api::router::create_router;
use rumenx_poker::api::state::AppState;
use tower::ServiceExt;

/// Build a test app (router + fresh state).
fn app() -> axum::Router {
    let state = AppState::new();
    create_router(state)
}

/// Helper: make a GET request and return (status, body as JSON value).
async fn get(app: &axum::Router, uri: &str) -> (StatusCode, serde_json::Value) {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    (status, json)
}

/// Helper: make a POST request with JSON body.
async fn post_json(
    app: &axum::Router,
    uri: &str,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let req = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    (status, json)
}

/// Helper: make a POST request with empty body.
async fn post_empty(app: &axum::Router, uri: &str) -> (StatusCode, serde_json::Value) {
    let req = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from("{}"))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    (status, json)
}

/// Helper: make a DELETE request.
async fn delete(app: &axum::Router, uri: &str) -> (StatusCode, serde_json::Value) {
    let req = Request::builder()
        .method("DELETE")
        .uri(uri)
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    (status, json)
}

// ---------------------------------------------------------------------------
// Health
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_health_endpoint() {
    let app = app();
    let (status, body) = get(&app, "/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
    assert!(body["version"].is_string());
}

// ---------------------------------------------------------------------------
// Rulesets
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_list_rulesets() {
    let app = app();
    let (status, body) = get(&app, "/api/rulesets").await;
    assert_eq!(status, StatusCode::OK);
    let rulesets = body.as_array().unwrap();
    assert!(rulesets.len() >= 5, "should have at least 5 variants");

    // Check Texas Hold'em is present.
    let names: Vec<&str> = rulesets.iter().filter_map(|r| r["name"].as_str()).collect();
    assert!(names.contains(&"texas_holdem"));
    assert!(names.contains(&"omaha"));
    assert!(names.contains(&"omaha_hilo"));
    assert!(names.contains(&"five_card_draw"));
    assert!(names.contains(&"seven_card_stud"));
}

// ---------------------------------------------------------------------------
// Game CRUD
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_create_game() {
    let app = app();
    let (status, body) = post_json(
        &app,
        "/api/games",
        serde_json::json!({ "variant": "texas_holdem" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["id"].is_string());
    assert_eq!(body["variant"], "texas_holdem");
    assert_eq!(body["phase"], "Waiting for players");
}

#[tokio::test]
async fn test_create_game_default_variant() {
    let app = app();
    let (status, body) = post_empty(&app, "/api/games").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["variant"], "texas_holdem");
}

#[tokio::test]
async fn test_create_game_invalid_variant() {
    let app = app();
    let (status, body) = post_json(
        &app,
        "/api/games",
        serde_json::json!({ "variant": "go_fish" }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "invalid_action");
}

#[tokio::test]
async fn test_list_games_empty() {
    let app = app();
    let (status, body) = get(&app, "/api/games").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_get_game() {
    let app = app();
    let (_, created) = post_empty(&app, "/api/games").await;
    let id = created["id"].as_str().unwrap();

    let (status, body) = get(&app, &format!("/api/games/{id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["id"], id);
}

#[tokio::test]
async fn test_get_nonexistent_game() {
    let app = app();
    let (status, body) = get(&app, "/api/games/nonexistent-id").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "game_not_found");
}

#[tokio::test]
async fn test_delete_game() {
    let app = app();
    let (_, created) = post_empty(&app, "/api/games").await;
    let id = created["id"].as_str().unwrap();

    let (status, body) = delete(&app, &format!("/api/games/{id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["deleted"], id);

    let (status, _) = get(&app, &format!("/api/games/{id}")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_nonexistent_game() {
    let app = app();
    let (status, _) = delete(&app, "/api/games/no-such-game").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// Join / Leave
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_join_game() {
    let app = app();
    let (_, created) = post_empty(&app, "/api/games").await;
    let id = created["id"].as_str().unwrap();

    let (status, body) = post_json(
        &app,
        &format!("/api/games/{id}/join"),
        serde_json::json!({ "name": "Alice", "chips": 1000 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["players"].as_array().unwrap().len(), 1);
    assert_eq!(body["players"][0]["name"], "Alice");
}

#[tokio::test]
async fn test_join_game_phase_transitions() {
    let app = app();
    let (_, created) = post_empty(&app, "/api/games").await;
    let id = created["id"].as_str().unwrap();

    // One player: still waiting.
    let (_, body) = post_json(
        &app,
        &format!("/api/games/{id}/join"),
        serde_json::json!({ "name": "Alice" }),
    )
    .await;
    assert_eq!(body["phase"], "Waiting for players");

    // Two players: ready.
    let (_, body) = post_json(
        &app,
        &format!("/api/games/{id}/join"),
        serde_json::json!({ "name": "Bob" }),
    )
    .await;
    assert_eq!(body["phase"], "Ready to deal");
}

// ---------------------------------------------------------------------------
// Start Hand
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_start_hand() {
    let app = app();
    let (_, created) = post_empty(&app, "/api/games").await;
    let id = created["id"].as_str().unwrap();

    // Join two players.
    post_json(
        &app,
        &format!("/api/games/{id}/join"),
        serde_json::json!({ "name": "Alice", "chips": 1000 }),
    )
    .await;
    post_json(
        &app,
        &format!("/api/games/{id}/join"),
        serde_json::json!({ "name": "Bob", "chips": 1000 }),
    )
    .await;

    // Start the hand.
    let (status, body) = post_empty(&app, &format!("/api/games/{id}/start")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["phase"], "Pre-Flop");
    assert_eq!(body["hand_number"], 1);
    assert!(body["current_player_id"].is_string());
}

#[tokio::test]
async fn test_start_hand_not_enough_players() {
    let app = app();
    let (_, created) = post_empty(&app, "/api/games").await;
    let id = created["id"].as_str().unwrap();

    post_json(
        &app,
        &format!("/api/games/{id}/join"),
        serde_json::json!({ "name": "Alice" }),
    )
    .await;

    let (status, _) = post_empty(&app, &format!("/api/games/{id}/start")).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ---------------------------------------------------------------------------
// Player Actions
// ---------------------------------------------------------------------------

/// Helper: create a game with 2 players and start it.
async fn setup_started_game(app: &axum::Router) -> (String, String, String) {
    let (_, created) = post_empty(app, "/api/games").await;
    let id = created["id"].as_str().unwrap().to_string();

    let (_, j1) = post_json(
        app,
        &format!("/api/games/{id}/join"),
        serde_json::json!({ "name": "Alice", "chips": 1000 }),
    )
    .await;
    let p1 = j1["players"][0]["id"].as_str().unwrap().to_string();

    let (_, j2) = post_json(
        app,
        &format!("/api/games/{id}/join"),
        serde_json::json!({ "name": "Bob", "chips": 1000 }),
    )
    .await;
    let p2 = j2["players"][1]["id"].as_str().unwrap().to_string();

    post_empty(app, &format!("/api/games/{id}/start")).await;

    (id, p1, p2)
}

#[tokio::test]
async fn test_fold_action() {
    let app = app();
    let (game_id, _, _) = setup_started_game(&app).await;

    // Get current player.
    let (_, game) = get(&app, &format!("/api/games/{game_id}")).await;
    let current = game["current_player_id"].as_str().unwrap();

    let (status, body) = post_json(
        &app,
        &format!("/api/games/{game_id}/action"),
        serde_json::json!({ "player_id": current, "action": "fold" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["phase"], "Hand Complete");
}

#[tokio::test]
async fn test_call_and_check() {
    let app = app();
    let (game_id, _, _) = setup_started_game(&app).await;

    // First player calls.
    let (_, game) = get(&app, &format!("/api/games/{game_id}")).await;
    let current = game["current_player_id"].as_str().unwrap();

    let (status, _) = post_json(
        &app,
        &format!("/api/games/{game_id}/action"),
        serde_json::json!({ "player_id": current, "action": "call" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Second player checks.
    let (_, game) = get(&app, &format!("/api/games/{game_id}")).await;
    let current = game["current_player_id"].as_str().unwrap();

    let (status, body) = post_json(
        &app,
        &format!("/api/games/{game_id}/action"),
        serde_json::json!({ "player_id": current, "action": "check" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["phase"], "Flop");
    assert_eq!(body["community_cards"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_not_your_turn() {
    let app = app();
    let (game_id, _, _) = setup_started_game(&app).await;

    // Get the player who is NOT current.
    let (_, game) = get(&app, &format!("/api/games/{game_id}")).await;
    let current = game["current_player_id"].as_str().unwrap();
    let other = game["players"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"].as_str().unwrap() != current)
        .unwrap();
    let other_id = other["id"].as_str().unwrap();

    let (status, body) = post_json(
        &app,
        &format!("/api/games/{game_id}/action"),
        serde_json::json!({ "player_id": other_id, "action": "fold" }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "invalid_action");
}

// ---------------------------------------------------------------------------
// Player View
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_player_view() {
    let app = app();
    let (game_id, p1, _) = setup_started_game(&app).await;

    let (status, body) = get(&app, &format!("/api/games/{game_id}/player/{p1}")).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["your_hole_cards"].is_array());
    assert_eq!(body["your_hole_cards"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_player_view_nonexistent() {
    let app = app();
    let (game_id, _, _) = setup_started_game(&app).await;

    let (status, body) = get(&app, &format!("/api/games/{game_id}/player/fake-id")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "player_not_found");
}

// ---------------------------------------------------------------------------
// History
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_history_empty() {
    let app = app();
    let (game_id, _, _) = setup_started_game(&app).await;

    let (status, body) = get(&app, &format!("/api/games/{game_id}/history")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["hand_number"], 1);
    // No actions yet (just dealt).
    assert!(body["actions"].is_array());
}

#[tokio::test]
async fn test_history_after_action() {
    let app = app();
    let (game_id, _, _) = setup_started_game(&app).await;

    let (_, game) = get(&app, &format!("/api/games/{game_id}")).await;
    let current = game["current_player_id"].as_str().unwrap();

    post_json(
        &app,
        &format!("/api/games/{game_id}/action"),
        serde_json::json!({ "player_id": current, "action": "fold" }),
    )
    .await;

    let (status, body) = get(&app, &format!("/api/games/{game_id}/history")).await;
    assert_eq!(status, StatusCode::OK);
    let actions = body["actions"].as_array().unwrap();
    assert!(!actions.is_empty());
    assert_eq!(actions[0]["action"], "Fold");
}

// ---------------------------------------------------------------------------
// Omaha Variant
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_create_omaha_game() {
    let app = app();
    let (status, body) = post_json(
        &app,
        "/api/games",
        serde_json::json!({ "variant": "omaha" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["variant"], "omaha");
}
