/// Provede fulltextové vyhledávání v Manticore
use crate::models::ManticoreResponse;
use reqwest::Client as ManticoreClient;
use serde_json::json;

// PŘIDÁNO: Potřebujeme serde_json pro ruční parsování
use serde_json::from_str as parse_json_from_string;
//use std::env;
use std::error::Error;

/// Provede fulltextové vyhledávání v Manticore
pub async fn search_manticore(
    client: &ManticoreClient,
    base_url: &str,
    index_name: &str,
    query: &str,
) -> Result<ManticoreResponse, Box<dyn Error>> {
    
    let search_url = format!("{}/search", base_url);

    let manticore_query = json!({
        "index": index_name,
        "query": { "match": { "_all": query } },
    });

    tracing::debug!("Odesílám Manticore dotaz: {}", manticore_query);

    // ---- ZMĚNA ZDE: Ruční zpracování odpovědi ----
    
    // 1. Odešleme dotaz a získáme surovou odpověď
    let response = client
        .post(&search_url)
        .json(&manticore_query)
        .send()
        .await?;

    // 2. Zjistíme HTTP status a získáme text
    let status = response.status();
    let response_text = response.text().await?;

    // 3. ZALOGUJEME SUROVÝ TEXT ODPOVĚDI (TOHLE JE KLÍČOVÉ)
    tracing::debug!(
        "Obdržená RAW odpověď z Manticore (Status: {}): {}",
        status,
        response_text
    );

    // 4. Zkontrolujeme, jestli Manticore nevrátil HTTP chybu
    if !status.is_success() {
        // Vrátíme chybu s textem, který poslal Manticore
        let err_msg = format!("Manticore vrátil chybu (Status {}): {}", status, response_text);
        // Převedeme na 'reqwest::Error'
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            err_msg,
        ).into());
    }

   // 5. Status byl OK, teď se pokusíme text parsovat na náš struct
    let manticore_response: ManticoreResponse = parse_json_from_string(&response_text)
        .map_err(|e| { // 'e' je typu serde_json::Error
            tracing::error!("Selhalo parsování JSON z Manticore: {}. Odpověď byla: {}", e, response_text);
            
            // STARÁ VERZE:
            // Box::from(e) // Toto je nejednoznačné

            // NOVÁ, EXPLICITNÍ VERZE:
            // 1. Vytvoříme 'Box<serde_json::Error>'
            // 2. Pomocí 'as' ho převedeme na 'Box<dyn Error>'
            Box::new(e) as Box<dyn Error>
        })?; // Tento '?' teď bude spokojený
    Ok(manticore_response)
    // ---- KONEC ZMĚNY ----
}