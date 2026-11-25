//! The acceptance tests: every computation the sources give a result for,
//! run, and compared with what the source says. Each test names its page.

use absys::machine::{Deriv, Negation, Notation, Opts, Select};
use absys::session::{answers, Session};

fn run(text: &str, opts: Opts) -> Session {
    let mut s = Session::new(opts);
    s.load(text).unwrap();
    s
}

fn r90(text: &str) -> Session {
    run(text, Opts::default())
}

fn r69(text: &str) -> Session {
    run(
        text,
        Opts {
            notation: Notation::Absys1,
            ..Opts::default()
        },
    )
}

fn value(d: &Deriv, name: &str, n: Notation) -> String {
    answers(d, n)
        .into_iter()
        .find(|a| a.name == name)
        .map(|a| a.value)
        .unwrap_or_else(|| panic!("no {name}"))
}

fn env(d: &Deriv) -> Vec<String> {
    let mut v: Vec<String> = d
        .environment(Notation::Retrospective)
        .into_iter()
        .map(|(k, v)| format!("{k}/{v}"))
        .collect();
    v.sort();
    v
}

fn sorted(v: &[&str]) -> Vec<String> {
    let mut v: Vec<String> = v.iter().map(|s| s.to_string()).collect();
    v.sort();
    v
}

const APP: &str = "app = lambda R, S, T\n {R = [], S = T} or\n {R = [X|L], T = [X|N], app(L, S, N)}\n";
const REV: &str = "rev1 = lambda R, S, T\n {R = [], S = T} or\n {R = [X|L], rev1(L, [X|S], T)}\n\
                   rev = lambda R, S\n {rev1(R, [], S)}\n";

/// E90 pp. 9-10: app([1], U, [1,2]) ends with the binding environment
/// X₁/1, L₁/[], N₁/[2], U/[2]; note 11: tout("U", U) produces "U = [2]".
#[test]
fn e90_p10_append_refutation() {
    let s = r90(&format!("{APP}app([1], U, [1,2])\ntout(\"U\", U)\n"));
    assert_eq!(s.comp.derivs.len(), 1);
    let d = &s.comp.derivs[0];
    assert_eq!(env(d), sorted(&["X₁/1", "L₁/[]", "N₁/[2]", "U/[2]"]));
    assert_eq!(d.outputs, vec!["U = [2]"]);
}

/// E90 p. 12: rev([1,2,U], [V|L]) hibernates with X₁/1, L₁/[2,U], X₂/2,
/// L₂/[U], L₃/[], L/[2,1], and the suspended equations U = X₃ and X₃ = V.
#[test]
fn e90_p12_suspended_equations() {
    let s = r90(&format!("{REV}rev([1,2,U], [V|L])\n"));
    assert_eq!(s.comp.derivs.len(), 1);
    let d = &s.comp.derivs[0];
    assert_eq!(
        env(d),
        sorted(&["X₁/1", "L₁/[2,U]", "X₂/2", "L₂/[U]", "L₃/[]", "L/[2,1]"])
    );
    let mut waits: Vec<String> = d
        .waiting()
        .map(|w| d.show_goal(&w.goal, Notation::Retrospective))
        .collect();
    waits.sort();
    assert_eq!(waits, vec!["U = X₃", "X₃ = V"]);
    // "the computed answer substitution for U is constrained to be U/V"
    assert_eq!(value(d, "U", Notation::Retrospective), "V");
}

/// E90 p. 13: adding rev([5], [V]) binds V to 5, reprocesses X₃ = V and then
/// U = X₃, giving X₁/1, L₁/[2,5], X₂/2, L₂/[5], L₃/[], L/[2,1], X₄/5, L₄/[],
/// V/5, X₃/5, U/5.
#[test]
fn e90_p13_an_assertion_added_later() {
    let s = r90(&format!("{REV}rev([1,2,U], [V|L])\nrev([5], [V])\n"));
    assert_eq!(s.comp.derivs.len(), 1);
    let d = &s.comp.derivs[0];
    assert_eq!(
        env(d),
        sorted(&[
            "X₁/1",
            "L₁/[2,5]",
            "X₂/2",
            "L₂/[5]",
            "L₃/[]",
            "L/[2,1]",
            "X₄/5",
            "L₄/[]",
            "V/5",
            "X₃/5",
            "U/5"
        ])
    );
    assert_eq!(d.waiting().count(), 0);
}

