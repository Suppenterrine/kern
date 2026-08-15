# Lokalisierung

Übersetzt werden **Inhalte**: Zahlenbedeutungen und LLM-Prompts. Berechnungen,
Chiffren-Namen und Fehlermeldungen sind nicht übersetzt.

---

## `Lang`

Der Sprachtyp in `src/lib.rs`:

```rust
pub enum Lang { De, #[default] En, Fr }
```

**Englisch ist der Standard.** Die API ist eine internationale Schnittstelle;
Deutsch braucht ein explizites `lang=de`.

`Lang::parse` wertet nur den primären Subtag aus, ohne Groß-/Kleinschreibung:
`en-US`, `en_GB` und `EN` ergeben alle `En`. Damit lässt sich ein roher
`Accept-Language`-Wert direkt durchreichen.

---

## Abdeckung

Nicht jeder Inhalt existiert in jeder Sprache:

| Inhalt | `en` | `de` | `fr` | Quelle |
|--------|:----:|:----:|:----:|--------|
| Zahlenbedeutungen | ✅ | ✅ | ✅ | `bedeutungen.en.yaml`, `bedeutungen.yaml`, `bedeutungen.fr.yaml` |
| SPEKTRA-Prompt | ✅ | ✅ | ❌ | `spektra_prompt.en.txt`, `spektra_prompt.txt` |
| RTAP-Prompts | ✅ | ✅ | ❌ | `rtap_*`-Keys in den Bedeutungsdateien |
| Fehlermeldungen | ✅ | — | — | im Code, immer Englisch |

Alle Dateien werden per `include_str!` zur Compilezeit eingebettet — die
Binaries sind eigenständig, zur Laufzeit kann kein Dateizugriff fehlschlagen.

---

## Keine stillen Fallbacks

Eine Sprache, die es für die angefragte Ressource nicht gibt, wird **abgelehnt**.
Es wird nie in einer anderen Sprache geantwortet.

| Anfrage | Ergebnis |
|---------|----------|
| `?lang=es` | 400 `unsupported_language` |
| `/lookup/7?lang=fr` | 200, französische Bedeutung |
| `/spektra?lang=fr` | 400 `language_not_available` |
| `/spektra` (ohne `lang`) | 200, englisch — Vorgabe, keine Ersetzung |

Der letzte Fall ist wichtig für das Verständnis: ein *nicht gesetzter* Parameter
bekommt den dokumentierten Standard. Wer nichts angefragt hat, bekommt keine
Ersetzung. Sobald aber explizit etwas verlangt wurde, gibt es nur Erfüllen oder
Fehlschlagen — [PRINCIPLES §1](../PRINCIPLES.md).

Jede Antwort trägt ein `lang`-Feld mit der verwendeten Sprache. Da es keine
Ersetzung gibt, ist das immer die angefragte.

---

## Wo die Regel im Code steht

| Ort | Rolle |
|-----|-------|
| `Lang::PROMPT_LANGS`, `Lang::has_prompts()` | Welche Sprachen Prompts haben |
| `bedeutungen_source(lang)` | YAML-Quelle für Bedeutungen — alle drei Sprachen |
| `rtap_source(lang) -> Option<_>` | YAML-Quelle für RTAP, `None` ohne Prompts |
| `spektra::prompt_assets(lang) -> Option<_>` | Template **und** Labels als Paar |
| `resolve_lang` / `resolve_prompt_lang` (Server) | Auflösen und Ablehnen an der HTTP-Grenze |

Die drei `Option`-Funktionen matchen **jede** `Lang`-Variante einzeln, ohne
`_`-Auffangzweig. Eine neue Sprache erzeugt einen Compile-Fehler und erzwingt
die Entscheidung, ob es Prompts dafür gibt — statt versehentlich englische zu
erben. Siehe [PRINCIPLES §5](../PRINCIPLES.md).

`prompt_assets` gibt Template und Labels als **Paar** zurück, weil sie
zusammengehören: der Füll-Regex wird aus den Labels gebaut und auf das Template
angewandt. Driften sie auseinander, wirft nichts einen Fehler — der Prompt geht
nur mit rohen `[Number]`-Platzhaltern raus. Der Test
`template_and_labels_match_for_every_language` fängt genau das ab.

---

## Deutsch-gepinnte Helfer

`load_bedeutungen()` und `lookup()` liefern **Deutsch**, bewusst nicht
`Lang::default()`. Sie bedienen deutschsprachige Oberflächen. Hingen sie am
Standard, würden nach der Umstellung auf Englisch stillschweigend englische
Bedeutungen in einen deutschen Prompt geraten.

Der Test `german_helpers_do_not_follow_the_default_language` sichert das ab.

---

## Eine Sprache hinzufügen

1. `bedeutungen.<code>.yaml` anlegen, gleiche Zahlen-Keys wie `bedeutungen.yaml`
2. Variante in `Lang` ergänzen. `code()`, `ALL`, `missing_meaning()`,
   `bedeutungen_source()`, `rtap_source()` und `prompt_assets()` matchen
   erschöpfend — der Compiler führt dich durch alle Stellen
3. Prompts entscheiden:
   - **ohne**: `rtap_source` und `prompt_assets` geben `None` zurück; die
     Prompt-Endpunkte lehnen die Sprache sauber ab
   - **mit**: `spektra_prompt.<code>.txt`, `rtap_*`-Keys, `SpektraLabels`-Konstante
     und Eintrag in `Lang::PROMPT_LANGS`
4. `cargo test` — geprüft werden Vollständigkeit, Key-Gleichheit zwischen den
   Sprachen, dass Übersetzungen keine Kopien des deutschen Texts sind, und dass
   Template und Labels zusammenpassen
5. Diese Datei und die Tabelle im [README](../../README.md) ergänzen
