use axum::{extract::State, response::Json, response::IntoResponse};
use crate::state::AppState; // <-- Změna

/// API endpoint, který vrací aktuální MQTT statistiky jako JSON
///
/// Tato funkce je super rychlá - jen čte z paměti,
/// kterou na pozadí plní 'launch_mqtt_subscriber'.
pub async fn get_mqtt_stats(
    // Bereme si celý AppState, abychom se dostali k 'mqtt_stats'
    State(app_state): State<AppState>
) -> impl IntoResponse {
    
    // Získáme zámek pro čtení
    let stats_lock = app_state.mqtt_stats.read().await;
    
    // Klonujeme data (aby zámek mohl být hned uvolněn)
    let stats_data = stats_lock.clone();
    
    // Vrátíme JSON
    Json(stats_data).into_response()
}