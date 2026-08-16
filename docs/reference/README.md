# Reference

Aufbau-Dokumentation: wie die einzelnen Teile des Projekts funktionieren,
Modul für Modul. Nicht „wie benutze ich KERN" — das steht im
[README](../../README.md) — sondern „wie ist es gebaut und warum".

**Diese Dokumente wandern mit dem Code.** Wer ein Modul ändert, aktualisiert im
selben Zug seine Referenz. Siehe [PRINCIPLES §7](../PRINCIPLES.md).

---

## Vorhanden

| Dokument | Inhalt |
|----------|--------|
| [tooling.md](tooling.md) | `cargo xtask`-Befehle, Versionsregel, Konsistenz-Checks |
| [localization.md](localization.md) | Sprachsystem: `Lang`, Abdeckung pro Inhaltstyp, Ablehnungsregeln |
| [error-codes.md](error-codes.md) | Fehlercodes, Vertrag mit Consumern, CLI/Server-Deckungsgleichheit |
| [cli-arguments.md](cli-arguments.md) | Flag-Positionen, warum alle Flags global sind, `--`-Trenner |
| [phase.md](phase.md) | Fächer, Zyklus, warum die Argumentreihenfolge das Vorzeichen dreht |
| [flow-engine.md](flow-engine.md) | Pipeline, Schritte, der ungeschriebene Reihenfolge-Vertrag |

## Noch offen

Diese Module sind noch nicht dokumentiert. Die Liste ist bewusst hier, statt zu
suggerieren, die Referenz sei vollständig:

- `core/ciphers/` — Cipher-Trait und die 11 Implementierungen
- `core/spektra.rs` — Achsenberechnung (die Sprachseite steht in `localization.md`)
- `ui/` — TTY-Ausgabe, Theming, `is_tty`-Umschaltung
- Der Unterschied zwischen TTY- und Pipe-Modus im CLI

Wer eines dieser Module anfasst, legt die zugehörige Referenz an.
