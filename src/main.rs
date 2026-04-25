use agamenon::{
    cache::CachedSchemaStore,
    config::Config,
    routes::{
        check_compatibility, get_schema_by_id, list_subjects, list_versions, register_schema, AppState,
    },
    storage::PostgresSchemaStore,
};
use axum::{
    extract::Request,
    http::header::CONTENT_TYPE,
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tracing_subscriber;

async fn set_content_type_middleware(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        CONTENT_TYPE,
        "application/vnd.schemaregistry.v1+json"
            .parse()
            .unwrap(),
    );
    response
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let config = Config::from_env();

    println!("🔗 Connecting to database: {}", config.database_url);

    // Create connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    // Run migrations
    println!("📦 Running migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    println!("✅ Migrations completed");

    // Initialize storage
    let postgres_store = Arc::new(PostgresSchemaStore::new(pool));
    let cached_store = Arc::new(
        CachedSchemaStore::new(postgres_store, config.cache_max_capacity)
            .await,
    );

    let state = AppState {
        store: cached_store,
    };

    // Create router
    let app = Router::new()
        .route("/subjects", get(list_subjects))
        .route("/subjects/:name/versions", get(list_versions))
        .route("/subjects/:name/versions", post(register_schema))
        .route("/schemas/ids/:id", get(get_schema_by_id))
        .route(
            "/compatibility/subjects/:name/versions/:version",
            post(check_compatibility),
        )
        .layer(middleware::from_fn(set_content_type_middleware))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.server_host, config.server_port))
        .await
        .expect("Failed to bind to address");

    println!(
        "🚀 Agamenon listening on http://{}:{}",
        config.server_host, config.server_port
    );

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
