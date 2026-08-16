# CLI-Argumente

**Flags funktionieren an jeder Position.** Vor dem Input, dahinter, dazwischen.

```bash
kern hello --cipher chaldean --lookup     # empfohlen: erst das Wort, dann die Flags
kern --cipher chaldean hello --lookup     # gleichwertig
kern --cipher chaldean --lookup hello     # gleichwertig
```

Es gibt keine Reihenfolge, die man sich merken muss. Die Empfehlung „Flags ans
Ende" ist Geschmack, keine Regel — sie steht hier, nicht im Parser.

---

## Alle Flags sind global

Ein Flag gilt für den gesamten Aufruf, nie für einen einzelnen Input.

`kern a b --cipher chaldean` wendet Chaldean auf **beide** Wörter an. Es gibt
keine Möglichkeit, einem einzelnen Input im Strom ein eigenes Cipher zu geben.

> Ältere Fassungen von README und `CLAUDE.md` beschrieben „lokale Flags pro
> Pipeline-Position" (`word1 -v word2 -c py,ch word3`). Das war nie
> implementiert: `-v` wurde als Wort reduziert, `-c` existierte nicht einmal als
> Kurzform. Die Beschreibung ist entfernt, die Reste im Parser sind aufgeräumt.

---

## Inputs mit führendem Bindestrich

Sie brauchen `--` als Trenner:

```bash
$ kern -abc
error: unexpected argument '-a' found
  tip: to pass '-a' as a value, use '-- -a'

$ kern -- -abc
{"items":[{"input":"-abc","value":6}]}
```

Das ist der einzige Preis der Positionsfreiheit — und ein bewusster Tausch:
Der Fall scheitert sichtbar mit Lösungshinweis, statt still ein falsches
Ergebnis zu liefern ([PRINCIPLES §1](../PRINCIPLES.md)).

Datums-Ranges sind nicht betroffen: `kern -d -3..0` funktioniert unverändert,
weil `--date` sein eigenes `allow_hyphen_values` hat.

---

## Wie es vorher war

Bis einschließlich v2.0.0 wurden Flags **nur vor dem ersten Input** erkannt.
Danach landeten sie im Positional und wurden als Wörter reduziert:

```
$ kern hello --cipher chaldean          # v2.0.0
{"items":[{"input":"hello",…},{"input":"--cipher","value":5},{"input":"chaldean",…}]}
```

Die Quersumme der Zeichenkette `--cipher` wurde als Ergebnis gemeldet. Kein
Fehler, kein Hinweis. Ausnahme waren `-t/--total` und `-l/--lookup`, die
`parse_pipeline_tokens` von Hand aus dem Token-Strom fischte — weshalb
ausgerechnet diese beiden hinten funktionierten und die CLI inkonsistent statt
bloß streng war.

**Ursache:** `allow_hyphen_values(true)` am `ARGS`-Positional. Damit kann clap
Flags nicht mehr von Werten unterscheiden und hört ab dem ersten Positional auf,
sie zu erkennen. Die Behebung bestand darin, diese eine Zeile zu entfernen und
den handgeschriebenen Sonderfall aufzuräumen.

---

## Warum nicht „Flags nur hinten"

Der ursprüngliche Wunsch war, Flags ans Ende zu *erzwingen*. Als harte Regel
hätte das Kosten ohne Gegenwert: `kern --lang de hello` funktioniert seit jeher
und würde brechen, samt Skripten — und es wäre erneut eine Regel zum Merken,
während das Ziel war, sich nichts merken zu müssen.

---

## Tests

`tests/cli_arguments.rs` ruft die echte Binary auf und prüft jede Position
einzeln. Der Kerntest ist negativ formuliert: taucht ein Flag als `"input"` in
der Ausgabe auf, wurde es verschluckt — das ist die Signatur des alten Fehlers.
