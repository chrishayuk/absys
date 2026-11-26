//! A language server for Absys texts, over standard input and output
//! (`absys lsp`). `.abs` is the 1990 notation, `.ab1` the 1969 one.
//!
//! Problems as the text is typed: anything not in the notation, and every
//! assertion after which no derivation survives. Hover on an assertion shows
//! the computation after it was accepted: each derivation, its answers and
//! what is suspended; on an identifier, its value in each derivation. Colours
//! come as semantic tokens, so the same server colours the editor in VS Code
//! and in the playground.

use crate::json::{self, obj, Json};
use crate::machine::{Notation, Opts};
use crate::play::parse_partial;
use crate::session::{answers, describe_wait, notation_for, procedures, Session};
use crate::syntax::{line_col, Span};
use std::collections::HashMap;
use std::io::{BufRead, Write};

/// A smaller budget than the command line's: the server runs on every keystroke.
const BUDGET: usize = 20_000;

pub const TOKEN_TYPES: &[&str] = &[
    "comment",  // 0 % notes
    "string",   // 1 'BROWN' "U"
    "keyword",  // 2 lambda or not begin end key
    "number",   // 3
    "operator", // 4 = | & ; + - * /
    "variable", // 5 X, L, or (1969) any identifier
    "function", // 6 an identifier applied or defined: app(…), app = lambda
    "macro",    // 7 the primitives: tout, set, null
];

const KEYWORDS: &[&str] = &["lambda", "or", "and", "not", "begin", "end", "key"];
const PRIMITIVES: &[&str] = &["tout", "set", "null"];

/// (start, length, type) in characters, for one line.
pub fn line_tokens(text: &str, n: Notation) -> Vec<(usize, usize, usize)> {
    let cs: Vec<char> = text.chars().collect();
    let mut out = vec![];
    let mut i = 0;
    while i < cs.len() {
        let c = cs[i];
        if c == '%' {
            out.push((i, cs.len() - i, 0));
            break;
        }
        if c == '\'' || c == '"' {
            // in 1969, a quote straight after an identifier is a prime
            if c == '\'' && i > 0 && (cs[i - 1].is_alphanumeric() || cs[i - 1] == '_') {
                out.push((i, 1, 4));
                i += 1;
                continue;
            }
            let mut j = i + 1;
            while j < cs.len() && cs[j] != c {
                j += 1;
            }
            let end = (j + 1).min(cs.len());
            out.push((i, end - i, 1));
            i = end;
            continue;
        }
        if c.is_ascii_digit() {
            let mut j = i;
            while j < cs.len() && cs[j].is_ascii_digit() {
                j += 1;
            }
            out.push((i, j - i, 3));
            i = j;
            continue;
        }
        if c.is_ascii_alphabetic() || c == '_' {
            let mut j = i;
            while j < cs.len()
                && (cs[j].is_ascii_alphanumeric()
                    || cs[j] == '_'
                    || (n == Notation::Retrospective
                        && c.is_ascii_lowercase()
                        && cs[j] == '-'
                        && cs.get(j + 1).is_some_and(|d| d.is_ascii_alphabetic())))
            {
                j += 1;
            }
            let w: String = cs[i..j].iter().collect();
            let mut k = j;
            while k < cs.len() && cs[k] == ' ' {
                k += 1;
            }
            let next = cs.get(k).copied();
            let after_eq: String = cs[k..].iter().collect();
            let kind = if KEYWORDS.contains(&w.as_str()) {
                2
            } else if PRIMITIVES.contains(&w.as_str()) && next == Some('(') {
                7
            } else if next == Some('(')
                || after_eq.starts_with("= lambda")
                || after_eq.starts_with("=lambda")
                || after_eq.starts_with("'=lambda")
                || after_eq.starts_with("' = lambda")
            {
                6
            } else {
                5
            };
            out.push((i, j - i, kind));
            i = j;
            continue;
        }
        if "=|&;+-*/<>".contains(c) {
            out.push((i, if c == '=' && cs.get(i + 1) == Some(&'>') { 2 } else { 1 }, 4));
        }
        i += 1;
    }
    out
}

