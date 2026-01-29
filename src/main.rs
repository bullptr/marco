use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, anyhow};
use clap::Parser;
use glob::glob;
use markdown::mdast::Node;
use markdown::{ParseOptions, to_mdast};
use rayon::prelude::*;
use serde::Deserialize;
use serde_yaml;
use shell_words;
use similar::{ChangeTag, TextDiff};

/// Command-line options
#[derive(Parser, Debug, Clone)]
pub struct Args {
    /// Glob or direct file for test collection (default: "**/*.marco.md")
    #[clap(short, long, default_value = "**/*.marco.md")]
    pub input: String,
}

impl Args {
    pub fn set_defaults(mut self) -> Self {
        if self.input.is_empty() {
            self.input = "**/*.marco.md".to_owned();
        }
        self
    }
}

/// Represents the YAML header/frontmatter at the top of each test
#[derive(Debug, Clone, Deserialize)]
pub struct TestHeader {
    pub name: String,
    pub author: Option<String>,
    pub runner: Option<String>,
    pub passing: Option<bool>,
    pub date: Option<String>,
}

/// Represents a single test defined in a markdown file
#[derive(Debug, Clone)]
pub struct MarcoTestCase {
    pub header: TestHeader,
    pub file: PathBuf,
    pub input_data: String,
    pub expected_output: String,
    pub block_start_line: usize,
}

/// Test result summary
#[derive(Debug)]
pub struct TestResult {
    pub name: String,
    pub file: PathBuf,
    pub passed: bool,
    pub actual: String,
    pub expected: String,
    pub error: Option<String>,
}

fn main() -> Result<()> {
    let mut args = Args::parse();
    args = args.set_defaults();
    let files: Vec<PathBuf> = glob(&args.input)?.collect::<Result<Vec<_>, _>>()?;
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
    let results: Vec<TestResult> = tests.par_iter().map(|t| run_test_case(t)).collect();

    let passed = results.iter().filter(|r| r.passed).count();
    println!("\nResults: {} passed / {} total", passed, results.len());
    for res in &results {
        if res.passed {
            println!("✅ {} \x1b[90m(in {:?})\x1b[0m", res.name, res.file);
        } else {
            println!("❌ {} \x1b[90m(in {:?})\x1b[0m", res.name, res.file);
            if let Some(err) = &res.error {
                println!("    Error: {}", err);
            }
            // println!("    Expected: {}", res.expected.trim());
            // println!("    Actual:   {}", res.actual.trim());

            let diff = TextDiff::from_lines(res.actual.trim(), res.expected.trim());

            for change in diff.iter_all_changes() {
                let (tag_symbol, color) = match change.tag() {
                    ChangeTag::Delete => ("\x1b[91m-\x1b[0m ", "\x1b[97m"), // Red
                    ChangeTag::Insert => ("\x1b[92m+\x1b[0m ", "\x1b[97m"), // Green
                    ChangeTag::Equal => ("  ", "\x1b[90m"),                 // Grey
                };
                print!("    {}{}{}\x1b[0m", color, tag_symbol, change);
            }
        }
    }
    if passed != results.len() {
        std::process::exit(1);
    }
    Ok(())
}

/// Collects all test cases from the set of markdown test files
fn collect_tests(files: &[PathBuf]) -> Result<Vec<MarcoTestCase>> {
    let mut all = vec![];
    for file in files {
        let src =
            fs::read_to_string(file).with_context(|| format!("Failed to read file {:?}", file))?;
        let mut tests = parse_test_markdown(file.clone(), &src)?;
        all.append(&mut tests);
    }
    Ok(all)
}

