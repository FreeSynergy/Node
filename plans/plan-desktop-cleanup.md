# FreeSynergy.Desktop — Konkreter Code-Aufräumplan

**Repo:** https://github.com/FreeSynergy/FreeSynergy.Desktop  
**Stand:** 18 Commits, `crates/` Verzeichnis, 100% Rust

---

## WICHTIG: Warum dieser Plan anders ist

Ich konnte die einzelnen Dateien im Repo wegen GitHub-Rate-Limits nicht direkt lesen. Deshalb ist dieser Plan als **Anweisungs-Set für Claude Code** geschrieben — er soll den Code SELBST analysieren und dann die Änderungen machen. Jeder Schritt enthält den exakten Befehl den Claude Code ausführen soll.

**Gib Claude Code diesen Plan als CLAUDE.md oder als Prompt.**

---

## Schritt 0: Analyse — Was ist da?

Claude Code soll ZUERST den gesamten Code analysieren, BEVOR er irgendwas ändert.

```
ANWEISUNG AN CLAUDE CODE:

1. Zeige die komplette Verzeichnisstruktur:
   find . -name "*.rs" -o -name "*.toml" -o -name "*.css" -o -name "*.html" | sort

2. Zeige den Workspace Cargo.toml:
   cat Cargo.toml

3. Zeige ALLE Crate-Namen und ihre Cargo.toml:
   for f in $(find crates -name "Cargo.toml"); do echo "=== $f ==="; cat "$f"; done

4. Finde ALLEN alten Code der weg muss:
   grep -rn "FsyError\|Fsy[A-Z]\|fsy_\|fsy-" --include="*.rs" --include="*.toml" .
   grep -rn "podman.sock\|podman\.sock\|/run/podman\|podman_socket\|bollard" --include="*.rs" .
   grep -rn "docker\|docker-compose\|docker_compose" --include="*.rs" .
   grep -rn "ratatui\|rat_salsa\|rat-salsa\|crossterm" --include="*.rs" --include="*.toml" .
   grep -rn "conductor\|Conductor" --include="*.rs" .

5. Finde die UI/Farb-Definitionen:
   grep -rn "color\|Color\|background\|Background\|#[0-9a-fA-F]\{6\}\|rgb\|cyan\|darkcyan" --include="*.rs" --include="*.css" .

6. Finde die Window/Desktop-Konfiguration:
   grep -rn "WindowBuilder\|with_window\|with_decorations\|inner_size\|LogicalSize\|dioxus::desktop" --include="*.rs" .

7. Erstelle eine LISTE aller Probleme die Du findest, BEVOR Du irgendetwas änderst.
   Formatiere als Tabelle: Datei | Zeile | Problem | Lösung
```

---

## Schritt 1: fsy → fsn Umbenennung (ÜBERALL)

```
ANWEISUNG AN CLAUDE CODE:

Suche und ersetze in ALLEN Dateien:

IN RUST-DATEIEN (*.rs):
- "FsyError" → "FsnError"
- "FsyResult" → "FsnResult"
- "fsy_error" → "fsn_error"
- "fsy_types" → "fsn_types"
- "fsy_config" → "fsn_config"
- "fsy_i18n" → "fsn_i18n"
- "fsy_db" → "fsn_db"
- "fsy_sync" → "fsn_sync"
- "fsy_store" → "fsn_store"
- "fsy_theme" → "fsn_theme"
- "fsy_help" → "fsn_help"
- "fsy_ui" → "fsn_ui"
- "fsy_bus" → "fsn_bus"
- "fsy_channel" → "fsn_channel"
- "fsy_bot" → "fsn_bot"
- "fsy_crypto" → "fsn_crypto"
- "fsy_container" → "fsn_container"
- "fsy_template" → "fsn_template"
- "fsy_health" → "fsn_health"
- "fsy_auth" → "fsn_auth"
- "fsy_federation" → "fsn_federation"
- "fsy_pkg" → "fsn_pkg"
- "fsy_plugin" → "fsn_plugin"
- "fsy_bridge" → "fsn_bridge"
- "fsy_llm" → "fsn_llm"
- Jedes andere "fsy_" → "fsn_"
- Jedes "Fsy" am Wortanfang → "Fsn"

IN CARGO.TOML-DATEIEN:
- Crate-Namen: "fsy-error" → "fsn-error" usw.
- Package-Namen: name = "fsy-*" → name = "fsn-*"
- Dependencies: fsy-* → fsn-*

IN CSS/STYLE-STRINGS:
- "--fsy-" → "--fsn-"

NACHDEM alle Ersetzungen gemacht sind:
- cargo check (muss kompilieren!)
- Wenn es nicht kompiliert, zeige die Fehler und fixe sie
```

