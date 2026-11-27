//! What `absys FILE` prints.

use crate::machine::{Deriv, Measure, Notation, Status};
use crate::session::{answers, describe_wait, procedures, Session};

#[derive(Clone, Copy, Default)]
pub struct Show {
    /// every event, under the assertion that caused it
    pub trace: bool,
    /// the whole binding environment of each derivation, as the retrospective writes it
    pub env: bool,
    /// the state after each assertion, not only at the end
    pub each: bool,
}

pub fn derivation(d: &Deriv, n: Notation, show: Show, out: &mut String) {
    let status = match d.status {
        Status::Hibernating => "hibernating",
        Status::Running => "still elaborating",
    };
    out.push_str(&format!("  derivation #{} ({status})\n", d.id));
    for line in &d.outputs {
        out.push_str(&format!("    output      {line}\n"));
    }
    let procs = procedures(d);
    if !procs.is_empty() {
        out.push_str(&format!("    procedures  {}\n", procs.join(", ")));
    }
    for a in answers(d, n) {
        let via = if a.via.is_empty() {
            String::new()
        } else {
            format!("   (suspended: {})", a.via.join(", "))
        };
        if a.value == a.name && a.via.is_empty() {
            out.push_str(&format!("    {}  uninstantiated\n", a.name));
        } else {
            out.push_str(&format!("    {} = {}{via}\n", a.name, a.value));
        }
    }
    let waits: Vec<String> = d.waiting().map(|s| describe_wait(d, s, n)).collect();
    if !waits.is_empty() {
        out.push_str(&format!("    suspended   {}\n", waits.join(";  ")));
    }
    let pending = d.pending(n);
    if !pending.is_empty() {
        let more = if pending.len() > 6 {
            format!(";  … {} more", pending.len() - 6)
        } else {
            String::new()
        };
        out.push_str(&format!(
            "    to do       {}{more}\n",
            pending[..pending.len().min(6)].join(";  ")
        ));
    }
    if show.env {
        let env: Vec<String> = d.environment(n).into_iter().map(|(k, v)| format!("{k}/{v}")).collect();
        out.push_str(&format!("    environment {}\n", env.join(", ")));
    }
}

pub fn state(s: &Session, show: Show, out: &mut String) {
    let n = s.notation();
    let derivs = &s.comp.derivs;
    if derivs.is_empty() {
        out.push_str("No derivation survives: the assertions are unsatisfiable.\n");
        return;
    }
    let running = s.comp.running();
    out.push_str(&format!(
        "{} derivation{}{}:\n",
        derivs.len(),
        if derivs.len() == 1 { "" } else { "s" },
        if running > 0 {
            format!(" ({running} still elaborating when the budget ran out)")
        } else {
            String::new()
        }
    ));
    let shown = if crate::machine::FULL.load(std::sync::atomic::Ordering::Relaxed) {
        usize::MAX
    } else {
        MAX_SHOWN
    };
    for d in derivs.iter().take(shown) {
        derivation(d, n, show, out);
    }
    if derivs.len() > shown {
        out.push_str(&format!("  … and {} more\n", derivs.len() - shown));
    }
}

/// The most derivations a report lists.
pub const MAX_SHOWN: usize = 24;

/// Accepts a text an assertion at a time, printing as it goes.
pub fn run(s: &mut Session, text: &str, show: Show) -> Result<String, crate::syntax::Error> {
    let items = crate::session::parse(s.notation(), text)?;
    let mut out = String::new();
    for item in &items {
        let r = s.accept(text, item)?.clone();
        if show.trace || show.each {
            out.push_str(&format!("» {}\n", r.source.replace('\n', "\n  ")));
        }
        if show.trace {
            for e in &s.events()[r.events.0..r.events.1] {
                out.push_str(&format!(
                    "  {}#{:<4} {:<9} {}\n",
                    "  ".repeat(e.depth),
                    e.deriv,
                    e.kind,
                    e.text
                ));
            }
        }
        if r.exhausted {
            out.push_str(&format!(
                "  (the budget of {} steps ran out with derivations still elaborating)\n",
                s.budget
            ));
        }
        if show.each {
            let mut st = String::new();
            state(s, show, &mut st);
            for l in st.lines() {
                out.push_str("  ");
                out.push_str(l);
                out.push('\n');
            }
            out.push('\n');
        }
    }
    if !show.each {
        state(s, show, &mut out);
    }
    Ok(out)
}

/// With `--measure`: one table row per assertion and quantity, as
/// tab-separated `line  assertion  steps  quantity  now  peak`. The peak is
/// the largest value seen after any step of that assertion: exact at the
/// granularity of steps.
pub fn measures(s: &Session) -> String {
    let mut out = String::from("line\tassertion\tsteps\tquantity\tnow\tpeak\n");
    for r in &s.reports {
        let Some((now, peak)) = r.measure else { continue };
        let src: String = r.source.split_whitespace().collect::<Vec<_>>().join(" ");
        for (k, name) in Measure::FIELDS.iter().enumerate() {
            out.push_str(&format!(
                "{}\t{}\t{}\t{}\t{}\t{}\n",
                r.line,
                src,
                r.steps,
                name,
                now.values()[k],
                peak.values()[k]
            ));
        }
    }
    out
}
