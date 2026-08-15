# TODO

Offene Punkte für KERN. Erledigtes wird entfernt, nicht abgehakt.

---

## Offen

### v2.0.0 deployen

Version ist auf 2.0.0 gesetzt (`cargo set-version` + `cargo xtask sync-version`),
aber noch nicht deployed. Live läuft weiterhin v1.2.0.

**Breaking Changes für Consumer:**

- Standardsprache der API von Deutsch auf **Englisch** gewechselt — bestehende
  Aufrufe ohne `lang` bekommen jetzt englische Inhalte
- `/` liefert nur noch einen schlanken Service-Deskriptor; die vollständige
  Übersicht liegt auf `/help`
- Fehlerantworten haben ein zusätzliches `code`-Feld (additiv, nicht brechend)

**Das Gate läuft automatisch.** `release.yml` prüft vor jedem Upload Tests,
Versions-Konsistenz, Fehlercodes und ob der Release-Tag zur Crate-Version passt.
Schlägt es fehl, wird nichts veröffentlicht.

Lokal vorab prüfbar:

```bash
cargo test
cargo xtask check
cargo xtask check-tag v2.0.0
```

Zum Ausliefern: Release mit Tag `v2.0.0` und `[SERVER]` im Body anlegen — der
Workflow baut die Binaries und pusht das Docker-Image nach GHCR. Danach
`kern.lukasbaumert.de` auf das neue Image ziehen.

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

### Stale-Artefakte in `./target/` aufräumen

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
