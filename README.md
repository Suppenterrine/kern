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

**Lokale Cipher pro Input:**
```bash
$ kern input1 input2 -c chaldean input3
```
(input1 und input2 nutzen Standard-Cipher, input3 ergänzt lokal Chaldean)

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
https://kern.drehraum.wtf
```

---

### REST-API Endpoints

#### `GET /`

API-Übersicht mit allen verfügbaren Endpunkten und Beispielen.

**Beispiel:**
```bash
$ curl "https://kern.drehraum.wtf/"

{
  "name": "KERN API",
  "version": "1.0.2",
  "endpoints": [...],
  "examples": {...}
}
```

#### `GET /version`

Gibt Name und Versionsnummer zurück.

**Beispiel:**
```bash
$ curl "https://kern.drehraum.wtf/version"

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
$ curl "https://kern.drehraum.wtf/reduce?input=Wickfeld"

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
$ curl "https://kern.drehraum.wtf/reduce?input=Test,Love,Life"

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
$ curl "https://kern.drehraum.wtf/reduce?input=Test&cipher=or,py,ch"

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
$ curl "https://kern.drehraum.wtf/reduce?input=Test&cipher=all"

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
$ curl "https://kern.drehraum.wtf/reduce?input=Test&cipher=all&debug=true"

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

**Beispiel:**
```bash
$ curl "https://kern.drehraum.wtf/lookup/7?parts=full"

{
  "number": 7,
  "meaning": "Tiefe, Intuition, Analyse, Rückzug",
  "positive": "Spirituelle Tiefe, analytisches Denken...",
  "negative": "Isolation, Misstrauen..."
}
```

#### `GET /lookup`

Bedeutungen mehrerer Zahlen.

**Parameter:**
- `numbers` (erforderlich): Kommagetrennte Zahlen
- `parts` (optional): `full`, `pos`, `neg`, `both`

**Beispiel:**
```bash
$ curl "https://kern.drehraum.wtf/lookup?numbers=1,7,11&parts=full"

{
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

**Beispiel:**
```bash
$ curl "https://kern.drehraum.wtf/date?range=0..3&debug=true"

{
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

**Beispiel:**
```bash
$ curl "https://kern.drehraum.wtf/spektra?word=Love"

{
  "prompt": "Du bist das SPEKTRA-Analysemodul...\n[Vollständiger Prompt mit allen Chiffren und Bedeutungen]"
}
```

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

**STATUS:** Stabil | **VERSION:** 1.1.2

