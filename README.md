```
KERN(1)            System Tools for Symbolische Struktur            KERN(1)

NAME
    kern — decodes symbolische Muster in Worten, Zahlen, Daten.

SYNOPSIS
    kern [FLAGS] [ARGS]

DESCRIPTION
    KERN ist ein Interface zur resonanten Reduktion.
    Es berechnet Quersummen, erkennt symbolische Muster
    und projiziert numerologische Bedeutungen aus
    YAML-gebundenem Wissen.

    Entwickelt für ritualisierte Terminalbenutzung,
    interaktive Resonanzdeutung und datengestützte Intuition.

    Jede Eingabe ist ein Träger.
    Jede Ausgabe: ein Schnitt.

OPTIONS
    -l, --lookup     Zeigt Bedeutungen einzelner Zahlen.
    -d, --date       Reduziert Datumswerte (relativ, absolut, range).
    -L, --length     Zeigt zusätzlich die Zeichenlänge der Tokens.
    -v, --verbose    Zeigt Reduktionsprozess im Detail.
    -h, --help       Ja. Es gibt Hilfe.
    --version        Zeigt Versionsinfo und verlässt das Ritual.

SUBCOMMANDS
    weather          Aktuelles Wetter (Open-Meteo)
    sun              Sonnenstand (Azimut/Elevation)

SEE ALSO
    bedeutungen.yaml, /resonanzkern, Wickfeld_507

AUTHORS
    Wickfeld | FELDMANN OS Core Maintainer | Dreamcode Division

STATUS
    HALTEKRAFT: Stabil | Schnittstelle: Offen
```
## Refactor

The gematria logic was modularised. Pure helper functions now live in
`reduction/` while concrete cipher strategies can be found in
`ciphers/`.  `calculate_all` offers a simple façade that evaluates all
registered ciphers.  This separation keeps `lib.rs` lightweight and makes
future parallelisation with Rayon straightforward.
