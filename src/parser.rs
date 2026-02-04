use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use dom_query::{Document, Selection};
use markdown::mdast::Node;
use markdown::{ParseOptions, to_html, to_mdast};
use serde_yml;

use crate::types::{MarcoTestCase, TestHeader};

/// Collects all test cases from the set of markdown test files
pub fn collect_tests(files: &[PathBuf]) -> Result<Vec<MarcoTestCase>> {
    let mut all = vec![];
    for file in files {
        let src =
            fs::read_to_string(file).with_context(|| format!("Failed to read file {:?}", file))?;
        let mut tests = parse_test_markdown_html(file.clone(), &src)?;
        all.append(&mut tests);
    }
    Ok(all)
}

/// Parses a markdown file as HTML and extracts a list of test cases
pub fn parse_test_markdown_html(file: PathBuf, src: &str) -> Result<Vec<MarcoTestCase>> {
    let mut result: Vec<MarcoTestCase> = Vec::new();
    let html = to_html(src);
    let document = Document::from(html.clone());
    let frontmatter = document.try_select("h2:first-of-type");

    if frontmatter.is_none() {
        eprintln!("Warning: no frontmatter found in file {:?}", file);

        return Ok(vec![]);
    }

    let frontmatter = frontmatter.unwrap().text();
    let header: TestHeader = serde_yml::from_str(&frontmatter)
        .map_err(|e| anyhow!("Failed to parse frontmatter as header: {}", e))?;

    // Collect all pre blocks' text into a Vec
    let pre_blocks: Vec<_> = document.select("pre").iter().collect();

    // Pair every two <pre> blocks into a MarcoTestCase
    for pair in pre_blocks.chunks(2) {
        if pair.len() == 2 {
            let mut header = header.clone();

            if let Some(title) = get_el_title(pair[0].clone()) {
                header.name = format!("{}: {}", header.name, title);
            }

            // replace "\n" with "\r\n"; byproduct of dom_query parsing
            let input_data = pair[0].text().to_string().replace("\n", "\r\n");
            let expected_output = pair[1].text().to_string().replace("\n", "\r\n");
            let test_case = MarcoTestCase {
                header: header.clone(),
                file: file.clone(),
                input_data,
                expected_output,
                block_start_line: 0, // @TODO: try to get line number from HTML
            };
            result.push(test_case);
        } else {
            return Err(anyhow!(
                "Unmatched input/expected output pair in file {:?}",
                file
            ));
        }
    }

    Ok(result)
}

/// Gets the title of the element's preceding header
pub fn get_el_title(el: Selection) -> Option<String> {
    let mut current = el.prev_sibling();
    while !current.is_empty() {
        if current.is("h1, h2, h3, h4, h5, h6") {
            return Some(current.text().trim().to_string());
        }
        current = current.prev_sibling();
    }
    None
}

/// Parses a markdown file and extracts a list of test cases
#[allow(unused)]
#[deprecated(note = "Use parse_test_markdown_html instead")]
pub fn parse_test_markdown(file: PathBuf, src: &str) -> Result<Vec<MarcoTestCase>> {
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
