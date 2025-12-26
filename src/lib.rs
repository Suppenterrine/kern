//! Öffentliche Kern-API für Tests & Binary

pub mod ui;

pub mod core {
    use chrono::{Local, NaiveDate};
    use regex::Regex;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::fmt;

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

    #[path = "../core/flow.rs"]
    pub mod flow;

    #[path = "../core/spektra.rs"]
    pub mod spektra;

    #[path = "../core/phase.rs"]
    pub mod phase;

    pub use flow::{FlowContext, FlowFlags, Pipeline};

    pub use ciphers::{
        Cipher, CipherDescriptor, OrdinalCipher, PythagoreanCipher, ReverseOrdinalCipher,
        ReversePythagoreanCipher, available_cipher_names, default_cipher, descriptors, get_cipher,
    };

    pub use phase::{
        PhaseRelationResult, calculate_compartment, calculate_phase, generate_matrix_pairs,
    };

    use utils::char_to_value_ordinal;

    #[derive(Debug, Clone, Serialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Operation {
        Reduce,
        AggregateTotal,
        DateReduce,
        Lookup,
        PhaseRelation,
        Custom(String),
    }

    impl fmt::Display for Operation {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Operation::Reduce => write!(f, "reduce"),
                Operation::AggregateTotal => write!(f, "aggregate::total"),
                Operation::DateReduce => write!(f, "date::reduce"),
                Operation::Lookup => write!(f, "lookup"),
                Operation::PhaseRelation => write!(f, "phase::relation"),
                Operation::Custom(name) => write!(f, "{name}"),
            }
        }
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct Step {
        pub pipe_index: usize,
        pub cipher_index: usize,
        pub operation: Operation,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub metadata: Option<StepMetadata>,
    }

    #[derive(Debug, Clone, Serialize)]
    pub enum StepMetadata {
        PhaseRelation {
            left_index: usize,
            right_index: usize,
        },
    }

    impl Step {
        pub fn new(pipe_index: usize, cipher_index: usize, operation: Operation) -> Self {
            Self {
                pipe_index,
                cipher_index,
                operation,
                metadata: None,
            }
        }

        pub fn with_metadata(mut self, metadata: StepMetadata) -> Self {
            self.metadata = Some(metadata);
            self
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
        #[serde(skip_serializing_if = "Option::is_none")]
        pub payload: Option<String>,
    }

    impl KernResult {
        pub fn new(
            source: impl Into<String>,
            cipher: impl Into<String>,
            step: Step,
            value: u32,
            verbose: bool,
            trace: Vec<String>,
            payload: Option<String>,
        ) -> Self {
            Self {
                source: source.into(),
                cipher: cipher.into(),
                step,
                value,
                verbose,
                trace,
                payload,
            }
        }

        pub fn from_input(input: &str, verbose: bool, cipher: &dyn Cipher, step: Step) -> Self {
            let (value, trace) = reduce_number_steps_with_cipher(input, cipher);
            Self::new(input, cipher.name(), step, value, verbose, trace, None)
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
            Self::new(input, cipher.name(), step, value, verbose, trace, None)
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
        let value: serde_yaml::Value = serde_yaml::from_str(yaml_str)
            .expect("Failed to parse bedeutungen.yaml");

        let mut bedeutungen = HashMap::new();

        if let serde_yaml::Value::Mapping(map) = value {
            for (key, val) in map {
                // Only parse entries with numeric keys
                if let serde_yaml::Value::Number(num) = key {
                    if let Some(num_u64) = num.as_u64() {
                        let num_u32 = num_u64 as u32;
                        if let Ok(bedeutung) = serde_yaml::from_value::<Bedeutung>(val) {
                            bedeutungen.insert(num_u32, bedeutung);
                        }
                    }
                }
            }
        }

        bedeutungen
    }

    /// Load RTAP prompts from embedded bedeutungen.yaml
    pub fn load_rtap_prompts() -> HashMap<String, String> {
        let yaml_str = include_str!("../bedeutungen.yaml");
        let value: serde_yaml::Value = serde_yaml::from_str(yaml_str)
            .expect("Failed to parse bedeutungen.yaml");

        let mut prompts = HashMap::new();

        if let serde_yaml::Value::Mapping(map) = value {
            for (key, val) in map {
                if let (serde_yaml::Value::String(k), serde_yaml::Value::String(v)) = (key, val) {
                    if k.starts_with("rtap_") {
                        prompts.insert(k, v);
                    }
                }
            }
        }

        prompts
    }

    /// Get RTAP prompt by part number (1 or 2)
    pub fn get_rtap_prompt(part: u8, prompts: &HashMap<String, String>) -> Option<&str> {
        let key = format!("rtap_{}", part);
        prompts.get(&key).map(|s| s.as_str())
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
            "{} = {}",
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
                "{} = {}",
                digits
                    .iter()
                    .map(|d| d.to_string())
                    .collect::<Vec<_>>()
                    .join("+"),
                sum
            ));
            num = sum;
        }

        lines.push(format!("→ {num}"));
        (num, lines)
    }

    pub fn reduce_number_steps(input: &str) -> (u32, Vec<String>) {
        let cipher = OrdinalCipher;
        reduce_number_steps_with_cipher(input, &cipher)
    }

    pub fn reduce_number_verbose(input: &str, debug: bool) -> u32 {
        let step = Step::new(0, 0, Operation::Reduce);
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
}
