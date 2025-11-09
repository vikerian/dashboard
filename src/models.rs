use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

/// Toto je náš "View Model" pro hlavní stránku.
///
/// Pomocí makra 'Template' říkáme Askamě,
/// aby pro tuto structu použila soubor 'index.html'.
#[derive(Serialize)] // <-- Tera potřebuje 'Serialize'
pub struct IndexTemplate {
    pub app_name: String,
}

/// Datová struktura mapovaná 1:1 na tabulku v Postgres.
/// 'FromRow' umožní sqlx automaticky mapovat řádek na tento struct.
#[derive(Debug, FromRow, Serialize)]
pub struct MojeData {
    pub id: i32,
    pub nazev: String,
    pub hodnota: f64,
}

/// View Model pro 'postgres.html' šablonu
#[derive(Serialize)]
pub struct PostgresTemplate {
    pub polozky: Vec<MojeData>,
}

/// View Model pro 'valkey.html' šablonu
#[derive(Serialize)]
pub struct ValkeyTemplate {
    pub key: String,
    pub value: String, // Místo 'info_string'
}
/// Datová struktura pro řádek z TimescaleDB
#[derive(Debug, FromRow, Serialize)]
pub struct SenzorData {
    // 'TIMESTAMPTZ' se mapuje na 'DateTime<Utc>' z knihovny 'chrono'
    #[serde(rename = "time")] // Přejmenujeme pro JSON/Tera
    pub time: DateTime<Utc>,
    pub senzor_id: String,
    pub hodnota: f64,
}

/// View Model pro 'timescale.html' šablonu
#[derive(Serialize)]
pub struct TimescaleTemplate {
    pub senzory: Vec<SenzorData>,
}

/// Struktura pro uložení nasbíraných $SYS statistik
/// 'Default' nám umožní vytvořit prázdnou instanci.
#[derive(Debug, Default, Serialize, Clone)]
pub struct MqttStats {
    pub uptime: String,
    pub clients_connected: String,
    pub messages_sent: String,
    pub messages_received: String,
}

/// View Model pro 'mqtt.html' šablonu
#[derive(Serialize)]
pub struct MqttTemplate {
    pub broker_host: String,
    // Smažeme: pub stats: MqttStats,
    pub refresh_interval_ms: u64, // Přidáme interval
}

/// Reprezentuje jeden 'hit' (záznam) ve výsledcích hledání
#[derive(Debug, Deserialize, Serialize)]
pub struct ManticoreHit {
    // Manticore vrací data v '_source' poli
    pub _source: serde_json::Value, // Použijeme generickou JSON hodnotu
}

/// Reprezentuje obálku 'hits', kterou Manticore posílá
#[derive(Debug, Deserialize, Serialize)]
pub struct ManticoreHits {
    pub total: u32,
    pub hits: Vec<ManticoreHit>,
}

/// Reprezentuje celou odpověď z Manticore Search
#[derive(Debug, Deserialize, Serialize)]
pub struct ManticoreResponse {
    pub hits: ManticoreHits,
}

/// View Model pro 'search.html' šablonu
#[derive(Serialize)]
pub struct SearchTemplate {
    pub query: String, // Co uživatel hledal
    // ZMĚNA: Použijeme i64.
    // -1 bude znamenat "nevyhledáno"
    // -2 bude znamenat "chyba"
    // >= 0 bude znamenat počet výsledků
    pub total_hits: i64,
    pub results: Vec<serde_json::Value>, // Jen 'rozbalená' data
}