const STUDENTS: &str = "student = lambda X\n {X = 'BROWN'} or {X = 'SMITH'}\n\
    math-course = lambda C\n {C = 'C101'} or {C = 'C301'}\n\
    takes = lambda X, C\n {X = 'BROWN', C = 'C101'} or\n {X = 'SMITH', C = 'C101'} or\n {X = 'SMITH', C = 'C301'}\n\
    non-maths-major = lambda X\n {math-course(C), not(takes(X, C))}\n";

/// E90 p. 15: the completion of Clark's microdatabase. Brown takes C101 and
/// not C301, so is a non-maths-major in Clark's sense; Smith takes both.
#[test]
fn e90_p15_negation_as_failure() {
    assert_eq!(
        r90(&format!("{STUDENTS}non-maths-major('BROWN')\n")).comp.derivs.len(),
        1
    );
    assert_eq!(
        r90(&format!("{STUDENTS}non-maths-major('SMITH')\n")).comp.derivs.len(),
        0
    );
}

const SETDIFF: &str = "mem = lambda X, S\n {S = [X|T]} or {S = [Y|T], mem(X, T)}\n\
    setdiff = lambda S1, S2, S\n {set(X), {mem(X, S1), not(mem(X, S2))}, S}\n";

/// E90 p. 15: setdiff, from "a demonstration of Absys-4 circa 1968".
#[test]
fn e90_p15_set_aggregation() {
    let s = r90(&format!("{SETDIFF}setdiff([1,2,3,4], [2,4,6], D)\n"));
    assert_eq!(value(&s.comp.derivs[0], "D", Notation::Retrospective), "[1,3]");
    let s = r90(&format!("{SETDIFF}setdiff([1,2], [1,2], D)\n"));
    assert_eq!(value(&s.comp.derivs[0], "D", Notation::Retrospective), "[]");
}

/// Decision 15: with literals taken strictly in turn, the same text gives [1]
/// (unsafe negation, elaborated before the recursion that instantiates X).
#[test]
fn decision_15_fifo_breaks_setdiff() {
    let fifo = Opts {
        select: Select::Fifo,
        ..Opts::default()
    };
    let s = run(&format!("{SETDIFF}setdiff([1,2,3,4], [2,4,6], D)\n"), fifo);
    assert_eq!(value(&s.comp.derivs[0], "D", Notation::Retrospective), "[1]");
    // and waiting for the variables of not repairs it
    let delayed = Opts {
        negation: Negation::Delayed,
        ..fifo
    };
    let s = run(&format!("{SETDIFF}setdiff([1,2,3,4], [2,4,6], D)\n"), delayed);
    assert_eq!(value(&s.comp.derivs[0], "D", Notation::Retrospective), "[1,3]");
}

/// FE69: "the trivial program x=y y = 2 would make both x and y the datum 2."
#[test]
fn fe69_x_equals_y() {
    let s = r69("x=y   y = 2");
    let d = &s.comp.derivs[0];
    assert_eq!(value(d, "x", Notation::Absys1), "2");
    assert_eq!(value(d, "y", Notation::Absys1), "2");
}

/// FE69: "The occurrence in a program of the assertions y=x, x=2 and 3=y
/// would make that program terminate unsuccessfully".
#[test]
fn fe69_unsatisfiable() {
    assert!(r69("y=x x=2 3=y").comp.derivs.is_empty());
}

/// FE69: after p=[1;2] {p = [a;b] or p = [b;a]} "there are two sets of
/// distinct data"; after a=b+1 "one of the conjunctions will be
/// unsatisfiable and will disappear".
#[test]
fn fe69_alternatives() {
    let mut s = Session::new(Opts {
        notation: Notation::Absys1,
        ..Opts::default()
    });
    s.load("p=[1;2] {p = [a;b] or p = [b;a]}").unwrap();
    assert_eq!(s.comp.derivs.len(), 2);
    s.load("a=b+1").unwrap();
    assert_eq!(s.comp.derivs.len(), 1);
    let d = &s.comp.derivs[0];
    assert_eq!(
        (value(d, "a", Notation::Absys1), value(d, "b", Notation::Absys1)),
        ("2".into(), "1".into())
    );
}

/// FE69: arithmetic "in the same spirit": a = b + 1, a = 1 + b, 1 + b = a and
/// b + 1 = a are equivalent; and each works whichever of a and b is known.
#[test]
fn fe69_arithmetic_in_every_direction() {
    for form in ["a = b + 1", "a = 1 + b", "1 + b = a", "b + 1 = a"] {
        let d = &r69(&format!("{form} b = 4")).comp.derivs[0];
        assert_eq!(value(d, "a", Notation::Absys1), "5", "{form}");
        let d = &r69(&format!("a = 5 {form}")).comp.derivs[0];
        assert_eq!(value(d, "b", Notation::Absys1), "4", "{form}");
    }
}

