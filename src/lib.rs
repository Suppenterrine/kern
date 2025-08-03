//! Öffentliche Kern-API für Tests & Binary

pub mod core {
    use chrono::{Local, NaiveDate};
    use regex::Regex;
    use serde::Deserialize;
    use std::{collections::HashMap, fs, path::Path};

    #[derive(Debug, Deserialize)]
    pub struct Bedeutung {
        #[serde(alias = "bedeutung")]
        pub text: Option<String>,
    }

    pub fn char_to_value(ch: char) -> u32 {
        match ch {
            '0'..='9' => ch as u32 - '0' as u32,
            'A'..='Z' => ch as u32 - 'A' as u32 + 1,
            'a'..='z' => ch as u32 - 'a' as u32 + 1,
            _ => 0,
        }
    }

    pub fn load_bedeutungen(path: &Path) -> HashMap<u32, Bedeutung> {
        let yaml = fs::read_to_string(path).expect("bedeutungen.yaml nicht gefunden");
        serde_yaml::from_str(&yaml).expect("YAML konnte nicht geparst werden")
    }

    pub fn lookup<'a>(zahl: u32, map: &'a HashMap<u32, Bedeutung>) -> &'a str {
        map.get(&zahl)
            .and_then(|b| b.text.as_deref())
            .unwrap_or("– keine Bedeutung –")
    }

    pub fn reduce_number_verbose(input: &str, debug: bool) -> u32 {
        // Sonderfall: Eingabe ist Masterzahl → sofort zurückgeben
        if input == "11" || input == "22" || input == "33" {
            if debug {
                println!("{} ist eine Masterzahl → {}", input, input);
            }
            return input.parse().unwrap();
        }
        // 1. Werte der einzelnen Zeichen berechnen
        let values: Vec<u32> = input.chars().map(char_to_value).collect();
        let mut num: u32 = values.iter().sum();

        if debug {
            // Erste Debug-Zeile: Zeichenwerte
            println!(
                "{} → [{}] = {}",
                input,
                values
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join("+"),
                num
            );
        }

        // 2. Reduktionen durchführen
        while num > 9 && !matches!(num, 11 | 22 | 33) {
            let digits: Vec<u32> = num
                .to_string()
                .chars()
                .map(|c| c.to_digit(10).unwrap())
                .collect();
            let sum: u32 = digits.iter().sum();

            if debug {
                println!(
                    "→ {} = {}",
                    digits
                        .iter()
                        .map(|d| d.to_string())
                        .collect::<Vec<_>>()
                        .join("+"),
                    sum
                );
            }

            num = sum;
        }

        if debug {
            println!("→ Quersumme: {num}");
        }

        num
    }

    pub fn parse_range(spec: &str) -> Result<Vec<i32>, String> {
        let today = Local::now().date_naive();

        // A) Datums-Range: dd.mm.yyyy..dd.mm.yyyy
        if let Some((start, end)) = spec.split_once("..") {
            if let (Ok(sd), Ok(ed)) = (
                NaiveDate::parse_from_str(start, "%d.%m.%Y"),
                NaiveDate::parse_from_str(end, "%d.%m.%Y"),
            ) {
                let s = (sd - today).num_days() as i32;
                let e = (ed - today).num_days() as i32;
                let mut v = Vec::new();
                if s <= e {
                    for i in s..=e {
                        v.push(i);
                    }
                } else {
                    for i in (e..=s).rev() {
                        v.push(i);
                    }
                }
                return Ok(v);
            }
        }

        // B) Einzel-Datum
        if let Ok(d) = NaiveDate::parse_from_str(spec, "%d.%m.%Y") {
            let off = (d - today).num_days() as i32;
            return Ok(vec![off]);
        }

        // A)  -5..4   oder   3..-2
        if let Some((a, b)) = spec.split_once("..") {
            let s: i32 = a.parse().map_err(|_| "Ungültiger Start")?;
            let e: i32 = b.parse().map_err(|_| "Ungültiges Ende")?;
            let mut v = Vec::new();
            if s <= e {
                for i in s..=e {
                    v.push(i);
                }
            } else {
                for i in (e..=s).rev() {
                    v.push(i);
                }
            }
            return Ok(v);
        }

        // B) alte Syntax  0+3 / 0-3
        let re = Regex::new(r"^([+-]?\d+)([+-])(\d+)$").unwrap();
        if let Some(c) = re.captures(spec) {
            let start: i32 = c[1].parse().unwrap();
            let end_off: i32 = c[3].parse().unwrap();
            let end = if &c[2] == "+" { end_off } else { -end_off };
            let mut v = Vec::new();
            if start <= end {
                for i in start..=end {
                    v.push(i);
                }
            } else {
                for i in (end..=start).rev() {
                    v.push(i);
                }
            }
            return Ok(v);
        }

        // C) Einzelwert
        spec.parse::<i32>()
            .map(|v| vec![v])
            .map_err(|_| "Ungültige Range-Angabe".into())
    }
}
