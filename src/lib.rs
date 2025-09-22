//! Öffentliche Kern-API für Tests & Binary

pub mod core {
    use chrono::{Local, NaiveDate};
    use regex::Regex;
    use serde::Deserialize;
    use std::collections::HashMap;

    #[derive(Debug, Deserialize)]
    pub struct Bedeutung {
        #[serde(alias = "bedeutung")]
        pub text: Option<String>,
        #[serde(alias = "licht", alias = "lichtseite")]
        pub licht: Option<String>,
        #[serde(alias = "schatten", alias = "schattenseite")]
        pub schatten: Option<String>,
    }

    pub fn char_to_value(ch: char) -> u32 {
        match ch {
            '0'..='9' => ch as u32 - '0' as u32,
            'A'..='Z' => ch as u32 - 'A' as u32 + 1,
            'a'..='z' => ch as u32 - 'a' as u32 + 1,
            _ => 0,
        }
    }

    pub fn load_bedeutungen() -> HashMap<u32, Bedeutung> {
        // Datei wird zur Compilezeit als String eingebettet
        let yaml_str = include_str!("../bedeutungen.yaml");
        serde_yaml::from_str(yaml_str)
            .expect("Eingebettete bedeutungen.yaml konnte nicht geparst werden")
    }

    pub fn lookup<'a>(zahl: u32, map: &'a HashMap<u32, Bedeutung>) -> &'a str {
        map.get(&zahl)
            .and_then(|b| b.text.as_deref())
            .unwrap_or("- keine Bedeutung -")
    }

    pub fn reduce_number_steps(input: &str) -> (u32, Vec<String>) {
        // Sonderfall: Eingabe ist Masterzahl → sofort zurückgeben
        if input == "11" || input == "22" || input == "33" {
            let line = format!("{input} ist eine Masterzahl → {input}");
            return (input.parse().unwrap(), vec![line]);
        }

        // 1. Werte der einzelnen Zeichen berechnen
        let values: Vec<u32> = input.chars().map(char_to_value).collect();
        let mut num: u32 = values.iter().sum();

        let mut lines = Vec::new();
        lines.push(format!(
            "{} → [{}] = {}",
            input,
            values
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join("+"),
            num
        ));

        // 2. Reduktionen durchführen
        while num > 9 && !matches!(num, 11 | 22 | 33) {
            let digits: Vec<u32> = num
                .to_string()
                .chars()
                .map(|c| c.to_digit(10).unwrap())
                .collect();
            let sum: u32 = digits.iter().sum();
            lines.push(format!(
                "→ {} = {}",
                digits
                    .iter()
                    .map(|d| d.to_string())
                    .collect::<Vec<_>>()
                    .join("+"),
                sum
            ));
            num = sum;
        }

        lines.push(format!("→ Quersumme: {num}"));
        (num, lines)
    }

    pub fn reduce_number_verbose(input: &str, debug: bool) -> u32 {
        let (num, lines) = reduce_number_steps(input);
        if debug {
            for line in lines {
                println!("{line}");
            }
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

    // ---------------------------------------------------------------------
    // Wetter-Modul
    // ---------------------------------------------------------------------
    pub mod weather {
        use reqwest::blocking::Client;
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Deserialize, Serialize)]
        pub struct CurrentWeather {
            pub temperature: f64,
            pub windspeed: f64,
            #[serde(rename = "winddirection")]
            pub winddirection_deg: f64,
            pub weathercode: i32,
            pub time: String,
        }

        #[derive(Debug, Serialize)]
        pub struct CurrentWeatherView {
            pub temperature: f64,
            pub windspeed: f64,
            pub winddirection_deg: f64,
            pub winddirection: String,
            pub weathercode: i32,
            pub time: String,
        }

        impl CurrentWeather {
            pub fn to_view(&self) -> CurrentWeatherView {
                CurrentWeatherView {
                    temperature: self.temperature,
                    windspeed: self.windspeed,
                    winddirection_deg: self.winddirection_deg,
                    winddirection: deg_to_compass(self.winddirection_deg).to_string(),
                    weathercode: self.weathercode,
                    time: self.time.clone(),
                }
            }
        }

        #[derive(Deserialize)]
        struct ApiResponse {
            current_weather: CurrentWeather,
        }

        #[derive(Serialize)]
        struct QueryParams {
            latitude: f64,
            longitude: f64,
            current_weather: bool,
        }

        pub fn deg_to_compass(deg: f64) -> &'static str {
            const DIRS: [&str; 16] = [
                "N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE", "S", "SSW", "SW", "WSW", "W",
                "WNW", "NW", "NNW",
            ];
            let idx = ((deg + 11.25) % 360.0 / 22.5).floor() as usize;
            DIRS[idx % 16]
        }

        pub fn fetch_current_weather(lat: f64, lon: f64) -> Result<CurrentWeather, reqwest::Error> {
            fetch_current_weather_from(lat, lon, "https://api.open-meteo.com")
        }

        pub fn fetch_current_weather_view(
            lat: f64,
            lon: f64,
        ) -> Result<CurrentWeatherView, reqwest::Error> {
            fetch_current_weather(lat, lon).map(|cw| cw.to_view())
        }

        pub fn fetch_current_weather_from(
            lat: f64,
            lon: f64,
            base: &str,
        ) -> Result<CurrentWeather, reqwest::Error> {
            let url = format!("{base}/v1/forecast");
            let client = Client::new();
            let params = QueryParams {
                latitude: lat,
                longitude: lon,
                current_weather: true,
            };
            let resp: ApiResponse = client.get(&url).query(&params).send()?.json()?;
            Ok(resp.current_weather)
        }
    }

    // ---------------------------------------------------------------------
    // Sonnenstand-Modul
    // ---------------------------------------------------------------------
    pub mod sun {
        use chrono::{DateTime, Utc};
        use serde::Serialize;

        #[derive(Debug, Serialize)]
        pub struct SolarPos {
            pub azimuth: f64,
            pub elevation: f64,
        }

        #[derive(Debug, Serialize)]
        pub struct SolarPosView {
            pub azimuth: f64,
            pub azimuth_compass: String,
            pub elevation: f64,
        }

        impl SolarPos {
            pub fn to_view(&self) -> SolarPosView {
                let dir = super::weather::deg_to_compass(self.azimuth).to_string();
                SolarPosView {
                    azimuth: self.azimuth,
                    azimuth_compass: dir,
                    elevation: self.elevation,
                }
            }
        }

        pub fn solar_position(lat: f64, lon: f64, dt: DateTime<Utc>) -> SolarPos {
            use suncalc::{Timestamp, get_position};
            let ts = Timestamp(dt.timestamp_millis());
            let pos = get_position(ts, lat, lon);
            let az = (pos.azimuth.to_degrees() + 180.0) % 360.0;
            SolarPos {
                azimuth: az,
                elevation: pos.altitude.to_degrees(),
            }
        }

        pub fn solar_position_view(lat: f64, lon: f64, dt: DateTime<Utc>) -> SolarPosView {
            solar_position(lat, lon, dt).to_view()
        }
    }

    // ---------------------------------------------------------------------
    // Wetter und Sonnenstand Modul kombiniert
    // ---------------------------------------------------------------------
    pub mod sky {
        use super::sun;
        use super::weather;
        use chrono::{DateTime, Utc};
        use serde::Serialize;

        #[derive(Debug, Serialize)]
        pub struct SkyReport {
            pub weather: WeatherOut,
            pub sun: SunOut,
        }

        #[derive(Debug, Serialize)]
        pub struct WeatherOut {
            pub temperature: f64,
            pub windspeed: f64,
            pub winddirection_deg: f64,
            pub winddirection: String,
            pub weathercode: i32,
            pub time: String,
        }

        #[derive(Debug, Serialize)]
        pub struct SunOut {
            pub azimuth: f64,
            pub azimuth_compass: String,
            pub elevation: f64,
        }

        /// Kombinierter Report. `dt` optional; fällt sonst auf `Utc::now()` zurück.
        pub fn report(lat: f64, lon: f64, dt: Option<DateTime<Utc>>) -> Result<SkyReport, String> {
            let w = weather::fetch_current_weather(lat, lon)
                .map_err(|e| format!("weather error: {e}"))?;

            let t = dt.unwrap_or_else(|| Utc::now());
            let s = sun::solar_position(lat, lon, t);

            let weather_out = WeatherOut {
                temperature: w.temperature,
                windspeed: w.windspeed,
                winddirection_deg: w.winddirection_deg,
                winddirection: weather::deg_to_compass(w.winddirection_deg).to_string(),
                weathercode: w.weathercode,
                time: w.time,
            };

            let sun_out = SunOut {
                azimuth: s.azimuth,
                azimuth_compass: weather::deg_to_compass(s.azimuth).to_string(),
                elevation: s.elevation,
            };

            Ok(SkyReport {
                weather: weather_out,
                sun: sun_out,
            })
        }
    }
}
