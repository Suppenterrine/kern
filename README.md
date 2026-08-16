# KERN

**Numerologische Reduktion auf Basis von 11 Chiffren-Systemen.** KERN berechnet die digitale Wurzel von Wörtern, Daten und Zahlen mittels verschiedener Verschlüsselungssysteme und generiert numerologische Analysen. Verfügbar als CLI und REST-API.

---

## tl;dr

| Befehl | Ergebnis |
|--------|----------|
| `kern test` | Ordinal-Reduktion von "test" |
| `kern --cipher all test` | Alle 11 Chiffren für "test" |
| `kern --lookup test` | Mit numerologischen Bedeutungen |
| `kern --spektra test` | SPEKTRA-Analyse (Prompt in Zwischenablage) |
| `kern-server` | REST-API auf Port 3000 |

---

## Installation

<details>
<summary><b>Rust-Projekt (für Entwickler)</b></summary>

```bash
# Clone & Build
git clone <repo>
cd kern
cargo build --release

# CLI Binary
./target/release/kern --help

# Server Binary
./target/release/kern-server
```

**Voraussetzungen:** Rust 1.70+, Cargo

**Android/Termux Build:**
```bash
# Ohne Clipboard-Feature (für Plattformen ohne Clipboard-Support)
cargo build --release --no-default-features
```
*Bei deaktiviertem Clipboard-Feature wird der SPEKTRA-Prompt als Text ausgegeben statt in die Zwischenablage kopiert.*

</details>

---

## KERN CLI

### Grundlagen

**Einzelnes Wort reduzieren:**
```bash
$ kern hello
hello → 3 [ordinal]
```

**Mit spezifischen Chiffren:**
```bash
$ kern --cipher chaldean,pythagorean hello
hello
  chaldean     → 2
  pythagorean  → 5
```

**Alle 11 Chiffren:**
```bash
$ kern --cipher all hello
hello
  ordinal              → 3
  reverse_ordinal      → 6
  pythagorean          → 5
  reverse_pythagorean  → 4
  chaldean             → 2
  agrippa              → 3
  primes               → 7
  fibonacci            → 9
  squares              → 9
  cubes                → 7
  septenary            → 2
```

### Flags & Optionen

| Flag | Beschreibung |
|------|-------------|
| `-l, --lookup` | Zeigt numerologische Bedeutungen für alle Reduktionen |
| `--full` | Zeigt positive + negative Aspekte der Bedeutungen |
| `--pos` | Nur positive Aspekte anzeigen |
| `--neg` | Nur negative Aspekte anzeigen |
| `--cipher CIPHER` | Spezifische Chiffren (kommagetrennt oder `all`) |
| `--lang CODE` | Sprache der Inhalte: `en` (Standard), `de`, `fr` |
| `-L, --length` | Zeigt Wort-Länge an |
| `-t, --total` | Berechnet und zeigt die Summe aller Reduktionen |
| `-d, --date RANGE` | Reduziert Daten (z.B. `-d -3..7` oder `-d 25.12.2025`) |
| `--verbose` | Zeigt Berechnungsschritte |
| `--spektra` | Generiert SPEKTRA-Analysen-Prompt (Zwischenablage) |
| `--list-ciphers` | Listet alle verfügbaren Chiffren auf |

### Beispiele

**Mit Lookup:**
```bash
$ kern --cipher ordinal,chaldean --lookup hello
hello
  ordinal   → 3
  chaldean  → 2

3 · Kreativität, Ausdruck, Geselligkeit
  └─ hello [ordinal]

2 · Dualität, Beziehung, Harmonie, Sensibilität
  └─ hello [chaldean]
```

**Mit Summe:**
```bash
$ kern --cipher all --total hello
hello
  ordinal       → 3
  [... weitere Chiffren ...]

Total: 53 → 8
```

**Datums-Reduktion:**
```bash
$ kern --date -3..2
Offset  Datum      Reduktion [ordinal]
   -3   23.11.2025        8
   -2   24.11.2025        9
   -1   25.11.2025        1
    0   26.11.2025        2
   +1   27.11.2025        3
   +2   28.11.2025        4
```

**SPEKTRA-Analyse:**
```bash
$ kern --spektra test
⊕ Prompt in Zwischenablage kopiert
```
(Der vollständige numerologische Analyseprompt ist in der Zwischenablage und kann mit `Ctrl+V` eingefügt werden)

*Hinweis: Bei Builds ohne Clipboard-Feature (`--no-default-features`) wird der Prompt direkt als Text ausgegeben.*

**Position der Flags:** Flags müssen **vor** dem ersten Input stehen.

