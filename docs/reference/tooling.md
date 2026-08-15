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
| `cargo xtask check` | Führt alle Checks aus, schreibt nichts. Läuft als CI-Gate. |
| `cargo xtask sync-version` | Schreibt die `Cargo.toml`-Version in alle abgeleiteten Dateien |
| `cargo xtask sync-version --check` | Meldet Drift, schreibt nichts, Exit 1 bei Abweichung |
| `cargo xtask check-error-codes` | Vergleicht `ErrorCode::API` mit der OpenAPI-Spec |
| `cargo xtask check-tag <TAG>` | Prüft, ob ein Release-Tag zur `Cargo.toml`-Version passt |
| `cargo xtask bump version <major\|minor\|patch>` | Bumpt `Cargo.toml` und synchronisiert danach |

`check` sammelt **alle** Fehlschläge und meldet sie gemeinsam, statt beim ersten
abzubrechen — ein Durchlauf sagt dir alles, was zu tun ist.

`check-tag` ist nicht Teil von `check`, weil es ein Argument braucht; es läuft
nur im Release-Workflow.

---

## In der CI

| Workflow | Job | Wann | Inhalt |
|----------|-----|------|--------|
| `rust.yml` | `build` | Push auf `master`, PRs nach `master` | `cargo build`, `cargo test` |
| `rust.yml` | `consistency` | dito | `cargo xtask check` |
| `release.yml` | `consistency` | veröffentlichtes Release | `cargo xtask check`, `check-tag`, `cargo test` |

Im Release-Workflow ist `consistency` ein **Gate**: `build-win` hängt per
`needs` daran, `build-linux` wiederum an `build-win`. Schlägt das Gate fehl,
werden weder Binaries hochgeladen noch ein Docker-Image nach GHCR gepusht.

Der Tag-Check ist dabei der wichtigste: ohne ihn kann ein als `v2.0.0`
getaggtes Release Binaries ausliefern, die sich als `1.2.0` melden — dieselbe
Drift, die `sync-version` innerhalb des Repos verhindert, nur an der
Release-Grenze.

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

- `xtask` deckt Versionen, Fehlercodes und den Release-Tag ab. Andere
  Konsistenzfragen prüft es nicht — etwa ob die README-Beispielausgaben noch
  zur tatsächlichen Ausgabe passen, oder ob die Endpunkt-Beschreibungen in
  `/help` mit der OpenAPI-Spec übereinstimmen.
- `rust.yml` läuft nur auf `master` und auf PRs **nach** `master`. Pushes auf
  Feature-Branches ohne offenen PR werden nicht geprüft.
