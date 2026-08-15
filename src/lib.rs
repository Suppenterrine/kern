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

    /// Content language of the meanings knowledge base.
    ///
    /// Only the meanings (`bedeutung`/`lichtseite`/`schattenseite`) are
    /// translated. Calculation output, cipher names and the SPEKTRA/RTAP
    /// prompts are language independent.
    /// English is the default: the API is an international interface, so an
    /// unqualified request gets English. German requires an explicit `lang=de`.
    /// Note this is *only* the default for request parameters — content that is
    /// German by nature (the SPEKTRA and RTAP prompts) pins [`Lang::De`]
    /// explicitly and must not follow this default.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Lang {
        De,
        #[default]
        En,
        Fr,
    }

    impl Lang {
        pub const ALL: [Lang; 3] = [Lang::De, Lang::En, Lang::Fr];

        pub fn code(self) -> &'static str {
            match self {
                Lang::De => "de",
                Lang::En => "en",
                Lang::Fr => "fr",
            }
        }

        /// Comma separated list of supported codes, for help and error texts.
        pub fn supported() -> String {
            Lang::ALL
                .iter()
                .map(|l| l.code())
                .collect::<Vec<_>>()
                .join(", ")
        }

        /// Parses a language tag by its primary subtag, case-insensitively.
        /// `"EN"`, `"en-US"` and `"en_GB"` all resolve to [`Lang::En`], which
        /// makes raw `Accept-Language` values usable as-is.
        pub fn parse(tag: &str) -> Option<Lang> {
            let primary = tag
                .trim()
                .split(['-', '_'])
                .next()
                .unwrap_or_default()
                .to_ascii_lowercase();
            Lang::ALL.into_iter().find(|l| l.code() == primary)
        }

        /// Languages the SPEKTRA and RTAP prompts exist in.
        pub const PROMPT_LANGS: [Lang; 2] = [Lang::De, Lang::En];

        /// Whether the prompts are available in this language. A language
        /// without prompts is **rejected**, never silently served in another
        /// language — returning English text to someone who asked for French
        /// is a wrong answer dressed up as a successful one.
        pub fn has_prompts(self) -> bool {
            Lang::PROMPT_LANGS.contains(&self)
        }

        /// Comma separated list of languages the prompts exist in.
        pub fn prompt_langs() -> String {
            Lang::PROMPT_LANGS
                .iter()
                .map(|l| l.code())
                .collect::<Vec<_>>()
                .join(", ")
        }

        /// Placeholder for numbers without an entry in this language.
        pub fn missing_meaning(self) -> &'static str {
            match self {
                Lang::De => "- keine Bedeutung -",
                Lang::En => "- no meaning -",
                Lang::Fr => "- aucune signification -",
            }
        }
    }

    impl fmt::Display for Lang {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.code())
        }
    }

    impl std::str::FromStr for Lang {
        type Err = String;

        fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
            Lang::parse(s).ok_or_else(|| {
                format!(
                    "unsupported language '{s}'. supported: {}",
                    Lang::supported()
                )
            })
        }
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

    /// Pure alphabet-position lookup. Cipher-independent: a letter always maps
    /// to its human counting position (A=1, B=2, ...). Special characters and
    /// digits are skipped, matching the program's established behaviour
    /// (`normalize_char` only accepts ascii letters). The result is
    /// deduplicated — each letter appears once, in first-seen order.
    pub fn alphabet_index(input: &str) -> Vec<(char, u32)> {
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        for ch in input.chars() {
            if !ch.is_ascii_alphabetic() {
                continue;
            }
            let c = ch.to_ascii_uppercase();
            if seen.insert(c) {
                let idx = (c as u32) - ('A' as u32) + 1;
                out.push((c, idx));
            }
        }
        out
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn alphabet_index_is_one_based() {
            assert_eq!(alphabet_index("A"), vec![('A', 1)]);
            assert_eq!(alphabet_index("Z"), vec![('Z', 26)]);
        }

        #[test]
        fn alphabet_index_skips_special_chars() {
            assert_eq!(alphabet_index("a-b!c"), vec![('A', 1), ('B', 2), ('C', 3)]);
        }

        #[test]
        fn alphabet_index_dedupes() {
            assert_eq!(
                alphabet_index("kassel"),
                vec![('K', 11), ('A', 1), ('S', 19), ('E', 5), ('L', 12)]
            );
        }

        #[test]
        fn alphabet_index_case_insensitive() {
            assert_eq!(alphabet_index("AbC"), vec![('A', 1), ('B', 2), ('C', 3)]);
        }

        #[test]
        fn lang_default_is_english() {
            assert_eq!(Lang::default(), Lang::En);
            assert_eq!(Lang::default().code(), "en");
        }

        /// The SPEKTRA prompt is German and pulls its meanings through these
        /// helpers, so they must stay German even though the default is English.
        #[test]
        fn german_helpers_do_not_follow_the_default_language() {
            assert_ne!(Lang::default(), Lang::De, "precondition for this test");

            let map = load_bedeutungen();
            let german = load_bedeutungen_lang(Lang::De);
            assert_eq!(
                map.get(&1).unwrap().text,
                german.get(&1).unwrap().text,
                "load_bedeutungen() must stay German"
            );
            assert_eq!(
                lookup(10, &map),
                Lang::De.missing_meaning(),
                "lookup() must use the German placeholder"
            );
        }

        #[test]
        fn lang_parses_plain_codes() {
            assert_eq!(Lang::parse("de"), Some(Lang::De));
            assert_eq!(Lang::parse("en"), Some(Lang::En));
            assert_eq!(Lang::parse("fr"), Some(Lang::Fr));
        }

        #[test]
        fn lang_parsing_is_case_and_region_insensitive() {
            assert_eq!(Lang::parse("EN"), Some(Lang::En));
            assert_eq!(Lang::parse("en-US"), Some(Lang::En));
            assert_eq!(Lang::parse("fr_CA"), Some(Lang::Fr));
            assert_eq!(Lang::parse("  De  "), Some(Lang::De));
        }

        #[test]
        fn lang_rejects_unsupported_tags() {
            assert_eq!(Lang::parse("es"), None);
            assert_eq!(Lang::parse(""), None);
            assert_eq!(Lang::parse("english"), None);
        }

        #[test]
        fn lang_from_str_error_names_supported_codes() {
            let err = "es".parse::<Lang>().unwrap_err();
            assert!(err.contains("es"), "error should quote the input: {err}");
            for lang in Lang::ALL {
                assert!(
                    err.contains(lang.code()),
                    "error should list {}: {err}",
                    lang.code()
                );
            }
        }

        #[test]
        fn missing_meaning_is_localized_and_distinct() {
            let texts: Vec<&str> = Lang::ALL.iter().map(|l| l.missing_meaning()).collect();
            assert_eq!(texts.len(), 3);
            for (i, a) in texts.iter().enumerate() {
                assert!(!a.trim().is_empty());
                for b in texts.iter().skip(i + 1) {
                    assert_ne!(a, b, "placeholder texts must differ per language");
                }
            }
        }

        #[test]
        fn lookup_lang_uses_localized_placeholder_for_missing_number() {
            // 10 is not defined in any bedeutungen file.
            for lang in Lang::ALL {
                let map = load_bedeutungen_lang(lang);
                assert_eq!(lookup_lang(10, &map, lang), lang.missing_meaning());
            }
        }
    }

    /// Raw YAML source for a language. All files are embedded at compile time,
    /// so the binary stays self-contained and no runtime lookup can fail.
    fn bedeutungen_source(lang: Lang) -> &'static str {
        match lang {
            Lang::De => include_str!("../bedeutungen.yaml"),
            Lang::En => include_str!("../bedeutungen.en.yaml"),
            Lang::Fr => include_str!("../bedeutungen.fr.yaml"),
        }
    }

    /// Loads the meanings for `lang`. The German file is the base and also
    /// carries the language independent `rtap_*` prompts; non-numeric keys are
    /// skipped here in every language.
    pub fn load_bedeutungen_lang(lang: Lang) -> HashMap<u32, Bedeutung> {
        let yaml_str = bedeutungen_source(lang);
        let value: serde_yaml::Value = serde_yaml::from_str(yaml_str)
            .unwrap_or_else(|e| panic!("Failed to parse bedeutungen ({lang}): {e}"));

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

    /// Meanings in German, independent of [`Lang::default`]. This is for the
    /// German-only surfaces (SPEKTRA prompt); anything driven by a request
    /// parameter must use [`load_bedeutungen_lang`] instead.
    pub fn load_bedeutungen() -> HashMap<u32, Bedeutung> {
        load_bedeutungen_lang(Lang::De)
    }

    /// All languages at once — used by the server, which keeps every language
    /// resident in state instead of re-parsing YAML per request.
    pub fn load_all_bedeutungen() -> HashMap<Lang, HashMap<u32, Bedeutung>> {
        Lang::ALL
            .into_iter()
            .map(|lang| (lang, load_bedeutungen_lang(lang)))
            .collect()
    }

    /// YAML source carrying the `rtap_*` prompts for `lang`, or `None` if the
    /// prompts do not exist in that language. Matched exhaustively so a new
    /// language forces an explicit decision instead of inheriting a fallback.
    fn rtap_source(lang: Lang) -> Option<&'static str> {
        match lang {
            Lang::De => Some(include_str!("../bedeutungen.yaml")),
            Lang::En => Some(include_str!("../bedeutungen.en.yaml")),
            Lang::Fr => None,
        }
    }

    /// Load RTAP prompts for `lang`, or `None` if they do not exist in that
    /// language. Callers must surface that as an error rather than substituting
    /// another language.
    pub fn load_rtap_prompts_lang(lang: Lang) -> Option<HashMap<String, String>> {
        let yaml_str = rtap_source(lang)?;
        let value: serde_yaml::Value = serde_yaml::from_str(yaml_str)
            .unwrap_or_else(|e| panic!("Failed to parse RTAP prompts ({lang}): {e}"));

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

        Some(prompts)
    }

    /// RTAP prompts in the default language, which always has them.
    pub fn load_rtap_prompts() -> HashMap<String, String> {
        load_rtap_prompts_lang(Lang::default())
            .expect("the default language must have prompts")
    }

    /// Get RTAP prompt by part number (1 or 2)
    pub fn get_rtap_prompt(part: u8, prompts: &HashMap<String, String>) -> Option<&str> {
        let key = format!("rtap_{}", part);
        prompts.get(&key).map(|s| s.as_str())
    }

    /// Meaning text for `zahl`, falling back to the localized placeholder.
    /// `lang` must match the language `map` was loaded with — it only selects
    /// the placeholder wording.
    pub fn lookup_lang<'a>(zahl: u32, map: &'a HashMap<u32, Bedeutung>, lang: Lang) -> &'a str {
        map.get(&zahl)
            .and_then(|b| b.text.as_deref())
            .unwrap_or_else(|| lang.missing_meaning())
    }

    /// German lookup, paired with [`load_bedeutungen`].
    pub fn lookup<'a>(zahl: u32, map: &'a HashMap<u32, Bedeutung>) -> &'a str {
        lookup_lang(zahl, map, Lang::De)
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
            let s: i32 = a.parse().map_err(|_| "invalid range start")?;
            let e: i32 = b.parse().map_err(|_| "invalid range end")?;
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
            .map_err(|_| "invalid range specification".into())
    }
}
