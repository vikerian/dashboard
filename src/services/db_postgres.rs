use sqlx::PgPool;
use crate::models::{MojeData,SenzorData}; // Náš datový struct

/// Naše "fasáda" pro komunikaci s Postgres.
///
/// Načte všechny položky z ukázkové tabulky 'moje_data'.
/// Vrací 'Result', protože dotaz může selhat.
pub async fn get_data_from_postgres(
    pool: &PgPool
) -> Result<Vec<MojeData>, sqlx::Error> {
    
    // Použijeme makro 'query_as!', které je typově bezpečné
    // a automaticky mapuje výsledek na 'struct MojeData'
    // (díky 'derive(FromRow)').
    let data = sqlx::query_as!(
        MojeData,
        "SELECT id, nazev, hodnota FROM moje_data ORDER BY id"
    )
    .fetch_all(pool) // Spustí dotaz a načte všechny výsledky
    .await?; // '?' operátor vrátí chybu, pokud dotaz selže

    Ok(data) // Vrátí vektor dat
}
pub async fn get_timeseries_from_postgres(
    pool: &PgPool
) -> Result<Vec<SenzorData>, sqlx::Error> {
    
    // Použijeme 'query_as!' pro mapování na náš nový struct
    // Díky 'chrono' feature v sqlx si poradí s 'TIMESTAMPTZ'
    let data = sqlx::query_as!(
        SenzorData,
        // SQL dotaz je prostý SELECT, TimescaleDB se postará o rychlost
        r#"
        SELECT "time", senzor_id, hodnota 
        FROM senzor_data
        ORDER BY "time" DESC
        LIMIT 100
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(data)
}