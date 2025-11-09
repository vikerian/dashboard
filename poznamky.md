# zdroj informaci

- google gemini 2.5 pro, pod studentskou licensi, rozhovor : https://gemini.google.com/app/56a60d3fb92ba327

---

dashboard_2025-11-01_graphviz.code:
Co diagram ukazuje:
Uživatel (Prohlížeč): Posílá požadavky na náš server.

axum::Router: Náš Front Controller, který přijímá všechny požadavky a třídí je.

Controllers: Router volá specifickou funkci (handler) pro každou cestu (page_postgres, page_timescale...).

AppState (DI): Každý controller dostane "injektovaný" přístup k našemu sdílenému stavu. V AppState jsou uloženy klíčové prostředky: PgPool (připojení k Postgres), ValkeyClient a Tera (šablonovací engine).

Services (Facade): Controllery samy nedělají SQL dotazy. Volají naše servisní funkce (např. get_valkey_kv), které skrývají implementační detaily (to je Facade pattern).

Externí Služby: Naše servisní funkce pak komunikují přes síť s databázemi (PostgreSQL a Valkey).

Data: Diagram ukazuje, že Postgres u sebe drží jak tabulku moje_data, tak senzor_data (Hypertable), zatímco Valkey drží jednoduchý K:V pár.

Renderování: Nakonec controllery použijí Tera engine (z AppState) k vykreslení HTML a pošlou ho zpět uživateli (tato zpětná šipka není v diagramu kreslená, aby byl přehledný).

---
