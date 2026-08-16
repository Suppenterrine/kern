# TODO

Offene Punkte für KERN. Erledigtes wird entfernt, nicht abgehakt.

---

## Offen

### v2.0.0 auf den Live-Host ziehen

Release v2.0.0 ist veröffentlicht, das Image liegt auf GHCR
(`ghcr.io/suppenterrine/kern-server:v2.0.0` und `:latest`, Digest
`sha256:5dffa5a6…`). **`kern.lukasbaumert.de` läuft aber noch v1.2.0** — der
Release-Workflow pusht das Image, deployt es aber nicht.

Auf dem Host:

```bash
docker pull ghcr.io/suppenterrine/kern-server:v2.0.0
docker stop kern-server && docker rm kern-server
docker run -d -p 3000:3000 --name kern-server ghcr.io/suppenterrine/kern-server:v2.0.0
```

Danach verifizieren:

```bash
curl https://kern.lukasbaumert.de/            # muss version 2.0.0 melden
curl https://kern.lukasbaumert.de/lookup/7    # muss englisch antworten
```

**Consumer vorwarnen:** die Umstellung ist brechend. Aufrufe ohne `lang`
bekommen ab dem Deploy englische statt deutscher Inhalte, und `/` liefert nicht
mehr die Endpunkt-Übersicht (die liegt jetzt auf `/help`).

### Der neue Release-Workflow ist ungetestet

`release.yml` wurde auf `workflow_dispatch` mit Draft-First umgebaut, aber seit
dem Umbau noch nicht ausgeführt — v2.0.0 lief über den alten Weg. Beim nächsten
Release genau hinschauen, besonders auf die Asset-Uploads über die Release-ID.

### Deployment automatisieren

Der letzte Schritt ist Handarbeit auf dem Host — genau die Stelle, an der laut
[PRINCIPLES §4](PRINCIPLES.md) ein Werkzeug stehen sollte. Solange das so ist,
kann Live und Release auseinanderlaufen, ohne dass es jemand merkt.

Mindestens: ein Health-Check, der die live gemeldete Version gegen den neuesten
Release-Tag prüft.

### CLI: Flags an jeder Position erlauben

Heute werden Flags **nach** dem ersten Input stillschweigend als Wörter
reduziert — `kern hello --cipher chaldean` berechnet die Quersumme von
„--cipher". Kein Fehler, nur ein falsches Ergebnis. Ausnahme sind `-t` und
`-l`, was die Sache inkonsistent statt bloß streng macht.

Analyse, Messwerte und der Ein-Zeilen-Fix stehen in
[proposals/cli-argument-order.md](proposals/cli-argument-order.md). Vorher zu
entscheiden: ob „lokale Flags pro Pipeline-Position" aufgegeben werden (dort
Variante A, empfohlen) oder tatsächlich gebaut werden sollen.

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
