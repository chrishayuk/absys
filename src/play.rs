//! The run of a text as JSON, for the playground (`absys json`): the state of
//! the computation after each assertion is accepted, with what happened on the
//! way.

use crate::json::{obj, Json};
use crate::machine::{Deriv, Notation, Opts, Status, Why};
use crate::session::{answers, describe_wait, procedures, Session};
use crate::syntax::{line_col, Error, Item};

const MAX_DERIVS: usize = 48;
const MAX_EVENTS: usize = 400;
/// entries of a binding environment, and of a derivation's goal, sent to the page
const MAX_ENV: usize = 80;
const MAX_PENDING: usize = 8;

pub fn deriv_json(d: &Deriv, n: Notation) -> Json {
    let answers: Vec<Json> = answers(d, n)
        .into_iter()
        .map(|a| {
            obj([
                ("name", a.name.into()),
                ("value", a.value.into()),
                ("via", a.via.into_iter().map(Json::from).collect::<Vec<_>>().into()),
            ])
        })
        .collect();
    let suspended: Vec<Json> = d
        .waiting()
        .map(|s| {
            let why = match s.why {
                Why::Equation => "equation",
                Why::Undefined => "undefined",
                Why::Key => "key",
                Why::Arith => "arithmetic",
                Why::Tout => "tout",
                Why::Negation => "negation",
                Why::Undecided => "undecided",
            };
            obj([("text", describe_wait(d, s, n).into()), ("why", why.into())])
        })
        .collect();
    let env: Vec<Json> = d
        .environment(n)
        .into_iter()
        .take(MAX_ENV)
        .map(|(k, v)| Json::Arr(vec![k.into(), v.into()]))
        .collect();
    obj([
        ("id", d.id.into()),
        ("parent", d.parent.map(Json::from).unwrap_or(Json::Null)),
        (
            "status",
            match d.status {
                Status::Hibernating => "hibernating",
                Status::Running => "elaborating",
            }
            .into(),
        ),
        (
            "outputs",
            d.outputs
                .iter()
                .map(|o| Json::from(o.as_str()))
                .collect::<Vec<_>>()
                .into(),
        ),
        (
            "procedures",
            procedures(d).into_iter().map(Json::from).collect::<Vec<_>>().into(),
        ),
        ("answers", answers.into()),
        ("suspended", suspended.into()),
        (
            "pending",
            d.pending(n)
                .into_iter()
                .take(MAX_PENDING)
                .map(Json::from)
                .collect::<Vec<_>>()
                .into(),
        ),
        ("environment", env.into()),
    ])
}

fn error_json(text: &str, e: &Error) -> Json {
    let (line, col) = line_col(text, e.span.start);
    obj([
        ("line", (line + 1).into()),
        ("column", (col + 1).into()),
        ("start", e.span.start.into()),
        ("end", e.span.end.into()),
        ("message", e.message.clone().into()),
    ])
}

/// The assertions before the first error, and the error.
pub fn parse_partial(n: Notation, text: &str) -> (Vec<Item>, Option<Error>) {
    match crate::session::parse(n, text) {
        Ok(items) => (items, None),
        Err(e) => {
            let cut = text[..e.span.start.min(text.len())]
                .rfind('\n')
                .map(|k| k + 1)
                .unwrap_or(0);
            // the assertion the error is in may have begun on an earlier line: back off until it parses
            let mut cut = cut;
            loop {
                if let Ok(items) = crate::session::parse(n, &text[..cut]) {
                    return (items, Some(e));
                }
                match text[..cut.saturating_sub(1)].rfind('\n') {
                    Some(k) if cut > 0 => cut = k + 1,
                    _ => return (vec![], Some(e)),
                }
            }
        }
    }
}

pub fn run_json(text: &str, opts: Opts, budget: usize) -> String {
    let n = opts.notation;
    let (items, mut error) = parse_partial(n, text);
    let mut s = Session::new(opts);
    s.budget = budget;
    let mut steps = vec![];
    for item in &items {
        let r = match s.accept(text, item) {
            Ok(r) => r.clone(),
            Err(e) => {
                error = Some(e);
                break;
            }
        };
        let (line, _) = line_col(text, r.span.start);
        let (end_line, _) = line_col(text, r.span.end);
        let evs = &s.events()[r.events.0..r.events.1];
        let events: Vec<Json> = evs
            .iter()
            .take(MAX_EVENTS)
            .map(|e| {
                obj([
                    ("deriv", e.deriv.into()),
                    ("depth", e.depth.into()),
                    ("kind", e.kind.into()),
                    ("text", e.text.clone().into()),
                ])
            })
            .collect();
        let derivs: Vec<Json> = s
            .comp
            .derivs
            .iter()
            .take(MAX_DERIVS)
            .map(|d| deriv_json(d, n))
            .collect();
        steps.push(obj([
            ("line", (line + 1).into()),
            ("endLine", (end_line + 1).into()),
            ("start", r.span.start.into()),
            ("end", r.span.end.into()),
            ("source", r.source.clone().into()),
            ("steps", r.steps.into()),
            ("exhausted", r.exhausted.into()),
            ("failed", r.failed.into()),
            ("alive", r.alive.into()),
            ("events", events.into()),
            ("eventCount", evs.len().into()),
            ("derivations", derivs.into()),
        ]));
    }
    let mut out = String::new();
    obj([
        (
            "notation",
            match n {
                Notation::Retrospective => "1990",
                Notation::Absys1 => "1969",
            }
            .into(),
        ),
        (
            "error",
            error.as_ref().map(|e| error_json(text, e)).unwrap_or(Json::Null),
        ),
        ("budget", budget.into()),
        ("steps", steps.into()),
    ])
    .write(&mut out);
    out
}
