use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    /// Glob or direct file for test collection
    #[clap(short, long, default_value = "**/*.marco.md")]
    pub input: String,

    /// Command to run the tests with (overridden by "runner" field in test header)
    #[clap(short, long)]
    pub runner: Option<String>,

    /// Maximum number of threads to use in parallel (default: num_cpus)
    #[clap(long, env = "MARCO_MAX_THREADS", value_name = "N")]
    pub threads: Option<usize>,

    /// Verbose output
    #[clap(short, long, default_value_t = false)]
    pub verbose: bool,
}

impl Args {
    pub fn set_defaults(mut self) -> Self {
        if self.input.is_empty() {
            self.input = "**/*.marco.md".to_owned();
        }
        self
    }
}