fn u16_col(text: &str, c: usize) -> usize {
    text.chars().take(c).map(|ch| ch.len_utf16()).sum()
}

fn char_at_u16(text: &str, col16: usize) -> usize {
    let mut n = 0;
    for (i, ch) in text.chars().enumerate() {
        if n >= col16 {
            return i;
        }
        n += ch.len_utf16();
    }
    text.chars().count()
}

fn position(line: usize, character: usize) -> Json {
    obj([("line", line.into()), ("character", character.into())])
}

fn range(a: (usize, usize), b: (usize, usize)) -> Json {
    obj([("start", position(a.0, a.1)), ("end", position(b.0, b.1))])
}

/// The text run, as the server sees it.
pub struct Analysis {
    pub session: Session,
    /// per accepted assertion: its span, the derivations alive after it, and a summary of the state
    pub states: Vec<(Span, String, usize, bool)>,
    pub problems: Vec<(Span, u8, String)>,
}

pub fn analyse(text: &str, n: Notation) -> Analysis {
    let (items, error) = parse_partial(n, text);
    let mut s = Session::new(Opts {
        notation: n,
        ..Opts::default()
    });
    s.budget = BUDGET;
    let mut states = vec![];
    let mut problems = vec![];
    for item in &items {
        let was_alive = !s.comp.derivs.is_empty();
        match s.accept(text, item) {
            Ok(r) => {
                let r = r.clone();
                let mut md = state_markdown(&s, n);
                if r.exhausted {
                    md.push_str(&format!(
                        "\n\n_The budget of {BUDGET} steps ran out with derivations still elaborating._"
                    ));
                    problems.push((
                        item.span,
                        3,
                        format!("{BUDGET} steps, and derivations are still elaborating (perhaps for ever)"),
                    ));
                }
                if was_alive && s.comp.derivs.is_empty() {
                    let why = s.cx.failed.last().map(|f| f.1.clone()).unwrap_or_default();
                    problems.push((
                        item.span,
                        2,
                        format!("after this assertion no derivation survives: the text is unsatisfiable ({why})"),
                    ));
                }
                states.push((item.span, md, r.alive, r.exhausted));
            }
            Err(e) => {
                problems.push((e.span, 1, e.message));
                break;
            }
        }
    }
    if let Some(e) = error {
        problems.push((e.span, 1, e.message));
    }
    Analysis {
        session: s,
        states,
        problems,
    }
}

fn state_markdown(s: &Session, n: Notation) -> String {
    let ds = &s.comp.derivs;
    if ds.is_empty() {
        return "**No derivation survives**: the assertions are unsatisfiable.".into();
    }
    let mut md = format!("**{} derivation{}**", ds.len(), if ds.len() == 1 { "" } else { "s" });
    for d in ds.iter().take(8) {
        md.push_str(&format!("\n\n#{} ", d.id));
        md.push_str(if d.is_running() {
            "_elaborating_"
        } else {
            "_hibernating_"
        });
        let procs = procedures(d);
        if !procs.is_empty() {
            md.push_str(&format!("  \nprocedures: `{}`", procs.join("`, `")));
        }
        for a in answers(d, n) {
            md.push_str(&format!("  \n`{} = {}`", a.name, a.value));
            if !a.via.is_empty() {
                md.push_str(&format!(" (suspended: `{}`)", a.via.join("`, `")));
            }
        }
        for w in d.waiting() {
            md.push_str(&format!("  \nwaiting: `{}`", describe_wait(d, w, n)));
        }
        for o in &d.outputs {
            md.push_str(&format!("  \noutput: `{o}`"));
        }
    }
    if ds.len() > 8 {
        md.push_str(&format!("\n\n… and {} more", ds.len() - 8));
    }
    md
}

pub fn problems(text: &str, n: Notation) -> Vec<(usize, usize, usize, usize, u8, String)> {
    analyse(text, n)
        .problems
        .into_iter()
        .map(|(sp, sev, m)| {
            let (l0, c0) = line_col(text, sp.start);
            let (l1, c1) = line_col(text, sp.end.max(sp.start + 1).min(text.len()));
            (l0, c0, l1, c1, sev, m)
        })
        .collect()
}

