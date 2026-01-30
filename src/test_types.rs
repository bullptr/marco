use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum RunnerConfig {
    Simple(String),
    Platform {
        windows: Option<String>,
        unix: Option<String>,
        default: Option<String>,
    },
}

impl RunnerConfig {
    pub fn for_current_platform(&self) -> &str {
        match self {
            RunnerConfig::Simple(cmd) => cmd,
            #[allow(unused_variables)]
            RunnerConfig::Platform {
                windows,
                unix,
                default,
            } => {
                #[cfg(target_os = "windows")]
                {
                    windows.as_deref().or(default.as_deref()).unwrap_or("echo")
                }
                #[cfg(not(target_os = "windows"))]
                {
                    unix.as_deref().or(default.as_deref()).unwrap_or("echo")
                }
            }
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
pub struct TestHeader {
    pub name: String,
    pub author: Option<String>,
    pub runner: Option<RunnerConfig>,
    pub passing: Option<bool>,
    pub date: Option<String>,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct MarcoTestCase {
    pub header: TestHeader,
    pub file: PathBuf,
    pub input_data: String,
    pub expected_output: String,
    pub block_start_line: usize,
}

#[derive(Debug)]
pub struct TestResult {
    pub name: String,
    pub file: PathBuf,
    pub passed: bool,
    pub actual: String,
    pub expected: String,
    pub error: Option<String>,
}