---

## Schritt 2: Podman-Socket-Code KOMPLETT entfernen

```
ANWEISUNG AN CLAUDE CODE:

1. Finde alle Dateien die podman.sock / Socket / bollard referenzieren:
   grep -rn "podman.sock\|podman_socket\|bollard\|PodmanConnect\|socket_path\|/run/podman\|/run/user.*podman" --include="*.rs" .

2. Finde den "Conductor" oder ähnlichen Container-Monitor:
   grep -rn "conductor\|Conductor\|container_monitor\|ContainerMonitor\|PodmanMonitor" --include="*.rs" .

3. Für JEDE gefundene Datei:
   - Wenn die Datei NUR Podman-Socket-Code enthält → LÖSCHE die gesamte Datei
   - Wenn die Datei gemischten Code enthält → Entferne NUR die Socket-bezogenen Funktionen/Structs
   - Ersetze die Container-Status-Anzeige durch einen Placeholder:
     "Container status: use 'systemctl --user status <service>' (Podman socket removed)"

4. Entferne bollard aus allen Cargo.toml Dependencies

5. Wenn es einen "Conductor"-View/Tab in der UI gibt:
   - Ersetze den Inhalt durch: "Container management via Quadlet/systemctl (coming soon)"
   - Entferne den "Refreshes every 3s" Timer
   - Entferne den Socket-Verbindungsversuch

6. cargo check — muss kompilieren!
```

---

## Schritt 3: Farben komplett überarbeiten

```
ANWEISUNG AN CLAUDE CODE:

Das aktuelle Farbschema hat schwarze Schrift auf dunklem Hintergrund → unlesbar.
Ersetze ALLE Farb-Definitionen durch dieses Schema:

SUCHE ALLE STELLEN wo Farben definiert werden:
- CSS-Strings in rsx! Makros
- Color-Konstanten
- Style-Definitionen
- Theme-Dateien
- Inline-Styles

ERSETZE mit diesem Farbschema (Midnight Blue):

Hintergründe:
  bg-base:      #0c1222    (Haupt-Hintergrund, tiefes Dunkelblau)
  bg-surface:   #162032    (Karten, Panels)
  bg-elevated:  #1e2d45    (Erhöhte Elemente)
  bg-sidebar:   #0a0f1a    (Sidebar, dunkler als Content)
  bg-input:     #0f1a2e    (Input-Felder)
  bg-hover:     #243352    (Hover-Zustand)

Text (HELL auf DUNKEL — hoher Kontrast!):
  text-primary:   #e8edf5  (Haupttext — fast weiß)
  text-secondary: #a0b0c8  (Sekundärtext — helles Blaugrau)
  text-muted:     #5a6e88  (Gedämpft)
  text-bright:    #ffffff  (Reinweiß — Buttons, Hervorhebungen)

Primärfarbe:
  primary:       #4d8bf5   (Leuchtendes Blau)
  primary-hover: #3a78e8   (Dunkler beim Hover)

Akzent:
  accent:        #22d3ee   (Cyan — NUR als Akzent, NIE als Hintergrund)

Status:
  success: #34d399  (Grün)
  warning: #fbbf24  (Gelb)
  error:   #f87171  (Rot)
  info:    #60a5fa  (Blau)

Borders:
  border:       rgba(148, 170, 200, 0.18)  (Subtil)
  border-focus: #4d8bf5                     (Blau bei Fokus)

Sidebar:
  sidebar-bg:        #0a0f1a  (Sehr dunkel)
  sidebar-text:      #a0b0c8  (Helles Grau)
  sidebar-active:    #4d8bf5  (Blau)
  sidebar-active-bg: rgba(77, 139, 245, 0.15)  (Leichter Blau-Schimmer)

WICHTIG:
- NIEMALS dunkle Schrift auf dunklem Hintergrund
- NIEMALS Cyan/Teal als Hintergrundfarbe
- Text auf dunklem Hintergrund ist IMMER hell (#e8edf5 oder heller)
- Text auf hellen Buttons ist IMMER weiß (#ffffff)
```

---

## Schritt 4: Fenster-Konfiguration fixen

