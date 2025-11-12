# KERN

**Numerologisches Reduktions-Tool für Terminal und Web**

KERN berechnet Quersummen von Wörtern, Zahlen und Daten und zeigt deren numerologische Bedeutung. Das Tool kann verschiedene Verschlüsselungssysteme (Cipher) verwenden und komplexe Kombinationen erstellen.

---

## Was macht KERN?

KERN nimmt Text oder Zahlen, wandelt jeden Buchstaben in eine Zahl um und reduziert diese auf eine einstellige Zahl (1-9) oder eine Masterzahl (11, 22, 33).

**Beispiel:**
```bash
kern test
# test [ordinal]: 3
```

So funktioniert's:
- t=20, e=5, s=19, t=20
- 20+5+19+20 = 64
- 6+4 = 10
- 1+0 = 1... wait, das Ergebnis ist 3

Mit `--verbose` siehst du jeden Schritt:
```bash
kern test --verbose
# test [ordinal]
# test → [20+5+19+20] = 64
# → 6+4 = 10
# → 1+0 = 1
# → Quersumme: 1
```

---

## Installation

**Aus dem Repository:**
```bash
cargo build --release
```

Das Binary findest du dann in `target/release/kern`

---

## Grundlegende Nutzung

### Einzelne Wörter reduzieren
```bash
kern hallo
# hallo [ordinal]: 6

kern welt
# welt [ordinal]: 9
```

### Mehrere Wörter auf einmal
```bash
kern hallo welt beispiel
# hallo [ordinal]: 6
# welt [ordinal]: 9
# beispiel [ordinal]: 7
```

### Gesamtsumme berechnen
```bash
kern --total wort1 wort2 wort3
# wort1 [ordinal]: 9
# wort2 [ordinal]: 11
# wort3 [ordinal]: 4
# Gesamtsumme: 24 → 6
```

### Bedeutungen nachschlagen
```bash
kern --lookup test
# Zeigt eine Tabelle mit der numerologischen Bedeutung der Zahl
```

Mit `--licht` und `--schatten` kannst du zusätzliche Aspekte anzeigen:
```bash
kern --lookup --licht --schatten test
```

---

## Cipher-Systeme

KERN unterstützt verschiedene Verschlüsselungssysteme. Jedes System ordnet Buchstaben anders zu:

### Verfügbare Cipher anzeigen
```bash
kern --list-ciphers
```

### Einen bestimmten Cipher verwenden
```bash
kern --cipher py test
# test [pythagorean]: 2

kern --cipher ch test
# test [chaldean]: 4
```

### Mehrere Cipher gleichzeitig
```bash
kern --cipher ch,py,ro test
# test [chaldean]: 4
# test [pythagorean]: 2
# test [reverse_ordinal]: 7
```

### Alle Cipher auf einmal
```bash
kern --cipher all test
```

### Cipher-Abkürzungen
- `or` = Ordinal (Standard, A=1, B=2, ... Z=26)
- `py` = Pythagorean (A=1, B=2, ... I=9, dann wieder 1-9)
- `ch` = Chaldean
- `ro` = Reverse Ordinal (A=26, B=25, ... Z=1)
- `rp` = Reverse Pythagorean
- `ag` = Agrippa
- `pr` = Primes (Primzahlen)
- `fi` = Fibonacci
- `sq` = Squares (Quadratzahlen)
- `cu` = Cubes (Kubikzahlen)
- `se` = Septenary

---

## Lokale Flags: Pro Wort andere Einstellungen

Du kannst für jedes Wort individuelle Einstellungen verwenden. Lokale Flags kommen **nach** dem Wort, auf das sie sich beziehen.

### Einzelnes Wort verbose anzeigen
```bash
kern wort1 wort2 -v wort3
# wort1: normal
# wort2: mit detailliertem Reduktionsprozess
# wort3: normal
```

### Zusätzliche Cipher für einzelne Wörter
```bash
kern --cipher ch wort1 wort2 -c py wort3
# wort1: nur chaldean (global)
# wort2: chaldean + pythagorean (global + lokal)
# wort3: nur chaldean (global)
```

**Wichtig:** Lokale Cipher-Flags (`-c`) **ergänzen** die globalen Cipher, sie ersetzen sie nicht!

### Kombinationen
```bash
kern --cipher ch,ro wort1 wort2 -v -c py wort3 -c fi,sq wort4
# wort1: chaldean, reverse_ordinal
# wort2: chaldean, reverse_ordinal, pythagorean (mit verbose)
# wort3: chaldean, reverse_ordinal
# wort4: chaldean, reverse_ordinal, fibonacci, squares
```

---

## Datum-Reduktion

KERN kann auch Datumsangaben reduzieren.

### Einzelnes Datum
```bash
kern --date 28.07.2025
```

### Datum-Bereiche
```bash
# Die nächsten 7 Tage (relativ)
kern --date 0..6

# Von gestern bis übermorgen
kern --date -1..2

# Bestimmter Zeitraum
kern --date 01.01.2025..07.01.2025
```

### Mit Cipher kombinieren
```bash
kern --date 0..7 --cipher ch,py
```

---

## Fortgeschrittene Kombinationen

### Mehrere Wörter mit Lookup und Total
```bash
kern --cipher py,ch --lookup --total liebe licht frieden
```

### Verbose für bestimmte Wörter + Gesamtsumme
```bash
kern --total wort1 -v wort2 wort3 -v
```

### Zeichenlänge anzeigen
```bash
kern --length test beispiel wort
# test [ordinal]: 3 (4)
# beispiel [ordinal]: 7 (8)
# wort [ordinal]: 9 (4)
```

---

## Web-API Server

KERN kann auch als HTTP-Server laufen:

```bash
cargo run --bin kern-server
# Server läuft auf http://localhost:3000
```

### API-Endpunkte

**Wörter reduzieren:**
```
GET /reduce?input=test,hallo&debug=true
```

**Bedeutung nachschlagen:**
```
GET /lookup/7
GET /lookup/7?parts=light
GET /lookup?numbers=1,2,3&parts=both
```

**Datum reduzieren:**
```
GET /date?range=0..7
```

---

## Masterzahlen

Die Zahlen **11, 22, 33** werden nicht weiter reduziert. Sie haben besondere spirituelle Bedeutung:

```bash
kern --lookup test
# Wenn das Ergebnis 11, 22 oder 33 ist, bleibt es so
```

---

## Docker

```bash
# Image bauen
docker build -t kern-server .

# Container starten
docker run --rm -p 3000:3000 kern-server

# Oder von Docker Hub
docker pull 24biteggplant/kern-server:latest
docker run -d -p 3000:3000 24biteggplant/kern-server:latest
```

---

## Wichtige Hinweise

1. **Globale Flags müssen VOR den Wörtern stehen:**
   - ✅ `kern --cipher ch test`
   - ❌ `kern test --cipher ch` (behandelt --cipher als Wort)

2. **Lokale Flags kommen NACH dem Wort:**
   - ✅ `kern wort1 wort2 -c py wort3`
   - `-c py` gilt für wort2

3. **Lokale Cipher-Flags sind additiv:**
   - `kern --cipher ch wort1 -c py` → wort1 nutzt ch + py

---

## Hilfe & Weitere Informationen

```bash
kern --help          # Zeigt alle Optionen
kern --version       # Zeigt Version
kern --list-ciphers  # Zeigt alle verfügbaren Cipher
```

**Entwickler-Doku:** Siehe `CLAUDE.md` für technische Details zur Architektur und Entwicklung.

---

**STATUS:** Stabil | **VERSION:** 0.2.9
