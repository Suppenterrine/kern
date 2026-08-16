# Release Notes

Eine Datei pro Version, benannt nach der Version in `Cargo.toml`:

```text
docs/release-notes/2.1.0.md
```

Der Inhalt wird **unverändert** zum Body des GitHub-Releases. Es gibt keinen
zweiten Ort, an dem Release Notes stehen — nichts wird beim Veröffentlichen von
Hand getippt.

## Wann schreiben

**Vor dem Release, nicht danach.** `cargo xtask check-release` bricht ab, wenn
die Datei für die aktuelle Version fehlt oder leer ist, und der
Release-Workflow ruft das auf, bevor irgendetwas auf GitHub angelegt wird.

Der Grund ist nicht Ordnungsliebe: nachträglich ließe sich das nur durch
Bearbeiten eines bereits veröffentlichten Releases beheben — sichtbar für alle,
die zwischenzeitlich hingeschaut haben.

## Ablauf

```bash
cargo set-version 2.1.0        # einzige Quelle der Wahrheit
cargo xtask sync-version       # abgeleitete Stellen nachziehen
$EDITOR docs/release-notes/2.1.0.md
cargo xtask check-release      # Note vorhanden? Tag noch frei?
# committen, mergen, dann:
gh workflow run release.yml
```

Der Tag wird aus `Cargo.toml` **abgeleitet** (`v2.1.0`) und nirgends getippt.

## Was hineingehört

Was ein Nutzer merkt. Interne Umbauten gehören in die Commit-Historie, nicht
hierher.

Bei Breaking Changes: **oben**, mit dem alten und dem neuen Aufruf
nebeneinander. Wer aktualisiert, soll in zehn Sekunden sehen, ob es ihn trifft.

Der erste Absatz erscheint in Vorschauen — dort gehört die wichtigste Aussage
hin, nicht „Diverse Verbesserungen".
