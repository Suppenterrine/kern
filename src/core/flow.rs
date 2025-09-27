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

    pub fn run(
        &self,
        ctx: &mut FlowContext,
        inputs: &[String],
        ciphers: &[Box<dyn Cipher>],
    ) -> ResultSet {
        let mut result_set = ResultSet::new();
        let verbose = ctx.global_flags.verbose;

        for step in &self.steps {
            match &step.operation {
                Operation::Reduce | Operation::DateReduce => {
                    for (pipe_index, input) in inputs.iter().enumerate() {
                        for (cipher_index, cipher) in ciphers.iter().enumerate() {
                            let mut ctx_step = step.clone();
                            ctx_step.pipe_index = pipe_index;
                            ctx_step.cipher_index = cipher_index;

                            let result =
                                KernResult::from_input(input, verbose, cipher.as_ref(), ctx_step);
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
                    ctx_step.pipe_index = ctx.memory.len();
                    ctx_step.cipher_index = ciphers.len();

                    let result = KernResult::from_numeric_value_default(total, verbose, ctx_step);
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

                    let mut ctx_step = step.clone();
                    ctx_step.pipe_index = ctx.memory.len();
                    ctx_step.cipher_index = 0;

                    let result =
                        KernResult::new("lookup", "lookup", ctx_step, 0, verbose, vec![payload]);

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
}
