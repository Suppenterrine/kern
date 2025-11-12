use super::{Cipher, KernResult, Operation, ResultSet, Step};
use serde::Serialize;
use std::collections::{BTreeSet, HashMap, hash_map::Entry};

#[derive(Debug, Clone, Default)]
pub struct FlowFlags {
    pub verbose: bool,
    pub ciphers: Vec<String>,
    pub total: bool,
}

#[derive(Debug, Default)]
pub struct FlowContext {
    pub global_flags: FlowFlags,
    pub memory: Vec<KernResult>,
}

impl FlowContext {
    pub fn new(global_flags: FlowFlags) -> Self {
        Self {
            global_flags,
            memory: Vec::new(),
        }
    }

    pub fn record(&mut self, result: KernResult) {
        self.memory.push(result);
    }
}

#[derive(Debug, Default)]
pub struct Pipeline {
    steps: Vec<Step>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn add_step(&mut self, step: Step) {
        self.steps.push(step);
    }

    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    pub fn run(
        &self,
        ctx: &mut FlowContext,
        inputs: &[String],
        ciphers: &[Box<dyn Cipher>],
    ) -> ResultSet {
        let mut result_set = ResultSet::new();

        for step in &self.steps {
            let effective_verbose = step.local_flags.verbose.unwrap_or(ctx.global_flags.verbose);

            match &step.operation {
                Operation::Reduce | Operation::DateReduce => {
                    if let Some(input) = inputs.get(step.pipe_index) {
                        for (cipher_index, cipher) in self.select_ciphers(step, ciphers, &ctx.global_flags.ciphers) {
                            let mut ctx_step = step.clone();
                            ctx_step.cipher_index = cipher_index;

                            let result = KernResult::from_input(
                                input,
                                effective_verbose,
                                cipher.as_ref(),
                                ctx_step,
                            );
                            ctx.record(result.clone());
                            result_set.add(result);
                        }
                    }
                }
                Operation::AggregateTotal => {
                    let total: u32 = ctx
                        .memory
                        .iter()
                        .filter(|res| {
                            matches!(
                                res.step.operation,
                                Operation::Reduce | Operation::DateReduce | Operation::Custom(_)
                            )
                        })
                        .map(|res| res.value())
                        .sum();

                    let mut ctx_step = step.clone();
                    ctx_step.cipher_index = ciphers.len();

                    let result =
                        KernResult::from_numeric_value_default(total, effective_verbose, ctx_step);
                    ctx.record(result.clone());
                    result_set.add(result);
                }
                Operation::Lookup => {
                    #[derive(Serialize)]
                    struct LookupEntry {
                        value: u32,
                        sources: Vec<String>,
                    }

                    let mut grouped: HashMap<u32, BTreeSet<String>> = HashMap::new();
                    let mut order: Vec<u32> = Vec::new();

                    for res in &ctx.memory {
                        if matches!(
                            res.step.operation,
                            Operation::Reduce | Operation::DateReduce | Operation::Custom(_)
                        ) {
                            match grouped.entry(res.value()) {
                                Entry::Occupied(mut occ) => {
                                    occ.get_mut()
                                        .insert(format!("{} [{}]", res.source, res.cipher));
                                }
                                Entry::Vacant(vac) => {
                                    let mut set = BTreeSet::new();
                                    set.insert(format!("{} [{}]", res.source, res.cipher));
                                    vac.insert(set);
                                    order.push(res.value());
                                }
                            }
                        }
                    }

                    let mut entries: Vec<LookupEntry> = Vec::new();
                    for value in order {
                        if let Some(sources) = grouped.get(&value) {
                            entries.push(LookupEntry {
                                value,
                                sources: sources.iter().cloned().collect(),
                            });
                        }
                    }

                    let payload =
                        serde_json::to_string(&entries).unwrap_or_else(|_| String::from("[]"));

                    let mut trace = Vec::new();
                    if effective_verbose {
                        trace.push(format!("Lookup entries: {}", entries.len()));
                        for entry in &entries {
                            trace.push(format!("{} -> {}", entry.value, entry.sources.join(", ")));
                        }
                    }

                    let ctx_step = step.clone();

                    let result = KernResult::new(
                        "lookup",
                        "lookup",
                        ctx_step,
                        0,
                        effective_verbose,
                        trace,
                        Some(payload),
                    );

                    ctx.record(result.clone());
                    result_set.add(result);
                }
                Operation::Custom(_) => {
                    // Placeholder for future extensions.
                }
            }
        }

        result_set
    }

    fn select_ciphers<'a>(
        &self,
        step: &Step,
        ciphers: &'a [Box<dyn Cipher>],
        global_cipher_names: &[String],
    ) -> Vec<(usize, &'a Box<dyn Cipher>)> {
        use std::collections::HashSet;

        // Build the set of cipher names to use
        let mut target_names: HashSet<String> = global_cipher_names
            .iter()
            .map(|s| s.to_lowercase())
            .collect();

        // If local ciphers specified, ADD them to the global set (additive behavior)
        if let Some(local_names) = &step.local_flags.ciphers {
            for name in local_names {
                target_names.insert(name.to_lowercase());
            }
        }

        // If no target names (no global, no local), return all ciphers
        if target_names.is_empty() {
            return ciphers
                .iter()
                .enumerate()
                .map(|(idx, cipher)| (idx, cipher))
                .collect();
        }

        // Filter ciphers to match the target names
        let mut selected = Vec::new();
        for (idx, cipher) in ciphers.iter().enumerate() {
            if target_names.contains(&cipher.name().to_lowercase()) {
                selected.push((idx, cipher));
            }
        }

        // Fallback: if no matches found, return all ciphers
        if selected.is_empty() {
            ciphers
                .iter()
                .enumerate()
                .map(|(idx, cipher)| (idx, cipher))
                .collect()
        } else {
            selected
        }
    }
}
