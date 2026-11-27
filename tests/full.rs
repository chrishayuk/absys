//! `--full` (measurement only) prints lists and derivations in full, and
//! shows a cyclic term with `↻` rather than expanding it for ever. Absys has
//! no occur check, so a computation can finish with a cyclic value
//! (problem P01c of computational-paradigms), and printing it must not turn
//! a finished computation into one that seems not to end.

use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn full(text: &str) -> String {
    let dir = std::env::temp_dir().join(format!("absys-full-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join(format!("t{}.abs", text.len()));
    std::fs::File::create(&f).unwrap().write_all(text.as_bytes()).unwrap();
    let start = Instant::now();
    let mut child = Command::new(env!("CARGO_BIN_EXE_absys"))
        .arg(&f)
        .arg("--full")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if start.elapsed() > Duration::from_secs(20) {
            child.kill().unwrap();
            panic!("--full did not finish on {text:?}");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "{text:?}: {}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn a_cyclic_tail_is_shown_once() {
    assert!(full("X = [1|X]\n").contains("X = [1|↻]"));
}

#[test]
fn a_cyclic_head_is_shown_once() {
    assert!(full("X = [X]\n").contains("X = [↻]"));
}

#[test]
fn a_cycle_through_two_variables() {
    let o = full("X = [1,2|Y]\nY = [3|X]\n");
    assert!(o.contains("X = [1,2,3|↻]"), "{o}");
    assert!(o.contains("Y = [3,1,2|↻]"), "{o}");
}

#[test]
fn a_cycle_in_head_and_tail() {
    let o = full("X = [Y|X]\nY = [X]\n");
    assert!(o.contains("X = [[↻]|↻]"), "{o}");
}

#[test]
fn a_long_list_is_not_cut_short() {
    let items: Vec<String> = (1..=100).map(|i| i.to_string()).collect();
    let o = full(&format!("X = [{}]\n", items.join(",")));
    assert!(o.contains(&format!("X = [{}]", items.join(","))), "{o}");
    assert!(!o.contains('…'));
}

#[test]
fn a_partly_known_list_keeps_its_unknown_tail() {
    assert!(full("X = [1,2|T]\n").contains("X = [1,2|T]"));
}

#[test]
fn every_derivation_is_shown() {
    let o = full("p = lambda A\n  {A = 1} or {A = 2} or {A = 3}\np(A)\np(B)\np(C)\n");
    assert!(o.starts_with("27 derivations"), "{o}");
    assert_eq!(o.matches("(hibernating)").count(), 27, "{o}");
    assert!(!o.contains("more"));
}
