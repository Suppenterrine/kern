# Entwicklungsprinzipien

Die Prinzipien, die dieses Projekt tragen. Sie sind bindend: wo Code oder
Dokumentation ihnen widerspricht, ist der Code oder die Dokumentation falsch,
nicht das Prinzip.

Wer ein Prinzip ändern will, ändert diese Datei — und zwar bewusst, mit
Begründung. Stillschweigend davon abweichen gilt nicht.

---

## 1. Keine stillen Fallbacks

**Wenn das Angefragte nicht verfügbar ist, wird das gesagt — nicht etwas
anderes geliefert.**

Eine Antwort, die inhaltlich etwas anderes ist als das Angefragte, aber wie ein
Erfolg aussieht, ist eine falsche Antwort im Gewand einer richtigen. Der
Aufrufer hat keine Möglichkeit, den Unterschied zu bemerken, und baut auf einer
Annahme weiter, die nicht stimmt. Ein Fehler ist unbequemer, aber ehrlich — und
er ist behebbar, weil er sichtbar ist.

**Konkret im Projekt:**

- `?lang=es` → HTTP 400 `unsupported_language`. Nicht: stillschweigend Englisch.
- `/spektra?lang=fr` → HTTP 400 `language_not_available`, obwohl `fr` bei den
  Bedeutungen funktioniert. Nicht: englischer Prompt mit `"lang": "en"` in der
  Antwort. Wer Französisch anfragt, soll nicht Englisch bekommen.
- `--lang es` im CLI → Fehlermeldung, Exit-Code 1. Nicht: Ausgabe in einer
  anderen Sprache.

**Was kein Fallback in diesem Sinne ist:** ein *nicht gesetzter* Parameter, der
einen dokumentierten Standard bekommt. Wer `lang` weglässt, hat nichts
angefragt und bekommt Englisch — das ist eine Vorgabe, keine Ersetzung.

**Ausnahme:** Fehlt ein Wert, der für die Bedienbarkeit unkritisch ist, und die
Vorgabe ist dokumentiert und im Ergebnis erkennbar, darf sie greifen. Sobald der
Nutzer aber *explizit etwas verlangt hat*, gibt es nur Erfüllen oder Fehlschlagen.

---

## 2. Sprachwahl ist Inhalt, nicht Protokoll

`lang` steuert die **Inhalte** (Bedeutungen, Prompts). Das Protokoll —
Fehlermeldungen, Feldnamen, Codes — ist immer Englisch.

Fehlertexte zu übersetzen klingt hilfreich, macht sie aber für Consumer
unbrauchbar: gegen übersetzte Prosa kann man nicht programmieren. Deshalb trägt
jeder Fehler einen stabilen `code`, gegen den geprüft wird, und einen
englischen `error`-Text für Menschen. Wer lokalisierte Fehler in seiner
Oberfläche braucht, übersetzt selbst anhand des Codes.

---

## 3. Es gibt genau eine Quelle der Wahrheit

Werte, die an mehreren Stellen stehen, driften auseinander — nicht vielleicht,
sondern zuverlässig.

- Die Version steht in `Cargo.toml`. Jede andere Fundstelle ist eine
  *abgeleitete Kopie* und wird von `cargo xtask sync-version` geschrieben.
- Versionen werden **niemals von Hand editiert**. Bump: `cargo set-version <X>`,
  danach `cargo xtask sync-version`.
- `cargo xtask sync-version --check` schreibt nichts und schlägt bei Drift fehl —
  als Gate verwendbar.

Dass das nötig ist, ist belegt: vor Einführung des Tools stand die Version in
`Cargo.toml` auf 1.2.0, im README auf 1.1.2 und in der OpenAPI-Spec auf 1.0.0.
Drei Stellen, drei Werte.

---

## 4. Werkzeuge statt Sorgfalt

Wo Konsistenz von Handarbeit abhängt, geht sie verloren. Wiederkehrende
Konsistenzarbeit gehört in ein Werkzeug, das deterministisch dasselbe tut, oder
in einen Test.

Ein Werkzeug, das teilweise durchläuft und dann abbricht, ist schlechter als
keines — es hinterlässt einen halb geänderten Zustand. Deshalb: erst alles
auflösen, dann schreiben. `sync-version` prüft jede Zieldatei, bevor es die
erste anfasst.

---

## 5. Der Compiler soll die Entscheidung erzwingen

Wo eine neue Variante eine Entscheidung nötig macht, wird erschöpfend gematcht,
statt einen `_`-Zweig als Auffangbecken zu benutzen. Ein `_ => englisch` ist
Prinzip 1 durch die Hintertür.

`prompt_assets()` und `rtap_source()` matchen jede `Lang`-Variante einzeln. Wer
eine vierte Sprache ergänzt, bekommt einen Compile-Fehler und muss entscheiden,
ob es Prompts dafür gibt — statt versehentlich englische zu erben.

---

## 6. Ehrlich berichten

Was nicht getan wurde, wird gesagt. Was nur teilweise funktioniert, wird als
teilweise beschrieben. Ein grüner Testlauf, der die fragliche Stelle nicht
abdeckt, ist kein Beleg.

Gilt auch rückwirkend: Wenn sich herausstellt, dass etwas anders ist als
berichtet, wird das korrigiert, statt es stehenzulassen.
