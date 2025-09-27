use super::{Cipher, KernResult, Operation, ResultSet, Step};

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

    pub fn run(&self, inputs: &[String], ciphers: &[Box<dyn Cipher>], debug: bool) -> ResultSet {
        let mut result_set = ResultSet::new();

        for step in &self.steps {
            match &step.operation {
                Operation::Reduce | Operation::DateReduce => {
                    for (pipe_index, input) in inputs.iter().enumerate() {
                        for (cipher_index, cipher) in ciphers.iter().enumerate() {
                            let mut ctx_step = step.clone();
                            ctx_step.pipe_index = pipe_index;
                            ctx_step.cipher_index = cipher_index;

                            let result =
                                KernResult::from_input(input, debug, cipher.as_ref(), ctx_step);
                            result_set.add(result);
                        }
                    }
                }
                Operation::AggregateTotal => {
                    let total = result_set.total();

                    let mut ctx_step = step.clone();
                    ctx_step.pipe_index = result_set.len();
                    ctx_step.cipher_index = ciphers.len();

                    let result = KernResult::from_numeric_value_default(total, debug, ctx_step);
                    result_set.add(result);
                }
                Operation::Lookup => {
                    // Lookup stays in the CLI for now.
                }
                Operation::Custom(_) => {
                    // Placeholder for future extensions.
                }
            }
        }

        result_set
    }
}
