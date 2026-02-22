use rumenx_poker::api::router::create_router;
use rumenx_poker::api::state::AppState;
use rumenx_poker::config::AppConfig;

#[tokio::main]
async fn main() {
    // Handle --health-check flag for Docker HEALTHCHECK (works in scratch image).
    if std::env::args().any(|a| a == "--health-check") {
        match health_check().await {
            Ok(()) => std::process::exit(0),
            Err(e) => {
                eprintln!("Health check failed: {e}");
                std::process::exit(1);
            }
        }
    }

    // Initialize tracing (structured logging).
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rumenx_poker=info,tower_http=info".into()),
        )
        .init();

    let config = AppConfig::from_env();
    let bind_addr = config.bind_addr();
    let state = AppState::new();

    let app = create_router(state);

    tracing::info!(
        "rust-poker v{} starting on {bind_addr}",
        env!("CARGO_PKG_VERSION")
    );

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .expect("Failed to bind to address");

    tracing::info!("Listening on {bind_addr}");

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}

/// Perform a health check against the running server (for Docker HEALTHCHECK).
async fn health_check() -> Result<(), Box<dyn std::error::Error>> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8084".to_string());

    let stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{port}")).await?;
    let (mut reader, mut writer) = tokio::io::split(stream);

    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let request =
        format!("GET /health HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n");
    writer.write_all(request.as_bytes()).await?;

    let mut response = Vec::new();
    reader.read_to_end(&mut response).await?;

    let body = String::from_utf8_lossy(&response);
    if body.contains("\"ok\"") {
        Ok(())
    } else {
        Err(format!("unexpected response: {body}").into())
    }
}
