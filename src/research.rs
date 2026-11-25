//! Tools for the experiments of `docs/research.md`. These are new, and not
//! part of the reconstruction of Absys: they look at it from outside.
//!
//! * `absys permute FILE` accepts the assertions of a text in every order,
//!   and reports which orders give which answers, and at what cost (E02).
//! * `absys graph FILE` writes the computation as a graph in Graphviz's dot
//!   language: the tree of derivations the `or`s made, and, for each
//!   derivation left, its suspended processes and the variables they wait on
//!   (E05).

use crate::machine::{Deriv, Opts, Status, Why, V};
use crate::session::{answers, describe_wait, parse, Session};
use crate::syntax::{Error, Item};
use std::collections::BTreeMap;

/// What a computation came to: one line for each derivation left, sorted, so
/// that two orders of the same text can be compared.
pub fn outcome(s: &Session) -> Vec<String> {
    let n = s.notation();
    let mut v: Vec<String> = s
        .comp
        .derivs
        .iter()
        .map(|d| {
            let mut parts: Vec<String> = answers(d, n)
                .into_iter()
                .filter(|a| !a.name.starts_with('_'))
                .map(|a| {
                    // a value that is an internal variable's name is not comparable between orders
                    if a.value == a.name
                        || a.value.chars().any(|c| ('₀'..='₉').contains(&c))
                        || a.value.starts_with('_')
                    {
                        format!("{} uninstantiated", a.name)
                    } else {
                        format!("{} = {}", a.name, a.value)
                    }
                })
                .collect();
            parts.sort();
            let waits = d.waiting().count();
            if waits > 0 {
                parts.push(format!("{waits} suspended"));
            }
            if d.status == Status::Running {
                parts.push("still elaborating".into());
            }
            parts.join(", ")
        })
        .collect();
    v.sort();
    v
}

fn permutations(n: usize) -> Vec<Vec<usize>> {
    if n == 0 {
        return vec![vec![]];
    }
    let mut out = vec![];
    for p in permutations(n - 1) {
        for i in 0..=p.len() {
            let mut q = p.clone();
            q.insert(i, n - 1);
            out.push(q);
        }
    }
    out.sort();
    out
}

pub struct Order {
    pub order: Vec<usize>,
    pub steps: usize,
    pub outcome: Vec<String>,
    pub exhausted: bool,
}

pub fn permute(text: &str, opts: Opts, budget: usize) -> Result<(Vec<Item>, Vec<Order>), Error> {
    let items = parse(opts.notation, text)?;
    if items.len() > 8 {
        return Err(Error {
            span: items[8].span,
            message: format!(
                "{} assertions: at most 8 can be permuted (8! = 40320 orders)",
                items.len()
            ),
        });
    }
    let mut out = vec![];
    for order in permutations(items.len()) {
        let mut s = Session::new(opts);
        s.budget = budget;
        let mut exhausted = false;
        for &i in &order {
            exhausted |= s.accept(text, &items[i])?.exhausted;
        }
        out.push(Order {
            steps: s.cx.steps,
            outcome: outcome(&s),
            exhausted,
            order,
        });
    }
    Ok((items, out))
}

