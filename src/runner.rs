use crate::test_types::{MarcoTestCase, TestResult};
use crate::util::*;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn run_test_case(test: &MarcoTestCase) -> TestResult {
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
    let (prog, args) = {
        let shell_prog = "powershell".to_string();
        (
            shell_prog,
            vec![
                "-NoProfile".to_string(),
                "-Command".to_string(),
                runner_cmd.for_current_platform().to_string(),
            ],
        )
    };

    #[cfg(not(windows))]
    let (prog, args) = {
        match parse_shell_cmd(runner_cmd.for_current_platform()) {
            Some(x) => (x.0, x.1),
            None => {
                return TestResult {
                    name: test.header.name.clone(),
                    file: test.file.clone(),
                    passed: false,
                    actual: String::new(),
                    expected: test.expected_output.clone(),
                    error: Some(format!(
                        "Malformed 'runner' command: {:?}",
                        runner_cmd.for_current_platform()
                    )),
                };
            }
        }
    };

    let test_dir = test.file.parent().unwrap_or_else(|| Path::new("."));

    let mut child = match Command::new(&prog)
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(test_dir)
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
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

    if !test.input_data.is_empty() {
        if let Some(mut stdin) = child.stdin.take() {
            if let Err(e) = stdin.write_all(test.input_data.as_bytes()) {
                return TestResult {
                    name: test.header.name.clone(),
                    file: test.file.clone(),
                    passed: false,
                    actual: String::new(),
                    expected: test.expected_output.clone(),
                    error: Some(format!("Failed to write to child stdin: {}", e)),
                };
            }
            drop(stdin);
        }
    } else {
        drop(child.stdin.take());
    }

    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(e) => {
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

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

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
