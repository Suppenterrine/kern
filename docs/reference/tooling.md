# Tooling: `cargo xtask`

`xtask` ist ein Hilfsprogramm im Workspace (`xtask/`), aufgerufen über den Alias
in `.cargo/config.toml`:

```toml
[alias]
xtask = "run -p xtask --"
```

Es übernimmt die wiederkehrende Konsistenzarbeit, die sonst von Sorgfalt
abhängen würde — siehe [PRINCIPLES §4](../PRINCIPLES.md).

---

## Befehle

| Befehl | Wirkung |
|--------|---------|
| `cargo xtask check` | Führt alle Checks aus, schreibt nichts. Als CI-Gate gedacht. |
| `cargo xtask sync-version` | Schreibt die `Cargo.toml`-Version in alle abgeleiteten Dateien |
| `cargo xtask sync-version --check` | Meldet Drift, schreibt nichts, Exit 1 bei Abweichung |
| `cargo xtask check-error-codes` | Vergleicht `ErrorCode::API` mit der OpenAPI-Spec |
| `cargo xtask bump version <major\|minor\|patch>` | Bumpt `Cargo.toml` und synchronisiert danach |

`check` sammelt **alle** Fehlschläge und meldet sie gemeinsam, statt beim ersten
abzubrechen — ein Durchlauf sagt dir alles, was zu tun ist.

---

## Die Versionsregel

**Versionsnummern werden niemals von Hand editiert.**

`Cargo.toml` ist die einzige Quelle der Wahrheit. Jede andere Fundstelle ist
eine *abgeleitete Kopie* und wird geschrieben, nicht gepflegt.

```bash
cargo set-version 2.1.0     # Quelle der Wahrheit ändern (cargo-edit)
cargo xtask sync-version    # abgeleitete Kopien nachziehen
```

`cargo set-version` allein reicht **nicht** — es kennt nur `Cargo.toml` und
fasst weder README noch OpenAPI-Spec an. Der zweite Schritt ist Pflicht.

### Warum das nötig ist

Vor Einführung des Tools stand die Version an drei Stellen auf drei Werten:

| Datei | Wert |
|-------|------|
| `Cargo.toml` | 1.2.0 |
| `README.md` | 1.1.2 |
| `api/kern.definition.yaml` | 1.0.0 |

Das alte `bump version` war zudem **nicht atomar**: es schrieb `Cargo.toml`,
scheiterte dann am README und hinterließ ein halb aktualisiertes Repository.

`sync-version` löst deshalb erst *alle* Ziele auf und schreibt erst danach.
Fehlt eine Datei oder passt ein Muster nicht, wird gar nichts angefasst und
alle Probleme werden gemeinsam gemeldet.

### Eine neue abgeleitete Stelle hinzufügen

In die `DERIVED`-Tabelle in `xtask/src/main.rs`:

```rust
DerivedVersion {
    path: "pfad/zur/datei",
    pattern: r#"..."#,      // Version in Gruppe 1
    template: "...{version}...",  // ${1} usw. für erhaltene Gruppen
    what: "Beschreibung für Fehlermeldungen",
}
```

Nicht: die Datei von Hand pflegen.

---

## Der Fehlercode-Check

`check-error-codes` vergleicht die Codes, die die HTTP-API zurückgeben kann,
mit dem `enum` unter `components.schemas.ErrorResponse.properties.code` in
`api/kern.definition.yaml`.

Entscheidend: die erwartete Menge kommt aus **`ErrorCode::API` in der
Bibliothek**, nicht aus einem Regex über den Server-Quelltext. `xtask` hängt
dafür per `path`-Dependency am `kern`-Crate. Der Check prüft also, was der Code
tatsächlich ausgeben *kann*, nicht was zufällig im Text steht.

Er meldet beide Richtungen:

```
DRIFT   emitted but not in the spec: word_missing
DRIFT   in the spec but never emitted: erfundener_code
```

Details zu den Codes selbst: [error-codes.md](error-codes.md).

---

## Grenzen

- `xtask` deckt Versionen und Fehlercodes ab. Andere Konsistenzfragen (etwa ob
  die README-Beispielausgaben noch zur echten Ausgabe passen) prüft es nicht.
- `check` läuft noch nicht automatisch in CI. Solange das so ist, ist es ein
  manueller Schritt vor dem Release.
