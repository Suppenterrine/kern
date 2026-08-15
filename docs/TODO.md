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

Offen: Docker-Image bauen, nach GHCR pushen, `kern.lukasbaumert.de` aktualisieren.

### CLI-Fehler ohne `code`-Feld

Die API liefert `{"code": "...", "error": "..."}`, das CLI im Pipe-Modus nur
`{"error": "..."}`. Wer beide Wege konsumiert, muss zwei Formate behandeln.
Das CLI sollte dieselben Codes mitgeben.

### Fehler-Codes gegen die Spec testen

Code und Spec stimmen aktuell exakt überein (12 Codes), aber nichts erzwingt das.
Der Abgleich gehört als weiterer `xtask`-Check neben `sync-version`, damit er
wie die Versionen deterministisch statt per Sorgfalt läuft (PRINCIPLES §4).

Manueller Abgleich bis dahin:

```bash
python -c "
import re,yaml
emitted=set(re.findall(r'(?:bad_request|server_error)\(\s*\"([a-z_]+)\"',
            open('src/bin/kern-server.rs',encoding='utf-8').read()))
spec=yaml.safe_load(open('api/kern.definition.yaml',encoding='utf-8'))
doc=set(spec['components']['schemas']['ErrorResponse']['properties']['code']['enum'])
print('OK' if emitted==doc else f'DRIFT: {emitted^doc}')"
```

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
