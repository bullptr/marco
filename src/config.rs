use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    pub input_file: PathBuf,

    #[arg(short, long)]
    pub output_file: Option<PathBuf>,
}

impl Args {
    pub fn set_defaults(mut self) -> Self {
        if self.output_file.is_none() {
            let mut output_file = self.input_file.clone();
            output_file.set_extension("o");
            self.output_file = Some(output_file);
        }
        self
    }
}
