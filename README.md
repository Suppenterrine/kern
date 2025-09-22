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
        --licht      Optional: Zeigt im Lookup die Lichtseite.
        --schatten   Optional: Zeigt im Lookup die Schattenseite.
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

API HINWEIS
    GET /lookup/:number[?parts=light|shadow|both]
        Liefert wie bisher { number, meaning },
        optional zusaetzlich Felder { light, shadow }.
    GET /lookup?numbers=1,2,3[&parts=light|shadow|both]
        Liefert Liste von Items mit denselben optionalen Feldern.

AUTHORS
    Wickfeld | FELDMANN OS Core Maintainer | Dreamcode Division

STATUS
    HALTEKRAFT: Stabil | Schnittstelle: Offen
```
