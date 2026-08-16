# TODO

Offene Punkte für KERN. Erledigtes wird entfernt, nicht abgehakt.

---

## Offen

### Flow-Engine entrümpeln

Die Engine verspricht mehr, als sie tut — Details in
[reference/flow-engine.md](reference/flow-engine.md). Konkret:

- `Operation::Custom` wird nirgends erzeugt und tut nichts
- `select_ciphers` ignoriert seinen `_step`-Parameter (Auswahl pro Schritt war
  gedacht, kam nie)
- `Step::cipher_index` wird beim Anlegen übergeben, aber von `run()`
  überschrieben, bevor er gelesen wird
- Der Reihenfolge-Vertrag (Nachbearbeitung muss nach den Reduce-Schritten
  eingefügt werden) ist ungeschrieben und ungeprüft

Kein Umbau, sondern Wegnehmen. Teil von Issue #23.

### Referenz-Dokumentation vervollständigen

`docs/reference/` deckt bisher Tooling, Lokalisierung und Fehlercodes ab. Offen
sind Flow-Engine, Chiffren, Phase, SPEKTRA-Achsen, UI und der
TTY-/Pipe-Unterschied — die Liste steht in `docs/reference/README.md`.

Nicht als Block nachziehen: Wer eines dieser Module anfasst, dokumentiert es
dabei (PRINCIPLES §7).

### `CARGO_TARGET_DIR` global entfernen

`.cargo/config.toml` setzt `build.target-dir = "target"`, greift aber nicht,
solange die User-Umgebungsvariable `CARGO_TARGET_DIR=D:\rust-target` existiert —
Cargo rankt Umgebungsvariablen über Konfigurationsdateien. Wird von anderer
Stelle bearbeitet.

Nebenwirkung bis dahin: `./target/` enthält veraltete Binaries vom 19. Juli,
während echte Builds nach `D:\rust-target\` gehen. Beim Testen von Binaries auf
den Pfad achten.

### Stale-Artefakte in `./target/` aufräumen (erledigt, Notiz)

Bereits aufgeräumt: `./target/` enthält nur noch die beiden Release-Binaries.
Sobald `CARGO_TARGET_DIR` verschwindet, füllt sich das Verzeichnis wieder
normal.

Sobald der Target-Pfad zurückgestellt ist, sollten die alten Artefakte weg,
sonst liegen dort weiterhin irreführende Binaries.

---

## Bewusst nicht geplant

Siehe auch [PRINCIPLES.md](PRINCIPLES.md).

- **Französische Prompts.** SPEKTRA und RTAP gibt es nur auf Englisch und
  Deutsch. `lang=fr` wird auf diesen Endpunkten mit `language_not_available`
  abgelehnt — **kein** Fallback auf Englisch (PRINCIPLES §1). Nur die
  Zahlenbedeutungen sind dreisprachig. Wenn französische Prompts gewünscht sind,
  ist das ein eigener Punkt unter „Offen", keine stillschweigende Ersetzung.
- **Lokalisierte Fehlermeldungen.** Fehler bleiben immer Englisch. `lang`
  steuert Inhalte, nicht das Protokoll (PRINCIPLES §2).
- **Key-basierter i18n-Katalog / Framework.** Die Bedeutungen sind ein
  übersetzter Datensatz, kein UI-String-Katalog — der Key ist bereits die Zahl.
  Pro-Sprache-YAML bleibt das Format.
