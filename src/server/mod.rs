use crate::capabilities::Capabilities;
use crate::config::Config;

mod handlers;

/// Starts the Axum web server on the specified port.
pub async fn run(port: u16, config: Config) -> anyhow::Result<()> {
    let capabilities = Capabilities::discover(&config);
    println!("Capabilities: {:?}", capabilities);
    println!("Server running on port {}", port);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, app(config)).await?;

    Ok(())
}

fn app(config: Config) -> axum::Router {
    use axum::routing::get;

    axum::Router::new()
        .route("/", get(handlers::root))
        .route("/{type}/{format}/{source}", get(handlers::get_diagram))
        .with_state(config)
}
