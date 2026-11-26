//! An Absys session: a text, accepted an assertion at a time.
//!
//! "The on-line system is incremental in that, as assertions are accepted by
//! the system, whatever processing can be done on the basis of data already
//! present in the system is done" (Foster and Elcock 1969). So each
//! assertion is added to every derivation still alive, hibernating or not,
//! and the computation runs until every derivation hibernates or fails, or a
//! budget of steps is spent (a derivation can go on for ever; Absys's could
//! too).

use crate::compile;
use crate::machine::{Comp, Ctx, Deriv, Event, Notation, Opts, Susp, Why, V};
use crate::syntax::{self, Error, Item, Span};
use crate::syntax1969;
use std::rc::Rc;

pub const DEFAULT_BUDGET: usize = 100_000;

/// What happened when one assertion was accepted.
#[derive(Clone, Debug)]
pub struct Report {
    pub line: usize,
    pub span: Span,
    pub source: String,
    /// the events, as a range of `Session::events()`
    pub events: (usize, usize),
    pub steps: usize,
    /// the budget ran out with derivations still elaborating
    pub exhausted: bool,
    pub alive: usize,
    pub failed: usize,
}

pub struct Session {
    pub comp: Comp,
    pub cx: Ctx,
    pub budget: usize,
    pub reports: Vec<Report>,
    /// every derivation has failed: the text is unsatisfiable
    pub dead: bool,
}

impl Session {
    pub fn new(opts: Opts) -> Session {
        Session {
            comp: Comp::new(vec![Deriv::new(0)]),
            cx: Ctx::new(opts),
            budget: DEFAULT_BUDGET,
            reports: vec![],
            dead: false,
        }
    }

    pub fn notation(&self) -> Notation {
        self.cx.opts.notation
    }

    pub fn events(&self) -> &[Event] {
        &self.cx.events
    }

    /// Elaborates until every derivation hibernates or fails, or `budget` steps.
    /// Returns the steps taken and whether the budget ran out.
    pub fn run(&mut self, budget: usize) -> (usize, bool) {
        let start = self.cx.steps;
        while self.cx.steps - start < budget {
            if !self.comp.step(&mut self.cx) {
                break;
            }
        }
        if self.comp.derivs.is_empty() {
            self.dead = true;
        }
        (self.cx.steps - start, self.comp.running() > 0)
    }

    /// Accepts one assertion.
    pub fn accept(&mut self, text: &str, item: &Item) -> Result<&Report, Error> {
        let p = compile::assertion(text, &item.assertion)?;
        let ev0 = self.cx.events.len();
        let failed0 = self.cx.failed.len();
        for d in &mut self.comp.derivs {
            d.assert(&p);
        }
        let (steps, exhausted) = self.run(self.budget);
        self.reports.push(Report {
            line: item.line,
            span: item.span,
            source: text[item.span.start..item.span.end].to_string(),
            events: (ev0, self.cx.events.len()),
            steps,
            exhausted,
            alive: self.comp.derivs.len(),
            failed: self.cx.failed.len().saturating_sub(failed0),
        });
        Ok(self.reports.last().unwrap())
    }

    /// Reads a text and accepts each of its assertions in turn.
    pub fn load(&mut self, text: &str) -> Result<(), Error> {
        let items = parse(self.notation(), text)?;
        for item in &items {
            self.accept(text, item)?;
        }
        Ok(())
    }
}

pub fn parse(n: Notation, text: &str) -> Result<Vec<Item>, Error> {
    match n {
        Notation::Retrospective => syntax::parse(text),
        Notation::Absys1 => syntax1969::parse(text),
        Notation::Elcock1982 => syntax1969::parse_1982(text),
    }
}

/// The notation a file is in, from its name: `.ab1` is the 1969 notation,
/// `.ab2` the 1982 one.
pub fn notation_for(path: &str) -> Notation {
    if path.ends_with(".ab1") {
        Notation::Absys1
    } else if path.ends_with(".ab2") {
        Notation::Elcock1982
    } else {
        Notation::Retrospective
    }
}

/// One identifier of the text, as a query of a derivation answers it.
pub struct Answer {
    pub name: String,
    pub value: String,
    /// for a variable that is only constrained by suspended equations: the
    /// other identifiers of the text it must equal, and the equations that say so
    pub via: Vec<String>,
}

/// The answers of a derivation: every identifier of the text that is not a procedure.
pub fn answers(d: &Deriv, n: Notation) -> Vec<Answer> {
    let mut out = vec![];
    for (name, x) in &d.globals {
        if matches!(d.deref(&V::Var(*x)), V::Lam(_)) {
            continue;
        }
        let (value, via) = match d.deref(&V::Var(*x)) {
            V::Var(y) => {
                let class = equal_class(d, y);
                let others: Vec<String> = class
                    .iter()
                    .filter(|v| **v != y)
                    .filter(|v| d.globals.iter().any(|(_, g)| g == *v))
                    .map(|v| d.cells[*v as usize].name.to_string())
                    .collect();
                let eqs: Vec<String> = d
                    .waiting()
                    .filter(|s| s.why == Why::Equation && s.on.iter().any(|v| class.contains(v)))
                    .map(|s| d.show_goal(&s.goal, n))
                    .collect();
                if others.is_empty() && eqs.is_empty() {
                    (d.cells[y as usize].name.to_string(), vec![])
                } else if others.is_empty() {
                    (eqs[0].split(" = ").nth(1).unwrap_or("").to_string(), eqs)
                } else {
                    (others.join(" = "), eqs)
                }
            }
            v => (d.show(&v, n), vec![]),
        };
        out.push(Answer {
            name: name.to_string(),
            value,
            via,
        });
    }
    out
}

/// The variables a suspended equation between variables makes equal to `x`.
fn equal_class(d: &Deriv, x: u32) -> Vec<u32> {
    let mut class = vec![x];
    let mut i = 0;
    while i < class.len() {
        let v = class[i];
        for s in d.waiting().filter(|s| s.why == Why::Equation) {
            if s.on.contains(&v) {
                for w in &s.on {
                    if !class.contains(w) {
                        class.push(*w);
                    }
                }
            }
        }
        i += 1;
    }
    class
}

/// The procedures a derivation has: the identifiers instantiated to lambda expressions.
pub fn procedures(d: &Deriv) -> Vec<String> {
    d.globals
        .iter()
        .filter(|(_, x)| matches!(d.deref(&V::Var(*x)), V::Lam(_)))
        .map(|(n, _)| n.to_string())
        .collect()
}

pub fn describe_wait(d: &Deriv, s: &Susp, n: Notation) -> String {
    let g = d.show_goal(&s.goal, n);
    let on: Vec<String> =
        s.on.iter()
            .filter(|v| matches!(d.deref(&V::Var(**v)), V::Var(_)))
            .map(|v| d.cells[*v as usize].name.to_string())
            .collect();
    let why = match s.why {
        Why::Equation => return g,
        Why::Undefined => "waits for a definition",
        Why::Key => "waits for its key",
        Why::Arith => "waits for two of its numbers",
        Why::Tout => "waits for its term",
        Why::Negation => "waits for its variables",
        Why::Undecided => "cannot be decided yet; waits for",
    };
    format!("{g}  ({why}: {})", on.join(", "))
}

pub fn rc_text(s: &str) -> Rc<str> {
    Rc::from(s)
}
