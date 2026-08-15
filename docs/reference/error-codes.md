# Fehlercodes

Jeder Fehler von CLI und Server trägt einen stabilen `code` neben dem
menschenlesbaren `error`-Text.

```json
{"code": "invalid_range", "error": "invalid range specification"}
```

**Gegen `code` programmieren, nicht gegen `error`.** Der Text ist Prosa und darf
jederzeit umformuliert werden; der Code ist API-Oberfläche und sein Umbenennen
ist ein Breaking Change.

Fehlertexte sind **immer Englisch**, unabhängig von `lang`. `lang` steuert
Inhalte, nicht das Protokoll — siehe [PRINCIPLES §2](../PRINCIPLES.md).

---

## Aufbau

Definiert als Enum `ErrorCode` in `src/lib.rs`:

```rust
pub enum ErrorCode { InputMissing, NoValidInputs, /* ... */ }
```

Ein Enum, keine freien Strings. Ein nicht deklarierter Code lässt sich damit gar
nicht ausgeben — das ist die Voraussetzung dafür, dass
`cargo xtask check-error-codes` überhaupt etwas beweisen kann.

Zwei Listen:

| Konstante | Bedeutung |
|-----------|-----------|
| `ErrorCode::ALL` | Alle Codes, die eines der beiden Binaries ausgeben kann |
| `ErrorCode::API` | Nur die, die die HTTP-API zurückgibt — muss der OpenAPI-Spec **exakt** entsprechen |

Die Trennung existiert, damit ein reiner CLI-Fehlerfall ehrlich modelliert
werden kann, statt in einen unpassenden API-Code gepresst zu werden, nur damit
eine Liste ordentlich aussieht.

---

## Die Codes

### API und CLI

| Code | Anlass |
|------|--------|
| `input_missing` | `input`-Parameter bzw. Eingabe fehlt |
| `no_valid_inputs` | Eingabe enthielt keine verwertbaren Werte |
| `no_valid_ciphers` | `cipher` gesetzt, aber ohne gültige Codes |
| `unknown_cipher` | Unbekannter Chiffren-Code |
| `unsupported_language` | `lang` ist keine unterstützte Sprache |
| `language_not_available` | Sprache gültig, aber für diese Ressource nicht verfügbar |
| `invalid_range` | `range` nicht parsebar |
| `word_missing` | Wort für SPEKTRA fehlt |
| `insufficient_inputs` | Phasenmatrix braucht mindestens 2 Inputs |
| `invalid_rtap_part` | RTAP-Teil muss 1, 2 oder `both` sein |
| `rtap_prompt_missing` | RTAP-Prompt nicht in der Konfiguration |
| `spektra_failed` | SPEKTRA-Prompt konnte nicht erzeugt werden |

### Nur CLI

| Code | Anlass |
|------|--------|
| `invalid_arguments` | Argumentliste oder Flag-Kombination nicht interpretierbar |

Die HTTP-API hat kein Gegenstück — sie hat keine Flags. Deshalb steht dieser
Code in `ALL`, aber nicht in `API`, und taucht in der OpenAPI-Spec nicht auf.

### `unsupported_language` vs. `language_not_available`

Die Unterscheidung ist für Consumer relevant:

- `unsupported_language` — die Sprache kennt KERN gar nicht (`lang=es`).
  Ein Tippfehler oder eine falsche Annahme.
- `language_not_available` — die Sprache ist gültig, aber diese Ressource gibt
  es nicht darin (`lang=fr` auf `/spektra`). Ein anderer Aufruf mit derselben
  Sprache kann durchaus funktionieren.

---

## Deckungsgleichheit CLI ↔ Server

Beide Binaries beziehen ihre Codes aus demselben Enum in der Bibliothek und
geben dasselbe JSON-Format aus. Wer beide Wege konsumiert, behandelt ein Format.

```bash
$ curl "…/rtap?part=1&lang=fr"
{"code":"language_not_available","error":"prompts are not available in 'fr'. available: de, en"}

$ kern --lang fr --rtap 1
{"code":"language_not_available","error":"RTAP prompts are not available in 'fr'. available: de, en"}
```

Der `code` ist identisch, der Text unterscheidet sich im Wortlaut — genau die
Aufteilung, die oben beschrieben ist.

Im TTY-Modus gibt das CLI stattdessen die Klartextmeldung auf stderr aus und
beendet mit Exit-Code 1. JSON gibt es nur im Pipe-Modus.

---

## Einen Code hinzufügen

1. Variante in `ErrorCode` ergänzen; `as_str()` matcht erschöpfend, der
   Compiler verlangt die Zeichenkette
2. In `ErrorCode::ALL` eintragen — und in `ErrorCode::API`, **falls** die HTTP-API
   ihn zurückgeben kann
3. Bei API-Codes: ins `enum` in `api/kern.definition.yaml` aufnehmen
4. `cargo xtask check-error-codes` — muss grün sein
5. `cargo test` — der Test `api_error_codes_are_a_subset_of_all` schlägt
   absichtlich fehl, wenn sich die Menge der CLI-only-Codes ändert, damit das
   eine bewusste Entscheidung bleibt
6. Diese Datei ergänzen