fn diagnostics(uri: &str, text: &str) -> Json {
    let n = notation_for(uri);
    let lines: Vec<&str> = text.lines().collect();
    let col = |l: usize, c: usize| lines.get(l).map_or(c, |s| u16_col(s, c));
    let list: Vec<Json> = problems(text, n)
        .into_iter()
        .map(|(l0, c0, l1, c1, sev, msg)| {
            obj([
                ("range", range((l0, col(l0, c0)), (l1, col(l1, c1)))),
                ("severity", (sev as usize).into()),
                ("source", "absys".into()),
                ("message", msg.into()),
            ])
        })
        .collect();
    obj([
        ("jsonrpc", "2.0".into()),
        ("method", "textDocument/publishDiagnostics".into()),
        ("params", obj([("uri", uri.into()), ("diagnostics", list.into())])),
    ])
}

fn capabilities() -> Json {
    obj([
        (
            "capabilities",
            obj([
                ("textDocumentSync", 1.into()),
                ("hoverProvider", true.into()),
                ("documentSymbolProvider", true.into()),
                (
                    "semanticTokensProvider",
                    obj([
                        (
                            "legend",
                            obj([
                                (
                                    "tokenTypes",
                                    TOKEN_TYPES.iter().map(|t| Json::from(*t)).collect::<Vec<_>>().into(),
                                ),
                                ("tokenModifiers", Vec::new().into()),
                            ]),
                        ),
                        ("full", true.into()),
                    ]),
                ),
                ("completionProvider", obj([("resolveProvider", false.into())])),
            ]),
        ),
        (
            "serverInfo",
            obj([("name", "absys".into()), ("version", env!("CARGO_PKG_VERSION").into())]),
        ),
    ])
}

/// (line, start, length, type) in UTF-16 columns, in order.
pub fn semantic(text: &str, n: Notation) -> Vec<(usize, usize, usize, usize)> {
    let mut out = vec![];
    for (k, l) in text.lines().enumerate() {
        for (s, len, kind) in line_tokens(l, n) {
            let a = u16_col(l, s);
            let b = u16_col(l, s + len);
            if b > a {
                out.push((k, a, b - a, kind));
            }
        }
    }
    out
}

fn semantic_tokens(text: &str, n: Notation) -> Json {
    let mut data = Vec::new();
    let (mut last_line, mut last_start) = (0, 0);
    for (line, start, length, kind) in semantic(text, n) {
        let delta = if line == last_line { start - last_start } else { start };
        data.extend([line - last_line, delta, length, kind, 0].map(Json::from));
        (last_line, last_start) = (line, start);
    }
    obj([("data", data.into())])
}

/// The word at a place, and the byte offset of the place.
fn word_at(text: &str, line: usize, col16: usize) -> (String, usize) {
    let mut off = 0;
    for (k, l) in text.split('\n').enumerate() {
        if k == line {
            let c = char_at_u16(l, col16);
            let cs: Vec<char> = l.chars().collect();
            let is_w = |ch: char| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-';
            let mut a = c.min(cs.len());
            while a > 0 && is_w(cs[a - 1]) {
                a -= 1;
            }
            let mut b = c.min(cs.len());
            while b < cs.len() && is_w(cs[b]) {
                b += 1;
            }
            let byte: usize = cs[..c.min(cs.len())].iter().map(|ch| ch.len_utf8()).sum();
            return (cs[a..b].iter().collect(), off + byte);
        }
        off += l.len() + 1;
    }
    (String::new(), text.len())
}

