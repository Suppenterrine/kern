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
| `cargo xtask check-release` | Release Note vorhanden? Tag noch frei? |
| `cargo xtask bump version <major\|minor\|patch>` | Bumpt `Cargo.toml` und synchronisiert danach |

`check` sammelt **alle** Fehlschläge und meldet sie gemeinsam, statt beim ersten
abzubrechen — ein Durchlauf sagt dir alles, was zu tun ist.

`check-release` ist bewusst **nicht** Teil von `check`: während der Entwicklung
hat die nächste Version noch keine Release Note, und eine Prüfung, die bei
jedem PR fehlschlägt, gewöhnt man sich ab zu lesen.

---

## Release-Ablauf

Ausgelöst wird über `workflow_dispatch`, nicht durch ein von Hand angelegtes
Release:

```bash
cargo set-version 2.1.0        # einzige Quelle der Wahrheit
cargo xtask sync-version       # abgeleitete Stellen nachziehen
$EDITOR docs/release-notes/2.1.0.md
cargo xtask check-release      # vorab prüfen
# committen, mergen, dann:
gh workflow run release.yml
```

Der Workflow:

```
create-release   Gate (check, check-release, tests) → Draft anlegen
    ├─ build-win      kern.exe, kern-server.exe → Draft
    └─ build-linux    kern-server → Draft, Docker → GHCR
publish-release  Artefakte wirklich da? → veröffentlichen
```

Zwei Eigenschaften, die den Unterschied machen:

**Der Tag wird abgeleitet, nicht getippt.** Die Version kommt aus `Cargo.toml`,
der Tag ist `v<version>`. Eine Abweichung zwischen Tag und ausgelieferter
Version ist damit nicht mehr erkennbar-aber-möglich, sondern unmöglich. Deshalb
gibt es auch kein `check-tag` mehr — es könnte nur noch tautologisch bestehen.

**Veröffentlicht wird zuletzt.** Schlägt ein Build fehl, läuft
`publish-release` nie und das Release bleibt ein Draft: sichtbar für den, der
nachschaut, unsichtbar für alle anderen. Vorher war es umgekehrt — das Release
war öffentlich, bevor der erste Build lief, sodass ein fehlgeschlagener
Docker-Push ein unvollständiges Release unter deinem Namen hinterließ.

`publish-release` prüft zusätzlich, dass die drei Artefakte tatsächlich am
Release hängen und nicht leer sind. Ein Job kann Erfolg melden, ohne dass sein
Upload angekommen ist.

---

## In der CI

| Workflow | Job | Wann | Inhalt |
|----------|-----|------|--------|
| `rust.yml` | `build` | Push auf `master`, PRs nach `master` | `cargo build`, `cargo test` |
| `rust.yml` | `consistency` | dito | `cargo xtask check` |
| `release.yml` | `create-release` | manuell ausgelöst | `check`, `check-release`, Tests, Draft |
| `release.yml` | `publish-release` | nach allen Builds | Artefakt-Prüfung, Veröffentlichung |

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
