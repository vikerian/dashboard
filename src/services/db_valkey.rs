use redis::{Client as ValkeyClient,RedisResult};


/// Načte hodnotu pro daný klíč z Valkey.
///
/// Vrací 'RedisResult<Option<String>>', protože klíč
/// nemusí existovat (GET vrátí 'nil', což 'redis-rs'
/// správně přeloží na 'Ok(None)', pokud cílový typ je 'Option').
pub async fn get_valkey_kv(
    client: &ValkeyClient,
    key: &str,
) -> RedisResult<Option<String>> {
    
    // Získáme multiplexované připojení
    let mut con = client.get_multiplexed_async_connection().await?;

    // Spustíme příkaz GET a explicitně řekneme, že
    // očekáváme 'Option<String>'.
    let value: Option<String> = redis::cmd("GET")
        .arg(key) // Přidáme argument (náš klíč)
        .query_async(&mut con)
        .await?;
    
    Ok(value)
}