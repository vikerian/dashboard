// Zjednodušený import
use axum::{routing::get, Router};

//use crate::state::{AppState, AppStateBuilder};
use crate::state::AppStateBuilder;
use std::env;
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tracing::info;

// Ostatní moduly (main.rs nemusí znát `Json`)
mod logging;
mod models;
mod routes;
mod services;
mod state;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let _logging_guard = logging::setup_logging();

    let db_url = env::var("DATABASE_URL").expect("Proměnná DATABASE_URL není nastavena");
    let valkey_url = env::var("VALKEY_URL").expect("Proměnná VALKEY_URL není nastavena");
    let manticore_url = env::var("MANTICORE_URL").expect("Proměnná MANTICORE_URL není nastavena");

    info!("Sestavuji AppState a připojuji k DB...");
    let app_state = AppStateBuilder::new()
        .app_name("RPi Dashboard")
        .postgres_url(db_url)
        .valkey_url(valkey_url)
        .manticore_url(manticore_url)
        .build()
        .await;

    info!("AppState sestaven.");

    // Vytvoříme router pro statické soubory.
    // Tento router nepotřebuje 'AppState'.
    let static_router = Router::new()
        // Všechny požadavky na '/static/...' budou
        // obslouženy z adresáře 'static/'
        .nest_service("/static", ServeDir::new("static"));

    // ---- OPRAVENÁ DEFINICE ROUTERU ----
    // Sestavíme jeden finální router 'app'
    let app_with_state = Router::new()
        // Přidáme všechny webové routy
        .route("/", get(routes::web::page_index))
        .route("/postgres", get(routes::web::page_postgres))
        .route("/valkey", get(routes::web::page_valkey))
        .route("/timescale", get(routes::web::page_timescale))
        .route("/mqtt", get(routes::web::page_mqtt))
        .route("/search", get(routes::web::page_search))
        // Vnoříme všechny API routy pod prefix /api/v1
        .nest("/api/v1", 
            Router::new()
                .route("/mqtt/stats", get(routes::api::get_mqtt_stats))
            // Zde by přišly další API routy...
        )
        // AŽ TEĎ, úplně na konci, aplikujeme stav na VŠECHNY routy
        .with_state(app_state);
    // ---- KONEC OPRAVY ----

    let app = Router::new()
        .merge(app_with_state)
        .merge(static_router);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("Server bude naslouchat na adrese http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap_or_else(|e| {
        tracing::error!("Nepodařilo se bindovat na port 8080: {}", e);
        std::process::exit(1);
    });

    // Spustíme náš finální 'app'
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}