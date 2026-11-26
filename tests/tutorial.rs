//! Every example in the tutorial is run, and what it prints compared with the
//! `output` block under it. `ABSYS_BLESS=1 cargo test --test tutorial`
//! rewrites the output blocks from what the examples print.

use absys::machine::{Notation, Opts};
use absys::report::{run, Show};
use absys::session::Session;
use std::fs;

/// Runs one block as `absys FILE` (or with the flags on its info line) would.
fn output(info: &str, code: &str) -> String {
    let notation = if info.contains("1969") {
        Notation::Absys1
    } else if info.contains("1982") {
        Notation::Elcock1982
    } else {
        Notation::Retrospective
    };
    let show = Show {
        each: info.contains("--each"),
        env: info.contains("--env"),
        trace: info.contains("--trace"),
    };
    let mut s = Session::new(Opts {
        notation,
        ..Opts::default()
    });
    if let Some(steps) = info.split("--steps ").nth(1) {
        s.budget = steps.split_whitespace().next().unwrap().parse().unwrap();
    }
    match run(&mut s, code, show) {
        Ok(o) => o,
        Err(e) => {
            let (l, c) = absys::syntax::line_col(code, e.span.start);
            format!("{}:{}: {}\n", l + 1, c + 1, e.message)
        }
    }
}

/// An absys block: its info line, its code, and the line range of the output block under it.
type Block = (String, String, Option<(usize, usize)>);

/// Each absys block, and the output block under it if there is one.
fn blocks(text: &str) -> Vec<Block> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = vec![];
    let mut i = 0;
    while i < lines.len() {
        let l = lines[i];
        if let Some(info) = l.strip_prefix("```absys") {
            let start = i + 1;
            let mut j = start;
            while !lines[j].starts_with("```") {
                j += 1;
            }
            let code = lines[start..j].join("\n") + "\n";
            let mut k = j + 1;
            while k < lines.len() && lines[k].trim().is_empty() {
                k += 1;
            }
            let range = if k < lines.len() && lines[k].starts_with("```output") {
                let mut e = k + 1;
                while !lines[e].starts_with("```") {
                    e += 1;
                }
                Some((k + 1, e))
            } else {
                None
            };
            out.push((info.trim().to_string(), code, range));
            i = j + 1;
        } else {
            i += 1;
        }
    }
    out
}

#[test]
fn every_tutorial_example_prints_what_the_tutorial_says() {
    let bless = std::env::var("ABSYS_BLESS").is_ok();
    let mut names: Vec<_> = fs::read_dir("tutorial")
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "md"))
        .collect();
    names.sort();
    let mut checked = 0;
    let mut wrong = vec![];
    for path in names {
        let text = fs::read_to_string(&path).unwrap();
        let mut lines: Vec<String> = text.lines().map(String::from).collect();
        // replace from the end, so earlier ranges stay right
        for (info, code, range) in blocks(&text).into_iter().rev() {
            let Some((a, b)) = range else { continue };
            checked += 1;
            let got = output(&info, &code);
            let want = lines[a..b].join("\n") + "\n";
            if bless {
                lines.splice(a..b, got.trim_end_matches('\n').lines().map(String::from));
            } else if got.trim_end_matches('\n') != want.trim_end_matches('\n') {
                wrong.push(format!(
                    "{}: line {}\n--- want\n{want}--- got\n{got}",
                    path.display(),
                    a
                ));
            }
        }
        if bless {
            fs::write(&path, lines.join("\n") + "\n").unwrap();
        }
    }
    assert!(checked > 0, "no examples found");
    assert!(
        wrong.is_empty(),
        "{} of {checked} differ:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}
