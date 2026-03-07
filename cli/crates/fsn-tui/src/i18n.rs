// Minimal i18n — compile-time static string lookups per language.
// Key convention: "section.key" e.g. "welcome.title", "status.running"
// English is always the fallback for missing keys.

use crate::app::Lang;

pub fn t<'a>(lang: Lang, key: &'a str) -> &'a str {
    match lang {
        Lang::De => de(key).unwrap_or_else(|| en(key).unwrap_or(key)),
        Lang::En => en(key).unwrap_or(key),
    }
}

fn de(key: &str) -> Option<&'static str> {
    Some(match key {
        // Welcome screen
        "welcome.title"         => "Willkommen bei FreeSynergy.Node",
        "welcome.subtitle"      => "Dezentrale Infrastruktur — frei und selbst betrieben",
        "welcome.new_project"   => "Neues Projekt",
        "welcome.open_project"  => "Vorhandenes Projekt",
        "welcome.open_disabled" => "(bald verfügbar)",
        "welcome.hint"          => "Tab=Sprache  Enter=Auswahl  q=Beenden",
        // System info labels
        "sys.host"   => "Host",
        "sys.user"   => "Benutzer",
        "sys.ip"     => "IP",
        "sys.ram"    => "RAM",
        "sys.cpu"    => "CPU",
        "sys.uptime" => "Laufzeit",
        "sys.podman" => "Podman",
        "sys.arch"   => "Architektur",
        // Language picker
        "lang.label" => "Sprache",
        "lang.de"    => "Deutsch",
        "lang.en"    => "English",
        // Dashboard
        "dash.services"  => "Services",
        "dash.col.name"  => "Name",
        "dash.col.type"  => "Typ",
        "dash.col.domain"=> "Domain",
        "dash.col.status"=> "Status",
        "dash.hint"      => "↑↓=Nav  d=Deploy  r=Restart  x=Entfernen  l=Logs  q=Beenden",
        // Sidebar
        "sidebar.system" => "System",
        // Status badges
        "status.running" => "● Aktiv",
        "status.stopped" => "○ Gestoppt",
        "status.error"   => "✗ Fehler",
        "status.unknown" => "? Unbekannt",
        // Logs overlay
        "logs.hint"      => "q=Schließen  ↑↓=Scrollen",
        _ => return None,
    })
}

fn en(key: &str) -> Option<&'static str> {
    Some(match key {
        // Welcome screen
        "welcome.title"         => "Welcome to FreeSynergy.Node",
        "welcome.subtitle"      => "Decentralized infrastructure — free and self-hosted",
        "welcome.new_project"   => "New Project",
        "welcome.open_project"  => "Open Project",
        "welcome.open_disabled" => "(coming soon)",
        "welcome.hint"          => "Tab=Language  Enter=Select  q=Quit",
        // System info labels
        "sys.host"   => "Host",
        "sys.user"   => "User",
        "sys.ip"     => "IP",
        "sys.ram"    => "RAM",
        "sys.cpu"    => "CPU",
        "sys.uptime" => "Uptime",
        "sys.podman" => "Podman",
        "sys.arch"   => "Arch",
        // Language picker
        "lang.label" => "Language",
        "lang.de"    => "Deutsch",
        "lang.en"    => "English",
        // Dashboard
        "dash.services"  => "Services",
        "dash.col.name"  => "Name",
        "dash.col.type"  => "Type",
        "dash.col.domain"=> "Domain",
        "dash.col.status"=> "Status",
        "dash.hint"      => "↑↓=Nav  d=Deploy  r=Restart  x=Remove  l=Logs  q=Quit",
        // Sidebar
        "sidebar.system" => "System",
        // Status badges
        "status.running" => "● Running",
        "status.stopped" => "○ Stopped",
        "status.error"   => "✗ Error",
        "status.unknown" => "? Unknown",
        // Logs overlay
        "logs.hint"      => "q=Close  ↑↓=Scroll",
        _ => return None,
    })
}
