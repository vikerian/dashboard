# RPi Dashboard v Rustu

Tento projekt je výuková aplikace pro demonstraci "objektově orientovaných" principů
a moderních postupů v jazyce Rust. Cílem je vytvořit dashboard pro monitoring
služeb běžících na Raspberry Pi v rámci k3s clusteru.

Aplikace běží jako webový server (backend) a poskytuje:
1.  Webové rozhraní (HTML) renderované na serveru.
2.  REST API (JSON) pro monitoring a data.

## Technologický Stack

* **Jazyk:** Rust
* **Webový Framework:** `axum` (postavený na `tokio` a `hyper`)
* **Šablony (Templating):** `askama` (kompilované Jinja2-like šablony)
* **Logování:** `tracing` (s výstupem do souboru, stdout a volitelně syslog)
* **Databáze:**
    * `Valkey` (pro časové řady ze senzorů)
    * `PostgreSQL` (pro relační data z externí microslužby)
* **Zprávy:** `Mosquitto` (MQTT broker)
* **Vyhledávání:** `Manticore Search`
* **Nasazení:** `Docker` kontejner v `k3s` (Kubernetes)

## Struktura Aplikace

Aplikace dodržuje upravený **MVC (Model-View-Controller)** pattern a principy **Servisní vrstvy**.

* `src/routes/`: **Controllers** (handlery `axum`, rozdělené na `web` a `api`)
* `src/templates/`: **Views** (HTML šablony `askama`)
* `src/models.rs`: **Models** (datové `struct`y)
* `src/services/`: **Service Layer** (abstrakce nad databázemi a externími API)
* `src/state.rs`: **Shared State** (sdílený stav (`AppState`) obsahující DB pooly, injektovaný pomocí Dependency Injection)
* `src/main.rs`: Hlavní vstupní bod, který spojuje router, stav a spouští server.

## Spuštění (placeholder)

### Lokální vývoj
```bash
# Nainstalujte 'sqlx-cli' (pokud budeme používat)
# cargo install sqlx-cli
```

# Spuštění
cargo run

# Docker
```bash
# Build ARM64 obrazu
docker build -t rpi-dashboard .

# Spuštění
docker run -p 8080:8080 rpi-dashboard
```

---

### 3. Vytvoření `.gitignore`

Je klíčové ignorovat artefakty kompilace (`target/`) a naše logy (`logs/`).

Vytvořte soubor `dashboard-project/.gitignore` s tímto obsahem:

```gitignore
# Ignoruje výstupy kompilátoru Rustu
/target

# Ignoruje adresář s log soubory
/logs/

# Ignoruje IDE specifické soubory
.idea/
.vscode/

# Cargo lock soubor by měl být ve verzování, ale pokud byste nechtěl...
# Cargo.lock