```bash
$ kern --cipher chaldean hello    # ✅
$ kern hello --cipher chaldean    # ❌ "--cipher" wird als Wort reduziert
```

Ausnahme sind `-t/--total` und `-l/--lookup`, die auch hinten funktionieren.
Diese Inkonsistenz ist bekannt; Analyse und Lösungsvorschlag stehen in
[docs/proposals/cli-argument-order.md](docs/proposals/cli-argument-order.md).

---

## KERN-Server

### Start

**Lokal:**
```bash
$ kern-server
KERN Server v1.0.2 listening on http://0.0.0.0:3000
```

**Docker:**
```bash
docker pull 24biteggplant/kern-server:latest
docker run -d -p 3000:3000 --name kern-server 24biteggplant/kern-server:latest
```

**Live API:**
```
https://kern.lukasbaumert.de
```

---

### REST-API Endpoints

#### `GET /`

Schlanker Service-Deskriptor für Health-Checks: Name, Version, Sprachen und ein
Zeiger auf die Doku. Die vollständige Endpunkt-Übersicht liegt auf `/help`.

**Beispiel:**
```bash
$ curl "https://kern.lukasbaumert.de/"

{
  "name": "KERN API",
  "version": "2.0.0",
  "languages": ["de", "en", "fr"],
  "default_language": "en",
  "documentation": "/help"
}
```

#### `GET /help`

Vollständige Endpunkt-Übersicht mit Beispielen.

**Beispiel:**
```bash
$ curl "https://kern.lukasbaumert.de/help"

{
  "name": "KERN API",
  "version": "2.0.0",
  "endpoints": [...],
  "examples": {...}
}
```

#### `GET /version`

Gibt Name und Versionsnummer zurück.

**Beispiel:**
```bash
$ curl "https://kern.lukasbaumert.de/version"

{
  "name": "kern",
  "version": "1.0.2"
}
```

#### `GET /reduce`

Reduziert ein oder mehrere Inputs mit einem oder mehreren Chiffren.

**Parameter:**
- `input` (erforderlich): Kommagetrennte Eingaben
- `cipher` (optional): Kommagetrennte Cipher-Codes oder `all`
- `debug` (optional): `true` für Berechnungsschritte (chains)
- `length` (optional): `true` für Wort-Längen
- `onlyTotal` (optional): `true` für nur die Gesamtsumme

**Beispiele:**

**Einfache Reduktion:**
```bash
$ curl "https://kern.lukasbaumert.de/reduce?input=Wickfeld"

{
  "items": [
    {
      "input": "Wickfeld",
      "value": 1
    }
  ],
  "total": 1
}
```

**Multi-Input:**
```bash
$ curl "https://kern.lukasbaumert.de/reduce?input=Test,Love,Life"

{
  "items": [
    {"input": "Test", "value": 1},
    {"input": "Love", "value": 9},
    {"input": "Life", "value": 3}
  ],
  "total": 4
}
```

**Multi-Cipher:**
```bash
$ curl "https://kern.lukasbaumert.de/reduce?input=Test&cipher=or,py,ch"

{
  "items": [
    {
      "input": "Test",
      "ciphers": [
        {"name": "ordinal", "code": "or", "value": 1},
        {"name": "pythagorean", "code": "py", "value": 1},
        {"name": "chaldean", "code": "ch", "value": 4}
      ]
    }
  ],
  "total": 1
}
```

**Alle Ciphers:**
```bash
$ curl "https://kern.lukasbaumert.de/reduce?input=Test&cipher=all"

{
  "items": [
    {
      "input": "Test",
      "ciphers": [
        {"name": "ordinal", "code": "or", "value": 1},
        {"name": "reverse_ordinal", "code": "ro", "value": 8},
        {"name": "pythagorean", "code": "py", "value": 1},
        ... (alle 11 Ciphers)
      ]
    }
  ],
  "total": 1
}
```

**Mit Debug-Chains:**
```bash
$ curl "https://kern.lukasbaumert.de/reduce?input=Test&cipher=all&debug=true"

{
  "items": [
    {
      "input": "Test",
      "ciphers": [
        {
          "name": "ordinal",
          "code": "or",
          "value": 1,
          "chain": ["Test", "64", "10", "1"]
        },
        ...
      ]
    }
  ],
  "total": 1
}
```

#### `GET /lookup/:number`

Bedeutung einer einzelnen Zahl.

