//! Every example, run as `absys FILE --env`, prints its `.out`.
use absys::machine::Opts;
use absys::report::{run, Show};
use absys::session::{notation_for, Session};
use std::fs;

#[test]
fn every_example_prints_its_out() {
    let mut n = 0;
    for e in fs::read_dir("examples").unwrap() {
        let path = e.unwrap().path();
        let name = path.to_string_lossy().to_string();
        if !(name.ends_with(".abs") || name.ends_with(".ab1") || name.ends_with(".ab2")) {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap();
        let mut s = Session::new(Opts {
            notation: notation_for(&name),
            ..Opts::default()
        });
        let got = run(
            &mut s,
            &text,
            Show {
                env: true,
                ..Show::default()
            },
        )
        .unwrap();
        let want = fs::read_to_string(path.with_extension("out")).unwrap();
        assert_eq!(got, want, "{name}");
        n += 1;
    }
    assert!(n >= 11);
}

#[test]
fn every_research_text_runs() {
    for e in fs::read_dir("research").unwrap() {
        let path = e.unwrap().path();
        if !path.is_file() {
            continue; // research/e07 holds the Prolog side of E07
        }
        let name = path.to_string_lossy().to_string();
        let text = fs::read_to_string(&path).unwrap();
        let mut s = Session::new(Opts {
            notation: notation_for(&name),
            ..Opts::default()
        });
        s.budget = 5000;
        s.load(&text).unwrap_or_else(|e| panic!("{name}: {e}"));
    }
}