```
ANWEISUNG AN CLAUDE CODE:

1. Finde die Dioxus Desktop Launch-Konfiguration:
   grep -rn "dioxus::launch\|LaunchBuilder\|desktop::Config\|WindowBuilder\|with_window" --include="*.rs" .

2. Stelle sicher dass diese Einstellungen gesetzt sind:

   let cfg = dioxus::desktop::Config::new()
       .with_window(
           WindowBuilder::new()
               .with_title("FreeSynergy Desktop")
               .with_decorations(true)         // ← SYSTEM-Dekoration (X/Min/Max)
               .with_inner_size(LogicalSize::new(1280, 800))
               .with_min_inner_size(LogicalSize::new(900, 600))
               .with_maximized(false)
       );

   WICHTIG:
   - with_decorations(true) — damit der X-Button vom System gezeichnet wird
   - with_inner_size — vernünftige Startgröße
   - with_min_inner_size — damit das Fenster nicht zu klein wird

3. Wenn es custom window chrome gibt (eigene Title-Bar mit X-Button):
   - Entferne es ODER stelle sicher dass es korrekt funktioniert
   - Der einfachste Fix ist: System-Dekoration nutzen (with_decorations(true))
```

---

## Schritt 5: Store-Catalog TOML fixen

```
ANWEISUNG AN CLAUDE CODE:

1. Finde wo der Store-Catalog geladen wird:
   grep -rn "catalog\|catalog\.toml\|parse.*toml\|TOML parse error" --include="*.rs" .

2. Finde die Catalog-Datei oder URL:
   grep -rn "catalog\.toml\|raw\.githubusercontent.*catalog\|store.*url\|store.*catalog" --include="*.rs" --include="*.toml" .

3. Das Problem ist: Inline-TOML-Records (alles auf einer Zeile) statt mehrzeilige.
   FALSCH: code = "am"; name = "አማርኛ"; version = "1.0.0"; completeness = 100; direction = "ltr"
   RICHTIG:
   [[languages]]
   code = "am"
   name = "Amharic"
   version = "1.0.0"
   completeness = 100
   direction = "ltr"

4. Wenn der Catalog lokal gespeichert ist → fixe das Format
5. Wenn er von einer URL geladen wird → mache den Parser toleranter ODER
   fixe die Quelle im Store-Repo
6. Füge Fehlerbehandlung hinzu: Wenn Catalog-Parse fehlschlägt → zeige
   freundliche Fehlermeldung, nicht den rohen Parse-Error
```

---

## Schritt 6: Alten Docker/Wizard-Code entfernen

```
ANWEISUNG AN CLAUDE CODE:

1. Finde allen Docker-bezogenen Code:
   grep -rn "docker\|Docker\|docker-compose\|docker_compose\|compose\.yml\|compose\.yaml" --include="*.rs" .

2. Wenn es einen alten Wizard gibt der Docker referenziert:
   - LÖSCHE den gesamten alten Wizard
   - Ersetze ihn mit einem Placeholder-View:
     "Installation Wizard (coming soon — will use fsn-wizard)"

3. Entferne Docker-Dependencies aus Cargo.toml falls vorhanden

4. cargo check
```

---

## Schritt 7: Sauberer Desktop-Vollbild

```
ANWEISUNG AN CLAUDE CODE:

Der Desktop soll den gesamten Bildschirm nutzen (kein Rand/Padding um den Content).

1. Finde das Root-Layout:
   grep -rn "body\|main.*container\|app.*container\|root.*layout\|margin\|padding" --include="*.rs" .

2. Stelle sicher dass das Root-Element keinen Rand hat:
   Das äußerste div/container soll haben:
   - width: 100vw (oder 100%)
   - height: 100vh (oder 100%)
   - margin: 0
   - padding: 0
   - overflow: hidden (Scrolling nur innerhalb von Content-Bereichen)

3. Die Sidebar + Content sollen zusammen 100% der Breite füllen:
   Sidebar: feste Breite (z.B. 240px) oder prozentual
   Content: flex-grow: 1 (nimmt den Rest)
```

---

## Reihenfolge für Claude Code

```
1. Schritt 0: ANALYSE (NUR lesen, NICHTS ändern, Liste erstellen)
2. Schritt 1: fsy → fsn Umbenennung
3. Schritt 2: Podman-Socket-Code entfernen
4. Schritt 3: Farben überarbeiten
5. Schritt 4: Fenster-Konfiguration fixen
6. Schritt 5: Store-Catalog TOML fixen
7. Schritt 6: Docker/Wizard-Code entfernen
8. Schritt 7: Vollbild-Layout

Nach JEDEM Schritt: cargo check
Wenn es nicht kompiliert: SOFORT fixen bevor zum nächsten Schritt
```

---

## Zusätzlicher Hinweis für Claude Code

```
REGELN:
- ENGLISH in code and comments
- Jede Änderung in einem eigenen Commit mit klarer Message
- Wenn Du unsicher bist ob etwas weg kann: Kommentiere es aus statt zu löschen
- Wenn Du einen Import entfernst: Prüfe ob er woanders gebraucht wird
- Immer cargo check und cargo clippy nach Änderungen
- Keine neuen Features hinzufügen — NUR aufräumen und fixen
```
