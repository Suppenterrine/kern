# Phasenrelationen

Die Phasenrelation beschreibt, wie zwei reduzierte Werte zueinander stehen —
nicht als Abstand, sondern als **Bewegungsrichtung** in einem Dreierzyklus.

---

## Fächer (Compartments)

Jeder reduzierte Wert fällt in eines von drei Fächern:

| Fach | Werte | Masterzahl |
|------|-------|------------|
| 1 | 1, 4, 7 | 11 |
| 2 | 2, 5, 8 | 22 |
| 3 | 3, 6, 9 | 33 |

Implementiert in `calculate_compartment` (`src/core/phase.rs`).

---

## Der Zyklus

Die Fächer bilden einen Ring, keine Skala:

```
        1
       ↗ ↘
      3 ← 2
```

| Phase | Bedeutung | Übergänge |
|-------|-----------|-----------|
| `0` | gleiches Fach, synchron | 1→1, 2→2, 3→3 |
| `+1` | vorwärts | 1→2, 2→3, **3→1** |
| `-1` | rückwärts | 2→1, 3→2, **1→3** |

Wichtig sind die fett markierten Fälle: 3→1 ist **vorwärts**, weil der Ring
sich schließt, und 1→3 ist rückwärts. Wer die Fächer als Zahlenstrahl liest,
erwartet hier das Gegenteil.

---

## Die Reihenfolge bestimmt die Perspektive

**Ein Tausch der Argumente kehrt das Vorzeichen um.** Das ist kein Fehler,
sondern folgt daraus, dass die Relation gerichtet ist:

```bash
$ kern --prm feldmann lukas
feldmann+lukas = +1 (3→1) [ordinal]

$ kern --prm lukas feldmann
lukas+feldmann = -1 (1→3) [ordinal]
```

Gelesen wird immer **vom ersten zum zweiten Argument**: „von feldmann aus
betrachtet liegt lukas eine Position vorwärts". Dieselbe Konstellation, zwei
Blickrichtungen.

Wer eine richtungsunabhängige Aussage braucht, betrachtet den Betrag: `0`
bedeutet gleiches Fach, `1` bedeutet benachbartes Fach — unabhängig davon,
welches Wort zuerst stand.

Für die Matrix mit mehr als zwei Inputs gilt dasselbe paarweise; die Paare
werden von `generate_matrix_pairs` in Eingabereihenfolge gebildet.

---

## Die Fach-Anzeige

Im TTY-Modus zeigt jede Zeile, wo der Wert im Fächersystem sitzt:

```
lukas  [147] [258] [369]
              ↑ unterstrichen ist der eigene Wert
```

Alle neun Ziffern werden immer vollständig gedruckt; nur die zutreffende ist
unterstrichen. Masterzahlen erscheinen als ihre Kennziffer: 11 → `1`,
22 → `2`, 33 → `3`.

> Bis v2.0.0 war diese Anzeige falsch. Die Ziffer wurde in einen festen Platz
> im Fach gespleißt (`"36" + Wert` für Fach 3), was nur zutraf, wenn der Wert
> zufällig an dieser Stelle stand. Ein Wert von 6 erschien als `[366]` statt
> `[369]` — falsch bei sechs der neun möglichen Werte (Issue #21). Der
> Regressionstest steht in `src/ui/output.rs`.

---

## Einschränkungen

`--total` und `--lookup` sind im Phasenmodus nicht unterstützt und werden mit
`invalid_arguments` abgelehnt — an jeder Position, an der man sie tippt.

Der Modus braucht mindestens zwei Inputs (`insufficient_inputs`).
