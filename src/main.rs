use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tracing::info;

use finance_app::{
    config::AppConfig,
    create_app_instance,
    database::postgres::{create_postgres_pool, run_migrations},
    pkg::{logging::init_logger, realtime::RealtimeHub},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::from_env();

    // 1. Initialize OpenTelemetry Tracer if enabled
    let (otel_provider, otel_err) = match finance_app::pkg::telemetry::init_telemetry(&config) {
        Ok(provider) => (provider, None),
        Err(err) => (None, Some(err)),
    };

    // 2. Initialize structured JSON logging
    init_logger(&config, otel_provider.as_ref());

    if let Some(err) = otel_err {
        tracing::warn!(error = %err, "Failed to initialize OpenTelemetry tracer provider");
    }

    info!(
        host = %config.host,
        http_port = config.port,
        grpc_port = config.grpc_port,
        "Starting Coffe Finance, Procurement (PO), and Expense Microservice..."
    );

    // 3. Connect to PostgreSQL
    let pool = match create_postgres_pool(&config.database_url).await {
        Ok(pool) => {
            info!("Database connection pool established successfully");
            if let Err(e) = run_migrations(&pool).await {
                tracing::warn!(error = %e, "Migration check failed, continuing with existing schema");
            } else {
                info!("Database migrations executed successfully");
            }
            pool
        }
        Err(err) => {
            tracing::error!(
                error = %err,
                database_url = %config.database_url,
                "Failed to connect to database. Please ensure PostgreSQL is running."
            );
            return Err(Box::new(err) as Box<dyn std::error::Error>);
        }
    };

    // 4. Initialize Realtime Hub with Distributed Bridge
    let hub = Arc::new(RealtimeHub::default());
    let bridge = finance_app::pkg::realtime::create_realtime_bridge(&config, hub.sender()).await;
    hub.set_bridge(bridge).await;

    // 5. Build Application Instance (Router + Domain Services)
    let app_instance = create_app_instance(config.clone(), pool, hub);

    // 6. Start Axum HTTP REST listener with graceful shutdown
    let http_addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let listener = TcpListener::bind(http_addr).await?;
    info!("HTTP REST server listening on http://{}", http_addr);

    axum::serve(listener, app_instance.router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    info!("Server shutdown completed cleanly");
    finance_app::pkg::telemetry::shutdown_telemetry();
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C signal, shutting down...");
        },
        _ = terminate => {
            tracing::info!("Received terminate signal, shutting down...");
        },
    }
}
