//! Öffentliche Kern-API für Tests & Binary

pub mod core {
    use chrono::{Local, NaiveDate};
    use regex::Regex;
    use serde::{Deserialize, Serialize};
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

    #[path = "../core/utils.rs"]
    pub mod utils;

    #[path = "../core/ciphers/mod.rs"]
    pub mod ciphers;

    pub use ciphers::{
        Cipher, OrdinalCipher, PythagoreanCipher, ReverseOrdinalCipher, ReversePythagoreanCipher,
        available_cipher_names, default_cipher, get_cipher,
    };

    use utils::char_to_value_ordinal;

    #[derive(Debug, Clone, Serialize)]
    pub struct Step {
        pub pipe_index: usize,
        pub cipher_index: usize,
        pub operation: String,
    }

    impl Step {
        pub fn new(pipe_index: usize, cipher_index: usize, operation: impl Into<String>) -> Self {
            Self {
                pipe_index,
                cipher_index,
                operation: operation.into(),
            }
        }
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct KernResult {
        pub source: String,
        pub cipher: String,
        pub step: Step,
        pub value: u32,
        pub verbose: bool,
        pub trace: Vec<String>,
    }

    impl KernResult {
        pub fn new(
            source: impl Into<String>,
            cipher: impl Into<String>,
            step: Step,
            value: u32,
            verbose: bool,
            trace: Vec<String>,
        ) -> Self {
            Self {
                source: source.into(),
                cipher: cipher.into(),
                step,
                value,
                verbose,
                trace,
            }
        }

        pub fn from_input(input: &str, verbose: bool, cipher: &dyn Cipher, step: Step) -> Self {
            let (value, trace) = reduce_number_steps_with_cipher(input, cipher);
            Self::new(input, cipher.name(), step, value, verbose, trace)
        }

        pub fn from_input_default(input: &str, verbose: bool, step: Step) -> Self {
            let cipher = OrdinalCipher;
            Self::from_input(input, verbose, &cipher, step)
        }

        pub fn from_numeric_value(
            sum: u32,
            verbose: bool,
            cipher: &dyn Cipher,
            step: Step,
        ) -> Self {
            let input = sum.to_string();
            let (value, trace) = reduce_number_steps_with_cipher(&input, cipher);
            Self::new(input, cipher.name(), step, value, verbose, trace)
        }

        pub fn from_numeric_value_default(sum: u32, verbose: bool, step: Step) -> Self {
            let cipher = OrdinalCipher;
            Self::from_numeric_value(sum, verbose, &cipher, step)
        }

        pub fn value(&self) -> u32 {
            self.value
        }

        pub fn with_step(mut self, step: Step) -> Self {
            self.step = step;
            self
        }
    }

    #[derive(Debug, Default, Clone, Serialize)]
    pub struct ResultSet {
        pub results: Vec<KernResult>,
    }

    impl ResultSet {
        pub fn new() -> Self {
            Self {
                results: Vec::new(),
            }
        }

        pub fn add(&mut self, result: KernResult) {
            self.results.push(result);
        }

        pub fn iter(&self) -> impl Iterator<Item = &KernResult> {
            self.results.iter()
        }

        pub fn is_empty(&self) -> bool {
            self.results.is_empty()
        }

        pub fn len(&self) -> usize {
            self.results.len()
        }

        pub fn values(&self) -> impl Iterator<Item = u32> + '_ {
            self.results.iter().map(|r| r.value)
        }

        pub fn total(&self) -> u32 {
            self.values().sum()
        }

        pub fn lookup_value(&self, value: u32) -> Vec<&KernResult> {
            self.results.iter().filter(|r| r.value == value).collect()
        }
    }

    pub fn char_to_value(ch: char) -> u32 {
        char_to_value_ordinal(ch)
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

    pub fn reduce_number_steps_with_cipher(input: &str, cipher: &dyn Cipher) -> (u32, Vec<String>) {
        if input == "11" || input == "22" || input == "33" {
            let line = format!("{input} ist eine Masterzahl → {input}");
            return (input.parse().unwrap(), vec![line]);
        }

        let values: Vec<u32> = input.chars().map(|ch| cipher.char_to_value(ch)).collect();
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

    pub fn reduce_number_steps(input: &str) -> (u32, Vec<String>) {
        let cipher = OrdinalCipher;
        reduce_number_steps_with_cipher(input, &cipher)
    }

    pub fn reduce_number_verbose(input: &str, debug: bool) -> u32 {
        let step = Step::new(0, 0, "reduce");
        let result = KernResult::from_input_default(input, debug, step);
        if debug {
            for line in &result.trace {
                println!("{line}");
            }
        }
        result.value
    }

    pub fn parse_range(spec: &str) -> std::result::Result<Vec<i32>, String> {
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

        pub fn fetch_current_weather(
            lat: f64,
            lon: f64,
        ) -> std::result::Result<CurrentWeather, reqwest::Error> {
            fetch_current_weather_from(lat, lon, "https://api.open-meteo.com")
        }

        pub fn fetch_current_weather_view(
            lat: f64,
            lon: f64,
        ) -> std::result::Result<CurrentWeatherView, reqwest::Error> {
            fetch_current_weather(lat, lon).map(|cw| cw.to_view())
        }

        pub fn fetch_current_weather_from(
            lat: f64,
            lon: f64,
            base: &str,
        ) -> std::result::Result<CurrentWeather, reqwest::Error> {
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
        pub fn report(
            lat: f64,
            lon: f64,
            dt: Option<DateTime<Utc>>,
        ) -> std::result::Result<SkyReport, String> {
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
