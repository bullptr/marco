use crate::test_types::{MarcoTestCase, TestHeader};
use anyhow::{Context, Result, anyhow};
use markdown::mdast::Node;
use markdown::{ParseOptions, to_mdast};
use std::fs;
use std::path::PathBuf;

/// Collects all test cases from the set of markdown test files
pub fn collect_tests(files: &[PathBuf]) -> Result<Vec<MarcoTestCase>> {
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
pub fn parse_test_markdown(file: PathBuf, src: &str) -> Result<Vec<MarcoTestCase>> {
    use serde_yml;
    let mut result = Vec::new();
    let options = ParseOptions::default();
    let tree = to_mdast(src, &options).map_err(|e| anyhow!("Failed to parse markdown: {}", e))?;
    let mut iter = if let Node::Root(r) = &tree {
        r.children.iter().peekable()
    } else {
        return Err(anyhow!("Expected Root node from mdast tree"));
    };

    while let Some(node) = iter.next() {
        if let Node::ThematicBreak(_) = node {
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

                let header: TestHeader = serde_yml::from_str(&frontmatter)
                    .map_err(|e| anyhow!("Failed to parse frontmatter as header: {}", e))?;

                // Now advance until we find "Test:" and then "Input" and "Expected Output"
                while let Some(Node::Heading(test_heading)) = iter.peek() {
                    if test_heading
                        .children
                        .iter()
                        .any(|c| matches!(c, Node::Text(t) if t.value.trim().starts_with("Test:")))
                    {
                        iter.next();
                        let (input_data, input_line) = if let Some(Node::Heading(h)) = iter.next() {
                            if h.children
                                .iter()
                                .any(|c| matches!(c, Node::Text(t) if t.value.trim() == "Input"))
                            {
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

                        let expected_output = if let Some(Node::Heading(h)) = iter.next() {
                            if h.children.iter().any(|c| matches!(c, Node::Text(t) if t.value.trim() == "Expected Output")) {
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
                        iter.next();
                    }
                }
            }
        }
    }

    Ok(result)
}
