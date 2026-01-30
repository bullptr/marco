mod cli;
mod parser;
mod runner;
mod test_types;
mod util;

use anyhow::Result;
use clap::Parser;
use cli::Args;
use glob::glob;
use parser::*;
use rayon::prelude::*;
use runner::*;

fn main() -> Result<()> {
    let mut args = Args::parse();
    args = args.set_defaults();

    // rayon configuration
    if let Some(n_threads) = args.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(n_threads)
            .build_global()?;
        if args.verbose {
            println!("Thread pool set to {n_threads} threads.");
        }
    }

    let files: Vec<_> = glob(&args.input)?.collect::<Result<_, _>>()?;
    println!("Found {} markdown files for `{}`", files.len(), &args.input);
    if files.is_empty() {
        println!("No test markdown files found for `{}`", &args.input);
        return Ok(());
    }
    let tests = collect_tests(&files)?;
    if tests.is_empty() {
        println!("No tests found in markdown files for `{}`", &args.input);
        return Ok(());
    }
    println!("Found {} tests in {} files.", tests.len(), files.len());

    let results: Vec<_> = tests.par_iter().map(run_test_case).collect();

    let passed = results.iter().filter(|r| r.passed).count();
    println!("\nResults: {} passed / {} total", passed, results.len());
    for res in &results {
        if res.passed {
            println!(
                "\x1b[92m✔\x1b[0m {} \x1b[90m(in {:?})\x1b[0m",
                res.name, res.file
            );
        } else {
            println!(
                "\x1b[91m✘\x1b[0m {} \x1b[90m(in {:?})\x1b[0m",
                res.name, res.file
            );
            if let Some(err) = &res.error {
                println!("    Error: {}", err);
            }
            util::print_diff(&res.actual, &res.expected);
        }
    }
    Ok(())
}
