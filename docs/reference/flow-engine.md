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

## Aufgeräumt in v4.0.0

Die Engine hatte vier Stellen, die nach Funktionalität aussahen, aber keine
hatten. Sie sind entfernt — das Verhalten hat sich dadurch nicht geändert, es
steht nur weniger da, das nicht stimmt:

| Entfernt | War |
|----------|-----|
| `Operation::Custom(String)` | wurde nirgends erzeugt, der Zweig war leer |
| `_step` in `select_ciphers` | Parameter wurde nie gelesen; Auswahl ist global |
| `cipher_index` in `Step::new` | von `run()` überschrieben, bevor ihn jemand las |
| verworfenes `ResultSet` im SPEKTRA-Pfad | Rückgabewert weggeworfen zugunsten von `ctx.memory` |

`Step::new` nimmt seitdem `(pipe_index, operation)`. Der `cipher_index` bleibt
im `Step` — `run()` setzt ihn je Chiffre —, ist aber kein Konstruktor-Argument
mehr.

## Was noch offen ist

Der **Reihenfolge-Vertrag** ist weiterhin ungeschrieben und ungeprüft:
`AggregateTotal` und `Lookup` müssen nach den Reduce-Schritten eingefügt
werden. Wer sie davor anhängt, bekommt eine leere `memory` und stillschweigend
0 bzw. eine leere Liste.

Möglich wäre, dass `Pipeline` die Nachbearbeitungsschritte selbst ans Ende
sortiert, statt es den Aufrufern zu überlassen. Bis dahin steht der Vertrag
zumindest hier.

## Wer Pipelines baut

| Ort | Schritte |
|-----|----------|
| `main.rs` Datums-Modus | `DateReduce` je Offset, optional `Lookup` |
| `main.rs` SPEKTRA | `Reduce` + `Lookup` |
| `main.rs` Normalmodus | `Reduce` je Input, optional `AggregateTotal`, optional `Lookup` |
| `kern-server.rs` SPEKTRA | `Reduce` + `Lookup` |
| `kern-server.rs` Phase | `PhaseRelation` je Paar |
| `kern-server.rs` `/reduce` | `Reduce` je Input, optional `AggregateTotal` |

Seit v4.0.0 läuft auch `/reduce` über die Engine. Vorher rechnete der Endpunkt
direkt, mit einer **privaten Kopie** der Reduktionsroutine — weshalb CLI und
Server sich beim `chain`-Format und bei dem, worüber ein `total` summiert,
widersprachen (Issue #23). Die Kopie ist entfernt.
