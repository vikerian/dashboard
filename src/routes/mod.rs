// Deklarujeme pod-moduly. Tím říkáme:
// "Obsah souboru 'web.rs' patří do modulu 'routes::web'"
// "Obsah souboru 'api.rs' patří do modulu 'routes::api'"
// Klíčové slovo 'pub' je zde důležité, aby byly viditelné
// z 'main.rs'.
pub mod api;
pub mod web;