pub fn hover(text: &str, n: Notation, line: usize, col16: usize) -> Option<String> {
    let a = analyse(text, n);
    let (word, at) = word_at(text, line, col16);
    if let Some(d) = keyword_help(&word) {
        return Some(d.to_string());
    }
    let (_, md, _, _) = a.states.iter().find(|(sp, ..)| sp.start <= at && at <= sp.end)?;
    let mut out = String::new();
    if !word.is_empty() {
        let mut vals = vec![];
        for d in &a.session.comp.derivs {
            if let Some(ans) = answers(d, n).into_iter().find(|x| x.name == word) {
                vals.push(format!("#{}: `{} = {}`", d.id, ans.name, ans.value));
            } else if procedures(d).contains(&word) {
                vals.push(format!("#{}: `{word}` is a procedure", d.id));
                break;
            }
        }
        if !vals.is_empty() {
            out.push_str(&format!(
                "**{word}**, at the end of the text  \n{}\n\n---\n\n",
                vals.join("  \n")
            ));
        }
    }
    out.push_str("After this assertion: ");
    out.push_str(md);
    Some(out)
}

fn keyword_help(w: &str) -> Option<&'static str> {
    Some(match w {
        "lambda" => "**lambda** bound variables, then a body: a procedure. `app = lambda R, S, T …` instantiates `app` to it (Elcock 1990 p. 8).",
        "or" => "**or**: alternatives. Elaborating it distributes it through the goal: one derivation for each alternative, elaborated in parallel; those found unsatisfiable disappear (p. 9; Foster and Elcock 1969).",
        "not" => "**not**: an independent computation from the bindings as they stand, with the criteria reversed: it holds if every derivation of it fails (p. 14). Not safe: it does not wait for its variables (p. 15, note 18).",
        "begin" | "end" => "**begin … end** (1969): a conjunction; the scope of the names primed inside it.",
        "key" => "**key** (1969): the bound variables that must be present before an application is elaborated (Foster and Elcock 1969).",
        "tout" => "**tout(\"U\", U)**: output (Elcock 1990 p. 10, note 11). Here it writes when its term is complete.",
        "set" => "**{set(X), {…}, S}**: S is the set of the X for which the middle holds: Absys-4's aggregation operator (p. 15).",
        "null" => "**null(p)** (1969): p is the empty list.",
        _ => return None,
    })
}

fn completion(text: &str, n: Notation) -> Json {
    let mut items: Vec<Json> = ["lambda", "or", "not", "tout"]
        .iter()
        .chain(match n {
            Notation::Retrospective => ["set"].iter(),
            Notation::Absys1 => ["begin", "end", "key", "null"][..].iter(),
            Notation::Elcock1982 => ["key", "and"][..].iter(),
        })
        .map(|k| {
            obj([
                ("label", (*k).into()),
                ("kind", 14.into()),
                ("detail", keyword_help(k).unwrap_or("").into()),
            ])
        })
        .collect();
    // the procedures the text defines, whether or not any derivation survives
    for p in symbols(text, n).items() {
        let (name, detail) = (p.get("name").clone(), p.get("detail").clone());
        items.push(obj([("label", name), ("kind", 3.into()), ("detail", detail)]));
    }
    items.into()
}

fn symbols(text: &str, n: Notation) -> Json {
    let (items, _) = parse_partial(n, text);
    let mut out = vec![];
    for it in items {
        if let crate::syntax::Assertion::Eq(l, crate::syntax::Expr::Lambda(lam), _) = &it.assertion {
            let name = match l {
                crate::syntax::Expr::Name(x, _) | crate::syntax::Expr::Var(x, _) => x.clone(),
                _ => continue,
            };
            let params: Vec<&str> = lam.params.iter().map(|p| p.0.as_str()).collect();
            let a = line_col(text, it.span.start);
            let b = line_col(text, it.span.end);
            out.push(obj([
                ("name", name.into()),
                ("detail", format!("lambda {}", params.join(", ")).into()),
                ("kind", 12.into()),
                ("range", range(a, b)),
                ("selectionRange", range(a, a)),
            ]));
        }
    }
    out.into()
}