**Parameter:**
- `parts` (optional): `full`, `pos`, `neg`, `both` (legacy: `light`, `shadow`)
- `lang` (optional): `en` (Standard), `de`, `fr` — siehe [Sprachen](#sprachen)

**Beispiel:**
```bash
$ curl "https://kern.lukasbaumert.de/lookup/7?parts=full"

{
  "number": 7,
  "lang": "en",
  "meaning": "Depth, intuition, analysis, withdrawal",
  "positive": "Spiritual insight, analytical thinking...",
  "negative": "Isolation, mistrust..."
}
```

Für Deutsch explizit `lang=de` angeben:
```bash
$ curl "https://kern.lukasbaumert.de/lookup/7?parts=full&lang=de"

{
  "number": 7,
  "lang": "de",
  "meaning": "Tiefe, Intuition, Analyse, Rückzug",
  ...
}
```

#### `GET /lookup`

Bedeutungen mehrerer Zahlen.

**Parameter:**
- `numbers` (erforderlich): Kommagetrennte Zahlen
- `parts` (optional): `full`, `pos`, `neg`, `both`
- `lang` (optional): `en` (Standard), `de`, `fr`

**Beispiel:**
```bash
$ curl "https://kern.lukasbaumert.de/lookup?numbers=1,7,11&parts=full"

{
  "lang": "de",
  "items": [
    {
      "number": 1,
      "meaning": "Ursprung, Wille, Individualität",
      "positive": "...",
      "negative": "..."
    },
    {"number": 7, "meaning": "...", "positive": "...", "negative": "..."},
    {"number": 11, "meaning": "...", "positive": "...", "negative": "..."}
  ]
}
```

#### `GET /date`

Reduziert einen Datums-Bereich.

**Parameter:**
- `range` (erforderlich): Offset-Range (z.B. `-3..7`) oder Datums-Range (z.B. `25.12.2025..02.01.2026`)
- `debug` (optional): `true` für Berechnungsschritte
- `lang` (optional): `en` (Standard), `de`, `fr`

**Beispiel:**
```bash
$ curl "https://kern.lukasbaumert.de/date?range=0..3&debug=true"

{
  "lang": "de",
  "dates": [
    {
      "offset": 0,
      "date": "26.11.2025",
      "value": 2,
      "meaning": "Dualität, Beziehung, Harmonie",
      "chain": ["26112025", "19", "10", "1"]
    },
    ...
  ]
}
```

#### `GET /spektra`

Generiert SPEKTRA-Analyseprompt mit allen 11 Chiffren.

**Parameter:**
- `word` (erforderlich): Wort zur Analyse
- `lang` (optional): `en` (Standard) oder `de`; `fr` wird abgelehnt

**Beispiel:**
```bash
$ curl "https://kern.lukasbaumert.de/spektra?word=Love"

{
  "lang": "en",
  "prompt": "You are the SPEKTRA analysis module...\n[Vollständiger Prompt mit allen Chiffren und Bedeutungen]"
}
```

---

## Sprachen

**Standard ist Englisch.** Für Deutsch oder Französisch muss `lang` gesetzt werden.

Nicht alle Inhalte gibt es in allen Sprachen:

| Inhalt | `en` | `de` | `fr` | Dateien |
|--------|:----:|:----:|:----:|---------|
| Zahlenbedeutungen (`/lookup`, `/date`) | ✅ | ✅ | ✅ | `bedeutungen.{en,fr}.yaml`, `bedeutungen.yaml` (de) |
| SPEKTRA-Prompt (`/spektra`) | ✅ | ✅ | ❌ | `spektra_prompt.{en,}txt` |
| RTAP-Prompts (`/rtap`) | ✅ | ✅ | ❌ | `rtap_*`-Keys in den Bedeutungsdateien |
| Fehlermeldungen | ✅ | — | — | immer Englisch |

**Keine stillen Fallbacks.** `/spektra?lang=fr` liefert keinen englischen Text,
sondern einen Fehler:

```json
{"code": "language_not_available", "error": "prompts are not available in 'fr'. available: de, en"}
```

Das ist Absicht: eine Antwort in einer anderen Sprache als der angefragten wäre
eine falsche Antwort im Gewand einer erfolgreichen. Siehe
[docs/PRINCIPLES.md](docs/PRINCIPLES.md).

Berechnungen und Chiffren-Namen sind sprachunabhängig.

**Verwendung:**

```bash
# API
$ curl "https://kern.lukasbaumert.de/lookup/7?parts=full&lang=fr"
$ curl "https://kern.lukasbaumert.de/spektra?word=Love&lang=de"

# CLI
$ kern --lang de --lookup Wickfeld
$ kern --lang de --rtap 1
```

**Verhalten:**

- Ohne `lang` wird **Englisch** geliefert. Englisch ist der internationale
  Standard für eine öffentliche API; für Deutsch muss `lang=de` gesetzt werden.
- Regions-Subtags werden akzeptiert und ignoriert: `en-US` → `en`, `fr_CA` → `fr`.
- Ein nicht unterstützter Code wird **abgelehnt**, nicht stillschweigend
  ersetzt:

  ```bash
  $ curl -i "https://kern.lukasbaumert.de/lookup/7?lang=es"
  HTTP/1.1 400 Bad Request

  {"code": "unsupported_language", "error": "unsupported language 'es'. supported: de, en, fr"}
  ```

- Jede Antwort enthält ein `lang`-Feld mit der tatsächlich verwendeten Sprache.
- `lang` steuert die **Inhalte**, nicht das Protokoll: Fehlermeldungen sind
  immer Englisch.

---

## Fehlerformat

Jeder Fehler liefert einen stabilen `code` und einen menschenlesbaren `error`-Text
— **von API und CLI gleichermaßen**, im selben Format:

```json
{"code": "invalid_range", "error": "invalid range specification"}
```

**Gegen `code` programmieren, nicht gegen `error`** — der Text ist Prosa und
kann jederzeit umformuliert werden, der Code nicht.

Im TTY-Modus gibt das CLI stattdessen Klartext auf stderr aus und beendet mit
Exit-Code 1. Vollständige Referenz: [docs/reference/error-codes.md](docs/reference/error-codes.md).

| Code | Bedeutung |
|------|-----------|
| `input_missing` | `input`-Parameter fehlt |
| `no_valid_inputs` | `input` enthielt keine verwertbaren Werte |
| `no_valid_ciphers` | `cipher` gesetzt, aber ohne gültige Codes |
| `unknown_cipher` | Unbekannter Chiffren-Code |
| `unsupported_language` | `lang` ist keine unterstützte Sprache |
| `language_not_available` | Sprache ist gültig, aber für diese Ressource nicht verfügbar (z. B. `fr` bei Prompts) |
| `invalid_range` | `range` nicht parsebar |
| `word_missing` | `word`-Parameter fehlt (`/spektra`) |
| `insufficient_inputs` | `/phase` braucht mindestens 2 Inputs |
| `invalid_rtap_part` | `part` muss 1, 2 oder `both` sein |
| `rtap_prompt_missing` | RTAP-Prompt nicht in der Konfiguration |
| `spektra_failed` | SPEKTRA-Prompt konnte nicht erzeugt werden |
| `invalid_arguments` | Nur CLI: Argumente oder Flag-Kombination nicht interpretierbar |

**Neue Sprache hinzufügen:**

1. `bedeutungen.<code>.yaml` anlegen (gleiche Zahlen-Keys wie `bedeutungen.yaml`)
2. Variante in `Lang` ergänzen (`src/lib.rs`) — `code()`, `ALL` und
   `missing_meaning()` werden vom Compiler eingefordert
3. Datei in `bedeutungen_source()` einbinden
4. Entscheiden, ob die Prompts mit übersetzt werden. Der Compiler erzwingt die
   Entscheidung: `prompt_assets()` und `rtap_source()` matchen erschöpfend.
   Ohne Übersetzung → `None`, die Sprache wird auf den Prompt-Endpunkten sauber
   abgelehnt. Mit Übersetzung → `spektra_prompt.<code>.txt`, `rtap_*`-Keys in der
   Bedeutungsdatei, `SpektraLabels`-Konstante und Eintrag in `Lang::PROMPT_LANGS`
5. `cargo test` — die Tests prüfen Vollständigkeit, Key-Gleichheit aller Sprachen
   und dass Template und Platzhalter-Labels zusammenpassen

---

## Verfügbare Chiffren

| Name | Short | Beschreibung |
|------|-------|-------------|
| ordinal | `or` | A=1..Z=26 |
| reverse_ordinal | `ro` | A=26..Z=1 |
| pythagorean | `py` | Zyklisch 1-9 |
| reverse_pythagorean | `rp` | Reverse 1-9 |
| chaldean | `ch` | Antike Zuordnung |
| agrippa | `ag` | Ordinal (esoterisch) |
| primes | `pr` | Primzahlen-Sequenz |
| fibonacci | `fi` | Fibonacci-Sequenz |
| squares | `sq` | Quadratzahlen |
| cubes | `cu` | Kubikzahlen |
| septenary | `se` | Zyklisch 1-7 |

---

**STATUS:** Stabil | **VERSION:** 2.0.0

