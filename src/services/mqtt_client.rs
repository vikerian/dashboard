use crate::models::MqttStats;
use crate::state::MqttConfig;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Spustí trvalého MQTT klienta na pozadí (v 'tokio::spawn').
///
/// Tento klient naslouchá $SYS tématům a aktualizuje
/// sdílený 'stats_store'.
pub fn launch_mqtt_subscriber(
    config: MqttConfig,
    stats_store: Arc<RwLock<MqttStats>>
) {
    // Spustíme úplně novou, nezávislou úlohu na pozadí
    tokio::spawn(async move {
        // Nastavíme klienta
        let mut mqtt_options = MqttOptions::new("", config.host, config.port);
        mqtt_options
            .set_keep_alive(Duration::from_secs(5))
            .set_clean_session(true);

        let (client, mut eventloop) = AsyncClient::new(mqtt_options, 10);

        // Přihlásíme se k odběru
        // Používáme '.unwrap()' - pokud se sběrač nepřihlásí,
        // je to fatální chyba a má spadnout (a my to uvidíme v logu).
        client.subscribe("$SYS/broker/uptime", QoS::AtMostOnce).await.unwrap();
        client.subscribe("$SYS/broker/clients/connected", QoS::AtMostOnce).await.unwrap();
        client.subscribe("$SYS/broker/messages/sent", QoS::AtMostOnce).await.unwrap();
        client.subscribe("$SYS/broker/messages/received", QoS::AtMostOnce).await.unwrap();

        tracing::info!("MQTT Sběrač: Úspěšně připojen a přihlášen k odběru $SYS.");

        // Toto je smyčka, která poběží NAVŽDY
        // 'eventloop.poll()' se stará o živé připojení a příjem zpráv
        while let Ok(notification) = eventloop.poll().await {
            // Zpracujeme jen příchozí zprávy
            if let rumqttc::Event::Incoming(rumqttc::Packet::Publish(packet)) = notification {
                let topic = packet.topic;
                let payload = String::from_utf8(packet.payload.to_vec())
                                .unwrap_or_else(|_| "N/A".to_string());

                // Získáme zámek pro zápis do našeho sdíleného úložiště
                let mut stats = stats_store.write().await;

                // Aktualizujeme data
                match topic.as_str() {
                    "$SYS/broker/uptime" => stats.uptime = payload,
                    "$SYS/broker/clients/connected" => stats.clients_connected = payload,
                    "$SYS/broker/messages/sent" => stats.messages_sent = payload,
                    "$SYS/broker/messages/received" => stats.messages_received = payload,
                    _ => {} // Ignorujeme ostatní
                }
                // Zámek se zde automaticky uvolní
            }
        }
        
        tracing::warn!("MQTT Sběrač: Smyčka přerušena. Připojení ztraceno.");
    });
}