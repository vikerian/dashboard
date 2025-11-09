use axum::response::{Html, IntoResponse, Response};
use axum::extract::{Query,State};
use crate::models::{SearchTemplate};
use crate::state::AppState;       // Náš sdílený stav
use tera::Context;               // <-- Potřebujeme Context pro Tera
use crate::models::{IndexTemplate, PostgresTemplate, ValkeyTemplate,TimescaleTemplate,MqttTemplate}; 
use crate::services::{db_postgres, db_valkey,search_manticore};
use crate::state::MqttConfig;
use serde::Deserialize;

/// Handler pro hlavní stránku, nyní s 'Tera'
pub async fn page_index(
    State(state): State<AppState> // <-- DI funguje stejně
) -> Response { // <-- Vracíme obecnou 'Response' pro lepší chybovost
    
    tracing::info!("Obsloužen požadavek na / (page_index)");

    // 1. Připravíme data pro šablonu
    let data = IndexTemplate {
        app_name: state.app_name.clone(),
    };

    // 2. Vytvoříme 'tera::Context' z našich dat
    // Museli jsme pro 'IndexTemplate' přidat '#[derive(Serialize)]'
    let context = match Context::from_serialize(&data) {
        Ok(ctx) => ctx,
        Err(e) => {
            tracing::error!("Chyba při serializaci kontextu: {}", e);
            // Vrátíme interní chybu serveru
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Chyba serveru při přípravě dat"
            ).into_response();
        }
    };

    // 3. Renderujeme šablonu pomocí 'tera' z našeho 'AppState'
    match state.tera.render("index.html", &context) {
        Ok(html_string) => {
            // Pokud se podaří, vrátíme HTML
            Html(html_string).into_response()
        }
        Err(e) => {
            tracing::error!("Chyba při renderování šablony 'index.html': {}", e);
            // Vrátíme interní chybu serveru
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Chyba serveru při renderování šablony: {}", e)
            ).into_response()
        }
    }
}

/// Handler pro stránku /postgres
pub async fn page_postgres(
    State(state): State<AppState> // Injektujeme stav
) -> Response {
    tracing::info!("Obsloužen požadavek na /postgres");

    // 1. Zavoláme naši Servisní vrstvu (Model)
    let data_result = db_postgres::get_data_from_postgres(&state.postgres_pool).await;

    // 2. Zpracujeme výsledek (data nebo chyba)
    let data = match data_result {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("Chyba při dotazu do Postgres: {}", e);
            // V případě chyby DB vrátíme chybu serveru
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Chyba při čtení z databáze: {}", e)
            ).into_response();
        }
    };

    // 3. Připravíme View Model
    let template_data = PostgresTemplate {
        polozky: data
    };

    // 4. Vytvoříme 'tera::Context'
    let context = match Context::from_serialize(&template_data) {
        Ok(ctx) => ctx,
        Err(e) => {
            tracing::error!("Chyba při serializaci Postgres kontextu: {}", e);
            return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Chyba serveru").into_response();
        }
    };

    // 5. Renderujeme šablonu
    match state.tera.render("postgres.html", &context) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!("Chyba při renderování šablony 'postgres.html': {}", e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Chyba serveru").into_response()
        }
    }
}
pub async fn page_valkey(
    State(state): State<AppState>
) -> Response {
    tracing::info!("Obsloužen požadavek na /valkey (K:V)");

    let key_to_fetch = "dashboard:status";

    // 1. Zavoláme novou servisní funkci
    let value_option = match db_valkey::get_valkey_kv(&state.valkey_client, key_to_fetch).await {
        Ok(value) => value, // 'value' je typu 'Option<String>'
        Err(e) => {
            tracing::error!("Neočekávaná chyba z Valkey service (GET): {}", e);
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Chyba při čtení z Valkey: {}", e)
            ).into_response();
        }
    };

    // 2. Zpracujeme 'Option'
    // Pokud klíč neexistuje, 'value_option' bude 'None'.
    let value_string = value_option
        .unwrap_or_else(|| "(Klíč nebyl v databázi nalezen)".to_string());

    // 3. Připravíme View Model
    let template_data = ValkeyTemplate {
        key: key_to_fetch.to_string(),
        value: value_string
    };

    // 4. Vytvoříme 'tera::Context'
    let context = match Context::from_serialize(&template_data) {
        Ok(ctx) => ctx,
        Err(e) => {
            tracing::error!("Chyba při serializaci Valkey (K:V) kontextu: {}", e);
            return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Chyba serveru").into_response();
        }
    };

    // 5. Renderujeme šablonu
    match state.tera.render("valkey.html", &context) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!("Chyba při renderování šablony 'valkey.html': {}", e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Chyba serveru").into_response()
        }
    }
}