/// Parses a markdown file and extracts a list of test cases
fn parse_test_markdown(file: PathBuf, src: &str) -> Result<Vec<MarcoTestCase>> {
    let mut result = Vec::new();
    let options = ParseOptions::default();
    let tree = to_mdast(src, &options).map_err(|e| anyhow!("Failed to parse markdown: {}", e))?;

    let mut iter = if let Node::Root(r) = &tree {
        r.children.iter().peekable()
    } else {
        return Err(anyhow!("Expected Root node from mdast tree"));
    };

    while let Some(node) = iter.next() {
        // Find the header: usually a ThematicBreak, then a Heading, and parse content as YAML.
        if let Node::ThematicBreak(_) = node {
            // The header should be a Heading node next
            if let Some(Node::Heading(h)) = iter.next() {
                let frontmatter = h
                    .children
                    .iter()
                    .filter_map(|n| match n {
                        Node::Text(t) => Some(t.value.as_str()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("");

                let header: TestHeader = serde_yaml::from_str(&frontmatter)
                    .map_err(|e| anyhow!("Failed to parse frontmatter as header: {}", e))?;

                // Now advance until we find "Test:" and then "Input" and "Expected Output"
                while let Some(Node::Heading(test_heading)) = iter.peek() {
                    if test_heading
                        .children
                        .iter()
                        .any(|c| matches!(c, Node::Text(t) if t.value.trim().starts_with("Test:")))
                    {
                        // Pop the "Test:" heading
                        iter.next();
                        // Look for "Input" and code block
                        let (input_data, input_line) = if let Some(Node::Heading(h)) = iter.next() {
                            if h.children
                                .iter()
                                .any(|c| matches!(c, Node::Text(t) if t.value.trim() == "Input"))
                            {
                                // Next node must be Code
                                if let Some(Node::Code(c)) = iter.next() {
                                    (
                                        c.value.clone(),
                                        c.position.as_ref().map(|p| p.start.line).unwrap_or(0),
                                    )
                                } else {
                                    return Err(anyhow!("Expected code block after Input heading"));
                                }
                            } else {
                                return Err(anyhow!("Expected 'Input' heading"));
                            }
                        } else {
                            return Err(anyhow!("Expected 'Input' heading after 'Test:'"));
                        };

                        // Look for "Expected Output" and code block
                        let expected_output = if let Some(Node::Heading(h)) = iter.next() {
                            if h.children.iter().any(|c| matches!(c, Node::Text(t) if t.value.trim() == "Expected Output")) {
                                // Next node must be Code
                                if let Some(Node::Code(c)) = iter.next() {
                                    c.value.clone()
                                } else {
                                    return Err(anyhow!("Expected code block after Expected Output heading"));
                                }
                            } else {
                                return Err(anyhow!("Expected 'Expected Output' heading"));
                            }
                        } else {
                            return Err(anyhow!("Expected 'Expected Output' heading"));
                        };

                        result.push(MarcoTestCase {
                            header: header.clone(),
                            file: file.clone(),
                            input_data,
                            expected_output,
                            block_start_line: input_line,
                        });
                    } else {
                        // Skip unrelated headings
                        iter.next();
                    }
                }
            }
        }
    }

    Ok(result)
}

/// Parses a commandline string into a program and its arguments, using shell quoting rules.
/// Returns (program, argument list) on success, or None on parse error/empty input.
fn _parse_shell_cmd(cmd: &str) -> Option<(String, Vec<String>)> {
    match shell_words::split(cmd) {
        Ok(words) if !words.is_empty() => {
            let prog = words[0].clone();
            let args = words[1..].to_vec();
            Some((prog, args))
        }
        _ => None,
    }
}

/// Run a single test case and return the result.
fn run_test_case(test: &MarcoTestCase) -> TestResult {
    let runner_cmd = match &test.header.runner {
        Some(cmd) => cmd,
        None => {
            return TestResult {
                name: test.header.name.clone(),
                file: test.file.clone(),
                passed: false,
                actual: String::new(),
                expected: test.expected_output.clone(),
                error: Some("No 'runner' command provided in test YAML header".to_string()),
            };
        }
    };

    #[cfg(windows)]
    let (prog, args, temp_script) = {
        // Write runner_cmd to a temporary .ps1 file
        use std::{
            env::temp_dir,
            time::{SystemTime, UNIX_EPOCH},
        };
        let mut temp_path = temp_dir();
        let filename = format!(
            "test_script_{:?}.ps1",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        temp_path.push(filename);
        if let Err(e) = fs::write(&temp_path, runner_cmd.as_bytes()) {
            return TestResult {
                name: test.header.name.clone(),
                file: test.file.clone(),
                passed: false,
                actual: String::new(),
                expected: test.expected_output.clone(),
                error: Some(format!("Failed to write temp .ps1 script: {}", e)),
            };
        }
        let shell_prog = "powershell";
        (
            shell_prog.to_string(),
            vec![
                "-NoProfile".to_string(),
                "-File".to_string(),
                temp_path.to_string_lossy().to_string(),
            ],
            Some(temp_path), // Return path for later deletion
        )
    };

    #[cfg(not(windows))]
    let (prog, args, temp_script) = {
        match parse_shell_cmd(runner_cmd) {
            Some(x) => (x.0, x.1, None),
            None => {
                return TestResult {
                    name: test.header.name.clone(),
                    file: test.file.clone(),
                    passed: false,
                    actual: String::new(),
                    expected: test.expected_output.clone(),
                    error: Some(format!("Malformed 'runner' command: {:?}", runner_cmd)),
                };
            }
        }
    };

    let test_dir = test.file.parent().unwrap_or_else(|| Path::new("."));

    let mut child = match Command::new(&prog)
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped()) // Capture stderr for diagnostic
        .current_dir(test_dir)
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            // Clean up script for Windows, if needed
            #[cfg(windows)]
            if let Some(ref s) = temp_script {
                let _ = fs::remove_file(s);
            }
            return TestResult {
                name: test.header.name.clone(),
                file: test.file.clone(),
                passed: false,
                actual: String::new(),
                expected: test.expected_output.clone(),
                error: Some(format!(
                    "Runner spawn error: {} (prog: {:?} args: {:?} dir: {:?})",
                    e, prog, args, test_dir
                )),
            };
        }
    };

    // Write to stdin and close it (signal EOF)
    if !test.input_data.is_empty() {
        if let Some(mut stdin) = child.stdin.take() {
            if let Err(e) = stdin.write_all(test.input_data.as_bytes()) {
                #[cfg(windows)]
                if let Some(ref s) = temp_script {
                    let _ = fs::remove_file(s);
                }
                return TestResult {
                    name: test.header.name.clone(),
                    file: test.file.clone(),
                    passed: false,
                    actual: String::new(),
                    expected: test.expected_output.clone(),
                    error: Some(format!("Failed to write to child stdin: {}", e)),
                };
            }
            // Explicitly close stdin
            drop(stdin);
        }
    } else {
        // In case the process tries to read from stdin, close it anyway
        drop(child.stdin.take());
    }

    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(e) => {
            #[cfg(windows)]
            if let Some(ref s) = temp_script {
                let _ = fs::remove_file(s);
            }
            return TestResult {
                name: test.header.name.clone(),
                file: test.file.clone(),
                passed: false,
                actual: String::new(),
                expected: test.expected_output.clone(),
                error: Some(format!("Failed waiting on child: {}", e)),
            };
        }
    };

    // Clean up temp script on Windows
    #[cfg(windows)]
    if let Some(ref s) = temp_script {
        let _ = fs::remove_file(s);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Compose actual for matching; for error reporting, show both stdout and stderr
    let actual = stdout.trim().to_owned();
    let expected = test.expected_output.trim();

    let passed = if is_json(expected) && is_json(&actual) {
        normalized_json_eq(expected, &actual)
    } else {
        actual == expected
    };

    TestResult {
        name: test.header.name.clone(),
        file: test.file.clone(),
        passed,
        actual: if passed {
            actual.clone()
        } else if !stderr.trim().is_empty() {
            format!("{}\n[stderr:{}]", actual, stderr.trim())
        } else {
            actual.clone()
        },
        expected: expected.to_string(),
        error: if passed {
            None
        } else {
            Some("Output did not match expected".to_string())
        },
    }
}

/// Checks if a &str is probably JSON (by looking for `{` or `[`)
fn is_json(s: &str) -> bool {
    let s = s.trim();
    s.starts_with('{') || s.starts_with('[')
}

/// Fast JSON normalization and comparison. Returns true if parsed JSONs are equal.
fn normalized_json_eq(a: &str, b: &str) -> bool {
    let v1: Result<serde_json::Value, _> = serde_json::from_str(a);
    let v2: Result<serde_json::Value, _> = serde_json::from_str(b);
    match (v1, v2) {
        (Ok(j1), Ok(j2)) => j1 == j2,
        _ => false,
    }
}