fn answer(method: &str, params: &Json, docs: &HashMap<String, String>) -> Option<Json> {
    let uri = params.get("textDocument").get("uri").str().unwrap_or("");
    let n = notation_for(uri);
    let text = docs.get(uri).map(String::as_str).unwrap_or("");
    let line = params.get("position").get("line").num().unwrap_or(0);
    let col = params.get("position").get("character").num().unwrap_or(0);
    Some(match method {
        "initialize" => capabilities(),
        "shutdown" => Json::Null,
        "textDocument/hover" => match hover(text, n, line, col) {
            Some(h) => obj([("contents", obj([("kind", "markdown".into()), ("value", h.into())]))]),
            None => Json::Null,
        },
        "textDocument/semanticTokens/full" => semantic_tokens(text, n),
        "textDocument/documentSymbol" => symbols(text, n),
        "textDocument/completion" => completion(text, n),
        _ => return None,
    })
}

fn read_message(input: &mut impl BufRead) -> Option<Json> {
    let mut length = None;
    loop {
        let mut line = String::new();
        if input.read_line(&mut line).ok()? == 0 {
            return None;
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                length = value.trim().parse::<usize>().ok();
            }
        }
    }
    let mut body = vec![0; length?];
    input.read_exact(&mut body).ok()?;
    json::parse(&String::from_utf8_lossy(&body)).ok()
}

fn send(output: &mut impl Write, message: Json) {
    let mut body = String::new();
    message.write(&mut body);
    let _ = write!(output, "Content-Length: {}\r\n\r\n{}", body.len(), body);
    let _ = output.flush();
}

pub fn serve(input: &mut impl BufRead, output: &mut impl Write) -> Result<(), String> {
    let mut docs: HashMap<String, String> = HashMap::new();
    let mut shutting_down = false;
    while let Some(message) = read_message(input) {
        let method = message.get("method").str().unwrap_or("").to_string();
        let params = message.get("params");
        let id = message.get("id").clone();
        let uri = params.get("textDocument").get("uri").str().unwrap_or("").to_string();
        match method.as_str() {
            "exit" => {
                return if shutting_down {
                    Ok(())
                } else {
                    Err("exit without shutdown".into())
                }
            }
            "textDocument/didOpen" => {
                let text = params.get("textDocument").get("text").str().unwrap_or("").to_string();
                send(output, diagnostics(&uri, &text));
                docs.insert(uri, text);
            }
            "textDocument/didChange" => {
                if let Some(text) = params
                    .get("contentChanges")
                    .items()
                    .last()
                    .and_then(|c| c.get("text").str())
                {
                    send(output, diagnostics(&uri, text));
                    docs.insert(uri, text.to_string());
                }
            }
            "textDocument/didClose" => {
                docs.remove(&uri);
                send(output, diagnostics(&uri, ""));
            }
            _ if id == Json::Null => {}
            _ => {
                shutting_down |= method == "shutdown";
                let reply = match answer(&method, params, &docs) {
                    Some(result) => obj([("jsonrpc", "2.0".into()), ("id", id), ("result", result)]),
                    None => obj([
                        ("jsonrpc", "2.0".into()),
                        ("id", id),
                        (
                            "error",
                            obj([
                                ("code", Json::Num(-32601.0)),
                                ("message", format!("method not found: {method}").into()),
                            ]),
                        ),
                    ]),
                };
                send(output, reply);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const APP: &str = include_str!("../examples/append.abs");

    #[test]
    fn hover_shows_the_state_after_an_assertion() {
        let line = APP.lines().position(|l| l.starts_with("app([1]")).unwrap();
        let h = hover(APP, Notation::Retrospective, line, 9).unwrap();
        assert!(h.contains("U = [2]"), "{h}");
    }

    #[test]
    fn tokens_and_problems() {
        let t = line_tokens("app = lambda R, S, T  % note", Notation::Retrospective);
        assert_eq!(t[0], (0, 3, 6));
        assert!(t.iter().any(|x| x.2 == 2));
        assert!(t.iter().any(|x| x.2 == 0));
        assert!(problems(APP, Notation::Retrospective).is_empty());
        let bad = format!("{APP}\nU = [3]\n");
        let p = problems(&bad, Notation::Retrospective);
        assert!(
            p.iter().any(|x| x.4 == 2 && x.5.contains("no derivation survives")),
            "{p:?}"
        );
    }
}
