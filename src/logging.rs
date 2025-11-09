use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
/// , Layer};

/// Nastaví globální 'tracing' subscriber pro celou aplikaci.
///
/// Loguje na 'stdout' (konzole) a zároveň do rotujícího souboru v adresáři 'logs/'.
///
/// Vrací 'WorkerGuard', který musí být držen v 'main' funkci,
/// aby se zajistilo, že všechny logy stihnou být zapsány do souboru
/// i při ukončení aplikace.
pub fn setup_logging() -> WorkerGuard {
    
    // 1. Filtr logů z proměnné prostředí RUST_LOG
    // Příklad: RUST_LOG="info,dashboard_project=debug"
    // Pokud proměnná není nastavena, použije se default "info".
    let filter_layer = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // 2. Vrstva pro logování do konzole (stdout)
    // 'fmt::layer()' vytvoří formátovací vrstvu.
    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout) // Píšeme na standardní výstup
        .with_ansi(true);             // Chceme barevný výstup v terminálu

    // 3. Vrstva pro logování do souboru
    // 'rolling::daily' vytvoří soubor, který se bude rotovat každý den.
    // Soubory budou v adresáři 'logs' a budou mít název 'dashboard.log.YYYY-MM-DD'.
    let file_appender = rolling::daily("logs", "dashboard.log");

    // 'tracing_appender::non_blocking' je důležitý pro výkon.
    // Zápis do souboru je pomalá operace. Toto vytvoří 'worker' thread,
    // který se stará o zápis, a naše hlavní aplikační thready mu
    // jen rychle posílají zprávy.
    let (non_blocking_file_writer, file_guard) =
        tracing_appender::non_blocking(file_appender);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking_file_writer) // Píšeme do neblokujícího bufferu
        .with_ansi(false); // Do souboru nechceme ANSI escape kódy (barvičky)

    // 4. Kombinace vrstev a inicializace
    // 'tracing_subscriber::registry()' je základ, na který "vrstvíme"
    // naše konfigurace.
    tracing_subscriber::registry()
        .with(filter_layer) // Vrstva č. 1: Filtrování (co logovat)
        .with(stdout_layer) // Vrstva č. 2: Výstup do konzole
        .with(file_layer) // Vrstva č. 3: Výstup do souboru
        .init(); // Nastaví tento subscriber jako globální pro celou aplikaci

    // 5. Vrátíme 'guard'
    // 'main' funkce si ho musí uložit do proměnné, jinak by byl 'guard'
    // okamžitě zahozen, 'worker' thread by se ukončil a logy
    // do souboru by se nezapisovaly.
    file_guard
}