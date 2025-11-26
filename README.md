# KERN

**Numerologische Reduktion auf Basis von 11 Chiffren-Systemen.** KERN berechnet die digitale Wurzel von Wörtern, Daten und Zahlen mittels verschiedener esoterischer Verschlüsselungssysteme und generiert numerologische Analysen. Verfügbar als CLI und REST-API.

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

```bash
$ kern-server
Listening on http://0.0.0.0:3000
```

### REST-API Endpoints

#### `GET /reduce`

Reduziert ein oder mehrere Inputs.

**Parameter:**
- `input` (erforderlich): Kommagetrennte Eingaben
- `debug` (optional): `true` für Berechnungsschritte
- `length` (optional): `true` für Wort-Längen
- `onlyTotal` (optional): `true` für nur die Gesamtsumme

**Beispiel:**
```bash
$ curl "http://localhost:3000/reduce?input=hello,world"

{
  "items": [
    {"value": 3, "length": 5},
    {"value": 6, "length": 5}
  ],
  "total": 9
}
```

#### `GET /lookup/:number`

Bedeutung einer einzelnen Zahl.

**Parameter:**
- `parts` (optional): `full`, `pos`, oder `neg`

**Beispiel:**
```bash
$ curl "http://localhost:3000/lookup/7"

{
  "number": 7,
  "text": "Tiefe, Intuition, Analyse, Rückzug",
  "positive": "Spirituelle Tiefe...",
  "negative": "Isolation..."
}
```

#### `GET /lookup`

Bedeutungen mehrerer Zahlen.

**Parameter:**
- `numbers` (erforderlich): Kommagetrennte Zahlen
- `parts` (optional): `full`, `pos`, oder `neg`

**Beispiel:**
```bash
$ curl "http://localhost:3000/lookup?numbers=1,5,7"

[
  {"number": 1, "text": "Ursprung, Wille, Individualität..."},
  {"number": 5, "text": "Veränderung, Freiheit, Abenteuer..."},
  {"number": 7, "text": "Tiefe, Intuition, Analyse..."}
]
```

#### `GET /date`

Reduziert einen Datums-Bereich.

**Parameter:**
- `range` (erforderlich): Offset-Range (z.B. `-3..7`) oder Datums-Range (z.B. `25.12.2025..02.01.2026`)
- `debug` (optional): `true` für Berechnungsschritte

**Beispiel:**
```bash
$ curl "http://localhost:3000/date?range=-2..2"

{
  "dates": [
    {"offset": -2, "date": "24.11.2025", "value": 9},
    {"offset": 0, "date": "26.11.2025", "value": 2}
  ]
}
```

#### `GET /spektra`

Generiert SPEKTRA-Analyseprompt mit allen 11 Chiffren.

**Parameter:**
- `word` (erforderlich): Wort zur Analyse

**Beispiel:**
```bash
$ curl "http://localhost:3000/spektra?word=test"

{
  "prompt": "Du bist das SPEKTRA-Analysemodul.\n\n... [gefüllter Prompt mit allen Chiffren und Bedeutungen] ..."
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

**STATUS:** Stabil | **VERSION:** 1.0.2