pub fn permute_text(text: &str, opts: Opts, budget: usize) -> Result<String, Error> {
    let (items, orders) = permute(text, opts, budget)?;
    let mut groups: BTreeMap<Vec<String>, Vec<&Order>> = BTreeMap::new();
    for o in &orders {
        groups.entry(o.outcome.clone()).or_default().push(o);
    }
    let mut out = format!("{} assertions, {} orders:\n", items.len(), orders.len());
    for (k, it) in items.iter().enumerate() {
        let src = &text[it.span.start..it.span.end];
        let first = src.lines().next().unwrap_or("");
        let more = if src.lines().count() > 1 { " …" } else { "" };
        out.push_str(&format!("  {}  {first}{more}\n", k + 1));
    }
    out.push_str(&format!(
        "\n{} distinct outcome{}:\n",
        groups.len(),
        if groups.len() == 1 { "" } else { "s" }
    ));
    for (outcome, os) in &groups {
        let steps: Vec<usize> = os.iter().map(|o| o.steps).collect();
        let (lo, hi) = (steps.iter().min().unwrap(), steps.iter().max().unwrap());
        let ex = os.iter().filter(|o| o.exhausted).count();
        out.push_str(&format!(
            "\n  {} order{} ({}–{} steps{}), e.g. {}\n",
            os.len(),
            if os.len() == 1 { "" } else { "s" },
            lo,
            hi,
            if ex > 0 {
                format!("; {ex} ran out of budget")
            } else {
                String::new()
            },
            os[0]
                .order
                .iter()
                .map(|i| (i + 1).to_string())
                .collect::<Vec<_>>()
                .join(" ")
        ));
        if outcome.is_empty() {
            out.push_str("    no derivation survives\n");
        }
        for line in outcome {
            out.push_str(&format!("    {line}\n"));
        }
    }
    Ok(out)
}

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// The computation as a graph, in dot.
pub fn graph(s: &Session) -> String {
    let n = s.notation();
    let mut out = String::from("digraph absys {\n  rankdir=TB;\n  node [fontname=\"Helvetica\" fontsize=10];\n");
    out.push_str(
        "  subgraph cluster_derivations {\n    label=\"derivations: each or distributes into one per alternative\";\n",
    );
    let alive: BTreeMap<usize, &Deriv> = s.comp.derivs.iter().map(|d| (d.id, d)).collect();
    let failed: BTreeMap<usize, &String> = s.cx.failed.iter().map(|(i, w)| (*i, w)).collect();
    let mut seen = vec![0usize];
    for (p, kids) in &s.cx.tree {
        seen.push(*p);
        seen.extend(kids);
    }
    seen.sort();
    seen.dedup();
    for id in &seen {
        let (label, style) = if let Some(d) = alive.get(id) {
            let ans: Vec<String> = answers(d, n)
                .into_iter()
                .map(|a| esc(&format!("{} = {}", a.name, a.value)))
                .collect();
            let st = if d.status == Status::Hibernating {
                "hibernating"
            } else {
                "elaborating"
            };
            (
                format!("#{id} {st}\\n{}", ans.join("\\n")),
                "style=filled fillcolor=\"#d9f2d9\"",
            )
        } else if let Some(w) = failed.get(id) {
            let w: String = w.chars().take(60).collect();
            (
                format!("#{id} failed\\n{}", esc(&w)),
                "style=filled fillcolor=\"#f6d5d5\"",
            )
        } else {
            (format!("#{id}"), "")
        };
        out.push_str(&format!("    d{id} [shape=box label=\"{label}\" {style}];\n"));
    }
    for (p, kids) in &s.cx.tree {
        for k in kids {
            out.push_str(&format!("    d{p} -> d{k};\n"));
        }
    }
    out.push_str("  }\n");
    for d in s.comp.derivs.iter().take(12) {
        let waits: Vec<_> = d.waiting().collect();
        if waits.is_empty() {
            continue;
        }
        out.push_str(&format!(
            "  subgraph cluster_c{} {{\n    label=\"#{}: what is suspended, and on what\";\n",
            d.id, d.id
        ));
        let mut vars = vec![];
        for (k, w) in waits.iter().enumerate() {
            let live: Vec<u32> =
                w.on.iter()
                    .copied()
                    .filter(|v| matches!(d.deref(&V::Var(*v)), V::Var(_)))
                    .collect();
            if w.why == Why::Equation && live.len() == 2 {
                // an equation between variables is an edge between them
                out.push_str(&format!(
                    "    v{}_{} -> v{}_{} [dir=none style=dashed label=\"=\"];\n",
                    d.id, live[0], d.id, live[1]
                ));
            } else {
                out.push_str(&format!(
                    "    p{}_{k} [shape=note label=\"{}\"];\n",
                    d.id,
                    esc(&describe_wait(d, w, n))
                ));
                for v in &live {
                    out.push_str(&format!("    p{}_{k} -> v{}_{v} [style=dotted];\n", d.id, d.id));
                }
            }
            vars.extend(live);
        }
        vars.sort();
        vars.dedup();
        for v in vars {
            out.push_str(&format!(
                "    v{}_{v} [shape=ellipse label=\"{}\"];\n",
                d.id,
                esc(&d.cells[v as usize].name)
            ));
        }
        out.push_str("  }\n");
    }
    out.push_str("}\n");
    out
}
