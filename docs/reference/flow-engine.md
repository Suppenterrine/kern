# Flow-Engine

`src/core/flow.rs`, 266 Zeilen. Die Schicht zwischen „Eingaben und Flags" und
„Ergebnissen".

---

## Wie sie funktioniert

Drei Typen:

| Typ | Inhalt |
|-----|--------|
| `Step` | `pipe_index`, `cipher_index`, `operation`, optionale `metadata` |
| `Pipeline` | ein `Vec<Step>` |
| `FlowContext` | globale Flags, `memory: Vec<KernResult>`, `phase_results` |

`Pipeline::run` läuft die Schritte **der Reihe nach** durch und verzweigt auf
`step.operation`:

| Operation | Tut |
|-----------|-----|
| `Reduce` / `DateReduce` | reduziert `inputs[step.pipe_index]` mit jeder gewählten Chiffre |
| `AggregateTotal` | summiert alle bisherigen Reduce-Ergebnisse aus `ctx.memory` |
| `Lookup` | gruppiert alle bisherigen Werte aus `ctx.memory` nach Zahl, legt sie als JSON-Payload ab |
| `PhaseRelation` | reduziert zwei Inputs und berechnet ihre Phase |
| `Custom(String)` | **nichts** |

Jedes Ergebnis wandert in `ctx.memory` *und* in das zurückgegebene `ResultSet`.

---

## Der eigentliche Ablauf

Trotz des Namens ist es keine Graph-Ausführung, sondern eine feste Abfolge in
drei Phasen:

```
Reduce(input₀) … Reduce(inputₙ)   →   [AggregateTotal]   →   [Lookup]
                      ↓                      ↑                   ↑
                  ctx.memory ────────────────┴───────────────────┘
```

**Schritte reden nicht miteinander.** Sie schreiben alle in dieselbe flache
Liste `ctx.memory`, und `AggregateTotal` bzw. `Lookup` durchsuchen diese Liste
nachträglich. Es gibt keine Verbindung von Schritt zu Schritt, keine
Verzweigung, keine Bedingungen.

### Daraus folgt: die Reihenfolge ist ein ungeschriebener Vertrag

`AggregateTotal` und `Lookup` müssen **nach** den Reduce-Schritten eingefügt
werden, sonst finden sie eine leere `memory` und liefern stillschweigend 0 bzw.
eine leere Liste. Nichts erzwingt oder prüft das — es hängt daran, dass alle
Aufrufer die Schritte in der richtigen Reihenfolge anhängen.

---

## Was verspricht, was es nicht hält

Die Abstraktion ist erkennbar für mehr entworfen worden, als sie tut:

- **`Operation::Custom(String)`** — wird nirgends erzeugt und tut im Rumpf
  nichts. Ein Platzhalter ohne Inhalt.
- **`select_ciphers(&self, _step, …)`** — der `_step`-Parameter wird nicht
  benutzt. Die Chiffren-Auswahl kommt ausschließlich aus den globalen Flags.
  Gedacht war offensichtlich eine Auswahl **pro Schritt**; das war die Grundlage
  der „lokalen Flags", die es nie gab (siehe [cli-arguments.md](cli-arguments.md)).
- **`Step::cipher_index`** — der beim Anlegen übergebene Wert ist bedeutungslos.
  `run()` überschreibt ihn (`ctx_step.cipher_index = cipher_index`), bevor er
  irgendwo gelesen wird. Alle Aufrufer übergeben deshalb `0`.
- **Der Rückgabewert `ResultSet`** — im SPEKTRA-Pfad des Servers wird er
  verworfen (`let _result_set = …`) und stattdessen `ctx.memory` gelesen. Es
  gibt also zwei Wege an dieselben Daten, und der Code benutzt beide.

---

## Einschätzung

Für das, was tatsächlich passiert — eine lineare Kette aus Reduktionen mit zwei
optionalen Nachbearbeitungen — ist das mehr Maschinerie als nötig. Der Preis
ist nicht Laufzeit, sondern Lesbarkeit: Wer `Custom` oder `cipher_index` sieht,
nimmt an, dass es etwas bedeutet.

Ein vollständiger Umbau wäre allerdings viel Risiko für wenig Gewinn — die
Engine funktioniert. Der ehrlichere Schnitt wäre, ihr das Versprechen zu
nehmen, das sie nicht einlöst:

1. `Operation::Custom` entfernen (nichts erzeugt es)
2. `_step` aus `select_ciphers` entfernen oder die Auswahl pro Schritt wirklich
   implementieren — je nachdem, ob sie gewollt ist
3. `cipher_index` aus `Step::new` nehmen, da `run()` ihn ohnehin setzt
4. Den Reihenfolge-Vertrag entweder dokumentieren oder erzwingen, z. B. indem
   `Pipeline` die Nachbearbeitungsschritte selbst ans Ende sortiert

Das ist noch nicht gemacht — es steht als offener Punkt in
[TODO.md](../TODO.md) und in Issue #23.

---

## Wer Pipelines baut

| Ort | Schritte |
|-----|----------|
| `main.rs` Datums-Modus | `DateReduce` je Offset, optional `Lookup` |
| `main.rs` SPEKTRA | `Reduce` + `Lookup` |
| `main.rs` Normalmodus | `Reduce` je Input, optional `AggregateTotal`, optional `Lookup` |
| `kern-server.rs` SPEKTRA | `Reduce` + `Lookup` |
| `kern-server.rs` Phase | `PhaseRelation` je Paar |

Der `/reduce`-Endpunkt des Servers benutzt die Engine **gar nicht** — er
rechnet direkt in `reduce_number_steps_with_cipher`. Das ist einer der Gründe,
warum CLI und Server bei `reduce` unterschiedlich viel ausgeben (Issue #23).