pub async fn page_timescale(
    State(state): State<AppState>
) -> Response {
    tracing::info!("Obsloužen požadavek na /timescale");

    // 1. Zavoláme novou servisní funkci
    let senzory = match db_postgres::get_timeseries_from_postgres(&state.postgres_pool).await {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("Chyba při dotazu do TimescaleDB: {}", e);
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Chyba při čtení z databáze: {}", e)
            ).into_response();
        }
    };

    // 2. Připravíme View Model
    let template_data = TimescaleTemplate {
        senzory: senzory
    };

    // 3. Vytvoříme 'tera::Context'
    let context = match Context::from_serialize(&template_data) {
        Ok(ctx) => ctx,
        Err(e) => {
            tracing::error!("Chyba při serializaci Timescale kontextu: {}", e);
            return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Chyba serveru").into_response();
        }
    };

    // 4. Renderujeme šablonu
    match state.tera.render("timescale.html", &context) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!("Chyba při renderování šablony 'timescale.html': {}", e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Chyba serveru").into_response()
        }
    }
}

pub async fn page_mqtt(
    // Bereme si 'MqttConfig' (kvůli 'host' a 'interval')
    State(mqtt_config): State<MqttConfig>,
    // Bereme si 'AppState' (kvůli 'tera' enginu)
    State(state): State<AppState>
) -> Response {
    tracing::info!("Obsloužen požadavek na /mqtt (HTML kostra)");

    // 1. Vytvoříme View Model POUZE s konfigurací.
    //    Data si načte JavaScript sám.
    let template_data = MqttTemplate {
        broker_host: mqtt_config.host,
        refresh_interval_ms: mqtt_config.refresh_interval_ms,
    };

    // 2. Vytvoříme 'tera::Context'
    let context = match Context::from_serialize(&template_data) {
        Ok(ctx) => ctx,
        Err(e) => {
            tracing::error!("Chyba při serializaci MQTT kontextu: {}", e);
            return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Chyba serveru").into_response();
        }
    };

    // 3. Renderujeme šablonu
    match state.tera.render("mqtt.html", &context) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!("Chyba při renderování šablony 'mqtt.html': {}", e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Chyba serveru").into_response()
        }
    }
}

// ---- PŘIDÁNO PRO SEARCH ----

/// Struct pro parsování URL query parametrů (např. ?q=test)
#[derive(Debug, Deserialize)]
pub struct SearchParams {
    q: Option<String>, // 'q' je název našeho <input> pole
}

/// Handler pro stránku /search

pub async fn page_search(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Response {
    tracing::info!("Obsloužen požadavek na /search (query: {:?})", params.q);

    // 1. Připravíme výchozí data
    let mut template_data = SearchTemplate {
        query: params.q.clone().unwrap_or_default(),
        total_hits: -1, // <-- ZMĚNA: -1 znamená "nevyhledáno"
        results: vec![],
    };

    // 2. Pokud uživatel něco zadal do ?q=...
    if let Some(query) = params.q {
        if !query.is_empty() {
            match search_manticore::search_manticore(
                &state.manticore_client,
                &state.manticore_base_url,
                "dashboard_index",
                &query
            ).await {
                Ok(response) => {
                    // Úspěch - převedeme u32 na i64
                    template_data.total_hits = response.hits.total as i64; // <-- ZMĚNA
                    template_data.results = response.hits.hits
                        .into_iter()
                        .map(|hit| hit._source)
                        .collect();
                },
                Err(e) => {
                    tracing::error!("Chyba při dotazu do Manticore: {}", e);
                    template_data.total_hits = -2; // <-- ZMĚNA: -2 znamená chyba
                }
            }
        }
    }

    // 3. Vytvoříme 'tera::Context'
    let context = match Context::from_serialize(&template_data) {
        Ok(ctx) => ctx,
        Err(e) => {
            tracing::error!("Chyba při serializaci Search kontextu: {}", e);
            return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Chyba serveru").into_response();
        }
    };

    // 4. Renderujeme šablonu
    match state.tera.render("search.html", &context) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!("Chyba při renderování šablon 'search.html': {}", e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Chyba serveru").into_response()
        }
    }
}