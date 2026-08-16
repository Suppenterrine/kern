# Vorschlag: Flags an jeder Position erlauben

**Status:** Vorschlag, nicht umgesetzt. Beschreibt einen möglichen künftigen
Zustand, nicht den aktuellen.

**Betrifft:** nur die CLI. Der Server hat benannte Query-Parameter — dort gibt
es keine Reihenfolge und nichts zu ändern.

---

## Das Problem

Der Wunsch war: erst `kern`, dann das Wort, dann fällt einem ein, welche Flags
man noch will — also hinten anhängen. Das entspricht der Denkreihenfolge, nicht
der Unix-Konvention „Optionen zuerst".

Der heutige Zustand ist aber nicht bloß „Unix-konform". Er ist **inkonsistent**,
und er scheitert **still**.

### Gemessenes Verhalten (v2.0.0)

| Aufruf | Ergebnis |
|--------|----------|
| `kern --cipher chaldean hello` | ✅ Chaldean angewandt → 5 |
| `kern hello --cipher chaldean` | ❌ `--cipher` und `chaldean` werden **als Wörter reduziert** |
| `kern hello --lang de -l` | ❌ `--lang` ignoriert, Antwort bleibt englisch |
| `kern hello --verbose` | ❌ `--verbose` wird als Wort reduziert |
| `kern hello -L` | ❌ `-L` wird als Wort reduziert (Wert 3) |
| `kern hello --lookup` | ✅ funktioniert |
| `kern hello -t` | ✅ funktioniert |

Zwei Flags verhalten sich also genau andersherum als alle übrigen. Und der
Fehlerfall ist der schlimmstmögliche: **kein Fehler**, sondern eine
plausibel aussehende Zahl, die aus dem Wort „--cipher" berechnet wurde.

Das verstößt gegen [PRINCIPLES §1](../PRINCIPLES.md): keine stillen Fallbacks.
Hier ist es sogar schärfer als ein Fallback — es ist ein falsches Ergebnis ohne
jeden Hinweis.

### Warum es so ist

Zwei Ursachen, die sich überlagern:

1. **`allow_hyphen_values(true)` am `ARGS`-Positional** (`src/main.rs`). Damit
   akzeptiert das Positional Werte mit führendem Bindestrich — und weil clap
   dann Flags nicht mehr von Werten unterscheiden kann, hört es ab dem ersten
   Input auf, Flags zu erkennen. Alles danach ist Input.

2. **`parse_pipeline_tokens` fischt zwei Flags von Hand heraus**, nämlich
   `-t/--total` und `-l/--lookup`. Nur deshalb funktionieren ausgerechnet diese
   beiden hinten. Ein handgeschriebener Sonderfall, der die Inkonsistenz
   erzeugt, statt sie zu beheben.

### Nebenbefund: dokumentierte Funktion existiert nicht

`CLAUDE.md` und das README beschreiben „lokale Flags pro Pipeline-Position":

```bash
kern input1 input2 -c chaldean input3     # laut README
kern word1 -v word2 -c py,ch word3        # laut CLAUDE.md
```

**Das gibt es nicht.** Gemessen:

```
$ kern a -v b
{"items":[{"input":"a","value":1},{"input":"-v","value":22},{"input":"b","value":2}]}
```

`-v` wird als Wort reduziert. `parse_pipeline_tokens` kennt nur `-t` und `-l`
und bindet auch die nicht an eine Position, sondern setzt globale Schalter.
Zudem ist `-c` nirgends als Kurzform definiert — nur `--cipher` existiert.

Diese Doku-Stellen sind unabhängig von diesem Vorschlag zu korrigieren.

---

## Der Vorschlag

**Flags an jeder Position erlauben**, statt sie nach hinten zu *erzwingen*.

```bash
kern hello --cipher chaldean --lookup     # gewünschter Rhythmus
kern --cipher chaldean hello              # funktioniert weiterhin
```

### Warum „überall" und nicht „nur hinten"

Der Wunsch war „alle Flags nach hinten". Als **harte Regel** hätte das Kosten
ohne Gegenwert:

- `kern --lang de hello` funktioniert heute und würde brechen — inklusive
  bestehender Skripte und Muskelgedächtnis.
- Es wäre erneut eine Sonderregel, die man sich merken muss. Das Ziel war,
  sich *nichts* merken zu müssen.
