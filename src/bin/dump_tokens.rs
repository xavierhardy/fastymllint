#![forbid(unsafe_code)]

//! Development tool: dump the token stream of a YAML file in a canonical
//! format, for differential testing against PyYAML (see
//! `tests/dump_tokens.py`).

use fastymllint::pyyaml::scanner::Scanner;
use fastymllint::pyyaml::tokens::{DirectiveValue, TokenKind};

fn main() {
    let path = std::env::args().nth(1).expect("usage: dump_tokens FILE");
    let data = std::fs::read(&path).expect("read file");
    let content = fastymllint::decoder::auto_decode(&data).expect("decode");

    let (tokens, error, _buffer) = Scanner::scan_all(&content);
    for t in tokens {
        let (name, value): (&str, String) = match &t.kind {
            TokenKind::StreamStart => ("StreamStart", String::new()),
            TokenKind::StreamEnd => ("StreamEnd", String::new()),
            TokenKind::Directive { name, value } => (
                "Directive",
                match value {
                    DirectiveValue::Yaml(a, b) => format!("{name} ({a}, {b})"),
                    DirectiveValue::Tag(h, p) => format!("{name} ({h}, {p})"),
                    DirectiveValue::Other => format!("{name} None"),
                },
            ),
            TokenKind::DocumentStart => ("DocumentStart", String::new()),
            TokenKind::DocumentEnd => ("DocumentEnd", String::new()),
            TokenKind::BlockSequenceStart => ("BlockSequenceStart", String::new()),
            TokenKind::BlockMappingStart => ("BlockMappingStart", String::new()),
            TokenKind::BlockEnd => ("BlockEnd", String::new()),
            TokenKind::FlowSequenceStart => ("FlowSequenceStart", String::new()),
            TokenKind::FlowMappingStart => ("FlowMappingStart", String::new()),
            TokenKind::FlowSequenceEnd => ("FlowSequenceEnd", String::new()),
            TokenKind::FlowMappingEnd => ("FlowMappingEnd", String::new()),
            TokenKind::Key => ("Key", String::new()),
            TokenKind::Value => ("Value", String::new()),
            TokenKind::BlockEntry => ("BlockEntry", String::new()),
            TokenKind::FlowEntry => ("FlowEntry", String::new()),
            TokenKind::Alias(v) => ("Alias", v.clone()),
            TokenKind::Anchor(v) => ("Anchor", v.clone()),
            TokenKind::Tag { handle, suffix } => (
                "Tag",
                format!("({}, {})", handle.as_deref().unwrap_or("None"), suffix),
            ),
            TokenKind::Scalar {
                value,
                plain,
                style,
            } => (
                "Scalar",
                format!(
                    "plain={} style={} value={:?}",
                    plain,
                    style.map(|c| c.to_string()).unwrap_or("None".to_string()),
                    value
                ),
            ),
        };
        let line = format!(
            "{name} [{},{},{}]-[{},{},{}] {value}",
            t.start_mark.pointer,
            t.start_mark.line,
            t.start_mark.column,
            t.end_mark.pointer,
            t.end_mark.line,
            t.end_mark.column,
        );
        println!("{}", line.trim_end());
    }
    if let Some(e) = error {
        match e.problem_mark {
            Some(m) => println!(
                "ERROR {} [{},{},{}]",
                e.problem, m.pointer, m.line, m.column
            ),
            None => println!("ERROR {}", e.problem),
        }
    }
}