const ITEM: &str = "item'=lambda x=>z key x\n begin x = [p'&q'] {z=p or z=item(q)} end\n";

/// FE69: "the assertion i=item([1;2;3]) is equivalent to asserting
/// {i=1 or i=2 or i=3}".
#[test]
fn fe69_item() {
    let s = r69(&format!("{ITEM}i=item([1;2;3])"));
    let is: Vec<String> = s.comp.derivs.iter().map(|d| value(d, "i", Notation::Absys1)).collect();
    assert_eq!(is, vec!["1", "2", "3"]);
}

/// FE69: with p1, p2, p3 the lists [1;2;5;6], [3;7;1], [10;2;7;4], the
/// result "would be three distinct conjunctions of assertions with
/// associated data r, s, t 1, 3, 4, 1, 1, 2 and 6, 1, 7 respectively".
#[test]
fn fe69_triples() {
    let s = r69(&format!(
        "{ITEM}p1 = [1;2;5;6] p2 = [3;7;1] p3 = [10;2;7;4]\nr'=item(p1) s'=item(p2) t'=item(p3) r+s=t"
    ));
    let got: Vec<(String, String, String)> = s
        .comp
        .derivs
        .iter()
        .map(|d| {
            let v = |n| value(d, n, Notation::Absys1);
            (v("r"), v("s"), v("t"))
        })
        .collect();
    let want: Vec<(String, String, String)> = [("1", "3", "4"), ("1", "1", "2"), ("6", "1", "7")]
        .iter()
        .map(|(a, b, c)| (a.to_string(), b.to_string(), c.to_string()))
        .collect();
    assert_eq!(got, want);
}

/// FE69: map with key p terminates, asserting f of each item.
#[test]
fn fe69_map_with_key() {
    let s = r69(
        "map'=lambda p, f key p\n begin {null(p) or p = [r'&s'] f(r) map(s,f)} end\n\
         one'=lambda x begin x = 1 end\nmap([a;b;c], one)",
    );
    assert_eq!(s.comp.derivs.len(), 1);
    let d = &s.comp.derivs[0];
    for v in ["a", "b", "c"] {
        assert_eq!(value(d, v, Notation::Absys1), "1");
    }
    // without the key, map applied to a list not yet present waits for nothing:
    // its alternatives are elaborated at once, and it goes on for ever
    let mut s = Session::new(Opts {
        notation: Notation::Absys1,
        ..Opts::default()
    });
    s.budget = 2000;
    s.load("map'=lambda p, f\n begin {null(p) or p = [r'&s'] f(r) map(s,f)} end\none'=lambda x begin x = 1 end\nmap(l, one)")
        .unwrap();
    assert!(s.reports.last().unwrap().exhausted);
    // with it, the application waits for the list
    let s = r69("map'=lambda p, f key p\n begin {null(p) or p = [r'&s'] f(r) map(s,f)} end\none'=lambda x begin x = 1 end\nmap(l, one)");
    assert_eq!(s.comp.derivs.len(), 1);
    assert_eq!(s.comp.derivs[0].waiting().count(), 1);
}

/// E90 p. 8: one definition, used in several directions.
#[test]
fn e90_p8_one_procedure_in_every_direction() {
    let one = |goal: &str| r90(&format!("{APP}{goal}\n"));
    let s = one("app([1,2], [3], T)");
    assert_eq!(value(&s.comp.derivs[0], "T", Notation::Retrospective), "[1,2,3]");
    let s = one("app([1], S, [1,2,3])");
    assert_eq!(value(&s.comp.derivs[0], "S", Notation::Retrospective), "[2,3]");
    let s = one("app(R, [3], [1,2,3])");
    assert_eq!(s.comp.derivs.len(), 1);
    assert_eq!(value(&s.comp.derivs[0], "R", Notation::Retrospective), "[1,2]");
    let s = one("app(R, S, [1,2])");
    let parts: Vec<(String, String)> = s
        .comp
        .derivs
        .iter()
        .map(|d| {
            (
                value(d, "R", Notation::Retrospective),
                value(d, "S", Notation::Retrospective),
            )
        })
        .collect();
    assert_eq!(
        parts,
        vec![
            ("[]".into(), "[1,2]".into()),
            ("[1]".into(), "[2]".into()),
            ("[1,2]".into(), "[]".into())
        ]
    );
}