- Erzwingen bringt keinen technischen Vorteil. Es verhindert keinen Fehler.

„Überall erlaubt" liefert den gewünschten Rhythmus vollständig, ohne etwas
wegzunehmen. Die Empfehlung „Flags ans Ende" gehört dann in die Doku, nicht in
den Parser.

---

## Umsetzbarkeit

**Es ist eine Zeile.** Verifiziert an einem Experiment-Build von v2.0.0:

```diff
  Arg::new("ARGS")
      .num_args(1..)
-     .allow_hyphen_values(true)
      .help("Input strings to be reduced"),
```

Danach gemessen:

| Aufruf | vorher | nachher |
|--------|--------|---------|
| `kern hello --cipher chaldean` | „--cipher" als Wort | ✅ Chaldean → 5 |
| `kern hello --lang de -l` | englisch | ✅ deutsch |
| `kern hello --verbose` | „--verbose" als Wort | ✅ Rechenweg |
| `kern hello -L` | „-L" als Wort | ✅ Länge 5 |
| `kern --cipher chaldean hello` | ✅ | ✅ unverändert |
| `kern -d -3..0` | ✅ | ✅ unverändert |

Negative Datums-Offsets bleiben intakt, weil `-d` sein **eigenes**
`allow_hyphen_values` hat, das unangetastet bleibt.

### Was zu beachten ist

**1. Inputs mit führendem Bindestrich brauchen `--`.**

Das ist der gesamte Preis der Änderung:

```
$ kern -abc
error: unexpected argument '-a' found
  tip: to pass '-a' as a value, use '-- -a'

$ kern -- -abc
{"items":[{"input":"-abc","value":6}]}
```

Bemerkenswert: das ist ein **klarer Fehler mit Lösungshinweis** statt eines
stillen Falschergebnisses. Gemessen an PRINCIPLES §1 ist das eine Verbesserung,
selbst wenn man den Anwendungsfall verliert. Für numerologische Eingaben —
Wörter, Namen, Daten — dürfte ein führender Bindestrich ohnehin die Ausnahme
sein.

**2. Der Sonderfall in `parse_pipeline_tokens` wird überflüssig.**

Danach behandelt clap `-t` und `-l` wie alle anderen Flags. Die Zweige in
`parse_pipeline_tokens` können weg; `saw_total`/`saw_lookup` bleiben dann
dauerhaft `false`, und `show_total = show_total || parsed.saw_total` liefert
weiterhin das richtige Ergebnis. Aufräumen ist trotzdem angebracht, sonst
bleibt toter Code stehen, der eine Funktion vortäuscht.

**3. Der Vorschlag löst die Pipeline-Frage nicht.**

Die eigentliche Idee hinter der Reduktionspipeline — Flags, die an *eine
bestimmte Position* im Input-Strom binden — ist nicht umgesetzt und wäre mit
clap-Standardparsing auch nicht umsetzbar, weil clap die Reihenfolge zwischen
Flags und Positionals nicht bewahrt.

Das ist eine **eigene Entscheidung**, die vorher fallen sollte:

- **Variante A — lokale Flags aufgeben.** Alle Flags sind global. Die Doku wird
  ehrlich, der Parser einfacher, dieser Vorschlag greift unverändert.
- **Variante B — lokale Flags wirklich bauen.** Dann übernimmt ein eigener
  Parser `std::env::args()` vollständig, und clap entfällt für den Input-Strom.
  Deutlich mehr Arbeit, und die Frage „wo steht welcher Parameter" wird damit
  wieder schwerer statt leichter — also gegen das ursprüngliche Ziel.

Empfehlung: **A**. Der Wunsch war weniger Merkarbeit, nicht mehr.

**4. Tests fehlen.**

Für das Argument-Parsing gibt es derzeit keine Tests. Eine Änderung daran
sollte welche mitbringen — jede Zeile der Tabelle oben ist ein Testfall.

---

## Wenn umgesetzt

1. Die Zeile entfernen
2. `-t`/`-l`-Sonderfall aus `parse_pipeline_tokens` entfernen
3. Tests für die Positionsfälle ergänzen
4. Die falschen „lokale Flags"-Abschnitte in `CLAUDE.md` und README korrigieren
5. `docs/reference/cli-arguments.md` anlegen — die Regel gehört in die Referenz,
   sobald sie gilt
6. Release Note: Inputs mit führendem Bindestrich brauchen jetzt `--`
