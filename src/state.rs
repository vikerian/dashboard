use axum::extract::FromRef;
use crate::models::MqttStats;
use redis::Client as ValkeyClient; // valkey db (fork redisu)
use reqwest::Client as ManticoreClient;
use std::env;
use std::sync::Arc;
use sqlx::{postgres::PgPoolOptions, PgPool}; // database postgresql
use tera::Tera; // sablony
use tokio::sync::RwLock;



#[derive(Clone)]
pub struct MqttConfig {
    pub host: String,
    pub port: u16,
    pub refresh_interval_ms: u64, // <-- PŘIDÁNO
}

#[derive(Clone)]
pub struct AppState {
    pub tera: Tera,
    pub app_name: String,
    pub postgres_pool: PgPool,
    pub valkey_client: ValkeyClient,
    pub mqtt_config: MqttConfig,
    // PŘIDÁNO: Sdílené, thread-safe úložiště pro MQTT data
    pub mqtt_stats: Arc<RwLock<MqttStats>>,
    pub manticore_client: ManticoreClient,
    pub manticore_base_url: String,
}

#[derive(Default)]
pub struct AppStateBuilder {
    app_name: Option<String>,
    postgres_url: Option<String>,
    valkey_url: Option<String>,
    manticore_url: Option<String>,
}

// ---- ZMĚNA ZDE ----
// Naučíme 'axum', jak "vytáhnout" 'MqttConfig' z 'AppState'.
// Toto je pokročilejší DI - říkáme, že 'MqttConfig'
// je podmnožinou 'AppState'.
impl FromRef<AppState> for MqttConfig {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.mqtt_config.clone()
    }
}

impl AppStateBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn app_name(mut self, name: &str) -> Self {
        self.app_name = Some(name.to_string());
        self
    }
    
    // Nová metoda pro builder
    pub fn postgres_url(mut self, url: String) -> Self {
        self.postgres_url = Some(url);
        self
    }

    // Nová metoda pro builder
    pub fn valkey_url(mut self, url: String) -> Self {
        self.valkey_url = Some(url);
        self
    }

    pub fn manticore_url(mut self, url: String) -> Self {
        self.manticore_url = Some(url);
        self
    }

    /// Finální metoda, která sestaví 'AppState'.
    /// PŘIDÁVÁME 'async', protože načítání šablon z disku
    /// může selhat a je to I/O operace.
    pub async fn build(self) -> AppState {
        let tera = Tera::new("templates/**/*.html")
            .expect("Nepodařilo se načíst Tera šablony");

        // ---- PŘIDÁNO: Připojení k Postgres ----
        let db_url = self.postgres_url
            .expect("DATABASE_URL musí být nastaven");
        
        let pg_pool = PgPoolOptions::new()
            .max_connections(5) // Nastavíme max 5 připojení v poolu
            .connect(&db_url)
            .await // Asynchronně se připojíme
            .expect("Nepodařilo se připojit k Postgres databázi");
        
        tracing::info!("Úspěšně připojeno k Postgres databázi.");
        // ---- KONEC PŘIDÁNÍ ----

        // ---- PŘIDÁNO: Připojení k Valkey ----
        let valkey_url = self.valkey_url
            .expect("VALKEY_URL musí být nastaven");
            
        let valkey_client = ValkeyClient::open(valkey_url)
            .expect("Nepodařilo se vytvořit Valkey/Redis klienta");
        
        // Otestujeme připojení (volitelný, ale dobrý krok)
         
        let mut con = valkey_client.get_multiplexed_async_connection().await            
            .expect("Nepodařilo se získat Valkey/Redis připojení");
        let _ : () = redis::cmd("PING").query_async(&mut con).await
            .expect("PING na Valkey/Redis selhal");
            
        tracing::info!("Úspěšně připojeno k Valkey/Redis.");
        // ---- KONEC PŘIDÁNÍ ----

      // --- MQTT Konfigurace ---
        let mqtt_host = env::var("MQTT_HOST")
            .expect("Proměnná MQTT_HOST není nastavena");
        let mqtt_port = env::var("MQTT_PORT")
            .expect("Proměnná MQTT_PORT není nastavena")
            .parse::<u16>()
            .expect("MQTT_PORT musí být číslo");
        let mqtt_refresh_ms = env::var("MQTT_REFRESH_INTERVAL_MS")
            .unwrap_or_else(|_| "30000".to_string())
            .parse::<u64>()
            .expect("MQTT_REFRESH_INTERVAL_MS musí být číslo");

        let mqtt_config = MqttConfig {
            host: mqtt_host,
            port: mqtt_port,
            refresh_interval_ms: mqtt_refresh_ms,
        };
        tracing::info!("MQTT konfigurace načtena.");

        // ---- PŘIDÁNO: Vytvoření úložiště a spuštění sběrače ----
        
        // 1. Vytvoříme prázdné úložiště dat
        let stats_store = Arc::new(RwLock::new(MqttStats::default()));

        // 2. Spustíme sběrač na pozadí.
        //    Předáme mu konfiguraci a "pointer" na úložiště.
        crate::services::mqtt_client::launch_mqtt_subscriber(
            mqtt_config.clone(), // Sběrač potřebuje konfiguraci
            stats_store.clone()  // Sběrač potřebuje úložiště
        );
        // ---- KONEC PŘIDÁNÍ ----

        // ---- PŘIDÁNO: Vytvoření Manticore klienta ----
        
        // Uložíme si URL do proměnné prostředí.
        // Je to jednodušší než ji předávat přes AppState,
        // protože ji 'service' funkce potřebuje jako String.
        let manticore_url = self.manticore_url
            .expect("MANTICORE_URL musí být nastaven");
        
        
        // Vytvoříme klienta. Je 'lehký' a sdílený.
        let manticore_client = ManticoreClient::new();
        tracing::info!("Manticore HTTP klient vytvořen.");

        AppState {
            tera: tera,
            app_name: self.app_name.unwrap_or_else(|| "Výchozí App".to_string()),
            postgres_pool: pg_pool, 
            valkey_client: valkey_client,
            mqtt_config: mqtt_config,
            mqtt_stats: stats_store,
            manticore_client: manticore_client,
            manticore_base_url: manticore_url,
        }
    }

}