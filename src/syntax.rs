//! The notation: tokens, the tree of an assertion, and the parser.
//!
//! The surface is the one Elcock's 1990 retrospective uses to present Absys,
//! which he says takes "some slight liberties ... with the original syntax"
//! (p. 3). Everything it shows is accepted: `name = lambda R, S, T body`,
//! `{…}` for a conjunction, `or`, `[X|L]` and `[1, 2]`, quoted constants
//! (`'BROWN'`), names with hyphens (`math-course`), `not(…)`, the aggregation
//! `{set(X), {…}, S}` and the output assertion `tout("U", U)`. Where it is
//! silent the choice made here is in `docs/decisions.md`.
//!
//! A text is a conjunction of assertions. Nothing separates them: an
//! assertion ends where the next token cannot continue it (a `;` may be
//! written, and is ignored). `%` starts a note that runs to the end of the
//! line; notes are not Absys.

use std::fmt;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn to(self, o: Span) -> Span {
        Span {
            start: self.start.min(o.start),
            end: self.end.max(o.end),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    /// `X`, `L1`, `S2`: begins with a capital
    Var(String),
    /// `app`, `rev1`, `math-course`: begins with a small letter
    Name(String),
    Num(i128),
    /// `'BROWN'`
    Atom(String),
    /// `"U"`
    Str(String),
    Kw(&'static str),
    Sym(&'static str),
}

#[derive(Clone, Debug)]
pub struct Token {
    pub tok: Tok,
    pub span: Span,
    pub line: usize,
}

pub const KEYWORDS: &[&str] = &["lambda", "or", "not"];
const SYMBOLS: &[&str] = &["=", "(", ")", "[", "]", "{", "}", ",", "|", ";"];

#[derive(Clone, Debug, PartialEq)]
pub struct Error {
    pub span: Span,
    pub message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.message)
    }
}

fn err<T>(span: Span, message: impl Into<String>) -> Result<T, Error> {
    Err(Error {
        span,
        message: message.into(),
    })
}

/// The line (from 0) and column (in characters, from 0) of a byte offset.
pub fn line_col(text: &str, at: usize) -> (usize, usize) {
    let at = at.min(text.len());
    let before = &text[..at];
    let line = before.matches('\n').count();
    let col = before[before.rfind('\n').map(|n| n + 1).unwrap_or(0)..].chars().count();
    (line, col)
}

pub fn lex(text: &str) -> Result<Vec<Token>, Error> {
    let b = text.as_bytes();
    let mut out = vec![];
    let (mut i, mut line) = (0, 0);
    while i < b.len() {
        let c = b[i];
        if c == b'\n' {
            line += 1;
            i += 1;
            continue;
        }
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if c == b'%' {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        let start = i;
        let tok = if c.is_ascii_alphabetic() || c == b'_' {
            let small = c.is_ascii_lowercase();
            while i < b.len() {
                if b[i].is_ascii_alphanumeric() || b[i] == b'_' {
                    i += 1;
                } else if small && b[i] == b'-' && b.get(i + 1).is_some_and(|d| d.is_ascii_alphabetic()) {
                    // math-course, non-maths-major (p. 15): a hyphen between
                    // letters is part of a name. Absys has no subtraction to
                    // confuse it with.
                    i += 1;
                } else {
                    break;
                }
            }
            let w = &text[start..i];
            match KEYWORDS.iter().find(|k| **k == w) {
                Some(k) => Tok::Kw(k),
                None if small => Tok::Name(w.to_string()),
                None => Tok::Var(w.to_string()),
            }
        } else if c.is_ascii_digit() {
            while i < b.len() && b[i].is_ascii_digit() {
                i += 1;
            }
            match text[start..i].parse() {
                Ok(n) => Tok::Num(n),
                Err(_) => return err(Span { start, end: i }, "a number too large"),
            }
        } else if c == b'\'' || c == b'"' {
            i += 1;
            while i < b.len() && b[i] != c && b[i] != b'\n' {
                i += 1;
            }
            if i >= b.len() || b[i] != c {
                return err(
                    Span { start, end: i },
                    format!("a constant with no closing {}", c as char),
                );
            }
            i += 1;
            let s = text[start + 1..i - 1].to_string();
            if c == b'\'' {
                Tok::Atom(s)
            } else {
                Tok::Str(s)
            }
        } else if let Some(s) = SYMBOLS.iter().find(|s| text[i..].starts_with(**s)) {
            i += s.len();
            Tok::Sym(s)
        } else {
            let ch = text[i..].chars().next().unwrap();
            let end = i + ch.len_utf8();
            let hint = match ch {
                '+' | '*' | '/' | '<' | '>' => {
                    " (Absys, as the sources show it, has no arithmetic: see docs/decisions.md)"
                }
                '-' => " (a hyphen is part of a name only between letters, as in math-course)",
                '‘' | '’' | '“' | '”' => " (write the quotes as ' or \")",
                _ => "",
            };
            return err(Span { start, end }, format!("`{ch}` is not in the notation{hint}"));
        };
        out.push(Token {
            tok,
            span: Span { start, end: i },
            line,
        });
    }
    Ok(out)
}

/// A term: a constant, a list, a variable or a lambda expression.
#[derive(Clone, Debug)]
pub enum Expr {
    /// `X`: a variable.
    Var(String, Span),
    /// `app`: an identifier written with a small letter. Like `X`, it is a
    /// variable; `app = lambda …` instantiates it.
    Name(String, Span),
    Num(i128, Span),
    /// `'BROWN'`
    Atom(String, Span),
    /// `"U"`
    Str(String, Span),
    /// `[a, b | T]`
    List(Vec<Expr>, Option<Box<Expr>>, Span),
    Lambda(Rc<Lambda>),
    /// `b + 1` (1969 notation only)
    Ar(ArOp, Box<Expr>, Box<Expr>, Span),
    /// `item(q)` where a value is wanted: the result of a function (1969 notation only)
    App(Box<Expr>, Vec<Expr>, Span),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArOp {
    Add,
    Sub,
    Mul,
    Div,
}

impl ArOp {
    pub fn sym(self) -> &'static str {
        match self {
            ArOp::Add => "+",
            ArOp::Sub => "-",
            ArOp::Mul => "*",
            ArOp::Div => "/",
        }
    }
}

#[derive(Debug)]
pub struct Lambda {
    pub params: Vec<(String, Span)>,
    /// `=> z` (1969): the result, a last bound variable that an application in a term supplies
    pub result: Option<(String, Span)>,
    /// `key p` (1969): the bound variables that must be instantiated before an application is elaborated
    pub key: Vec<(String, Span)>,
    /// In the 1969 notation, the names introduced with a prime (`r'`); in the 1990 notation,
    /// None: every variable written with a capital that is not bound is local.
    pub primed: Option<Vec<String>>,
    pub body: Assertion,
    pub span: Span,
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Var(_, s)
            | Expr::Name(_, s)
            | Expr::Num(_, s)
            | Expr::Atom(_, s)
            | Expr::Str(_, s)
            | Expr::List(_, _, s)
            | Expr::Ar(_, _, _, s)
            | Expr::App(_, _, s) => *s,
            Expr::Lambda(l) => l.span,
        }
    }
}

/// Something asserted to hold.
#[derive(Clone, Debug)]
pub enum Assertion {
    Eq(Expr, Expr, Span),
    /// `app(L, S, N)`: an application of a lambda expression.
    Call(Expr, Vec<Expr>, Span),
    And(Vec<Assertion>, Span),
    Or(Vec<Assertion>, Span),
    Not(Box<Assertion>, Span),
    /// `{set(X), {…}, S}`: S is the set of the X for which the middle holds.
    Set(Expr, Box<Assertion>, Expr, Span),
    /// `tout("U", U)`: output.
    Tout(Expr, Expr, Span),
}

impl Assertion {
    pub fn span(&self) -> Span {
        match self {
            Assertion::Eq(_, _, s)
            | Assertion::Call(_, _, s)
            | Assertion::And(_, s)
            | Assertion::Or(_, s)
            | Assertion::Not(_, s)
            | Assertion::Set(_, _, _, s)
            | Assertion::Tout(_, _, s) => *s,
        }
    }
}

/// One assertion of the text, with its place.
#[derive(Clone, Debug)]
pub struct Item {
    pub assertion: Assertion,
    pub span: Span,
    pub line: usize,
}

struct Parser<'a> {
    toks: &'a [Token],
    i: usize,
    end: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.i).map(|t| &t.tok)
    }
    fn peek_at(&self, n: usize) -> Option<&Tok> {
        self.toks.get(self.i + n).map(|t| &t.tok)
    }
    fn here(&self) -> Span {
        self.toks.get(self.i).map(|t| t.span).unwrap_or(Span {
            start: self.end,
            end: self.end,
        })
    }
    fn last(&self) -> Span {
        self.toks[self.i.saturating_sub(1)].span
    }
    fn is_sym(&self, s: &str) -> bool {
        matches!(self.peek(), Some(Tok::Sym(x)) if *x == s)
    }
    fn is_kw(&self, k: &str) -> bool {
        matches!(self.peek(), Some(Tok::Kw(x)) if *x == k)
    }
    fn eat_sym(&mut self, s: &str) -> bool {
        let yes = self.is_sym(s);
        self.i += yes as usize;
        yes
    }
    fn eat_kw(&mut self, k: &str) -> bool {
        let yes = self.is_kw(k);
        self.i += yes as usize;
        yes
    }
    fn expect_sym(&mut self, s: &str, what: &str) -> Result<(), Error> {
        if self.eat_sym(s) {
            Ok(())
        } else {
            err(self.here(), format!("expected `{s}` {what}, found {}", self.describe()))
        }
    }
    fn describe(&self) -> String {
        match self.peek() {
            None => "the end of the text".into(),
            Some(Tok::Var(v) | Tok::Name(v)) => format!("`{v}`"),
            Some(Tok::Num(n)) => format!("`{n}`"),
            Some(Tok::Atom(s)) => format!("'{s}'"),
            Some(Tok::Str(s)) => format!("\"{s}\""),
            Some(Tok::Kw(k) | Tok::Sym(k)) => format!("`{k}`"),
        }
    }
    /// `(` on the same line as the token before it: an application.
    fn call_paren(&self) -> bool {
        self.is_sym("(") && self.i > 0 && self.toks[self.i].line == self.toks[self.i - 1].line
    }

    fn disjunction(&mut self) -> Result<Assertion, Error> {
        let first = self.conjunction()?;
        if !self.is_kw("or") {
            return Ok(first);
        }
        let mut alts = vec![first];
        while self.eat_kw("or") {
            alts.push(self.conjunction()?);
        }
        let span = alts[0].span().to(alts.last().unwrap().span());
        Ok(Assertion::Or(alts, span))
    }

    fn conjunction(&mut self) -> Result<Assertion, Error> {
        let first = self.unit()?;
        if !self.is_sym(",") {
            return Ok(first);
        }
        let mut parts = vec![first];
        while self.eat_sym(",") {
            parts.push(self.unit()?);
        }
        let span = parts[0].span().to(parts.last().unwrap().span());
        Ok(Assertion::And(parts, span))
    }

    fn unit(&mut self) -> Result<Assertion, Error> {
        let start = self.here();
        if self.eat_kw("not") {
            // not(takes(X, C)) as on p. 15, or not {…}, or not A
            let a = if self.eat_sym("(") {
                let a = self.disjunction()?;
                self.expect_sym(")", "to close `not(`")?;
                a
            } else {
                self.unit()?
            };
            return Ok(Assertion::Not(Box::new(a), start.to(self.last())));
        }
        if self.eat_sym("{") {
            if matches!(self.peek(), Some(Tok::Name(n)) if n == "set") && self.peek_at(1) == Some(&Tok::Sym("(")) {
                return self.set_form(start);
            }
            if self.eat_sym("}") {
                return Ok(Assertion::And(vec![], start.to(self.last())));
            }
            let inner = self.disjunction()?;
            self.expect_sym("}", "to close the `{`")?;
            let span = start.to(self.last());
            return Ok(match inner {
                Assertion::And(v, _) => Assertion::And(v, span),
                Assertion::Or(v, _) => Assertion::Or(v, span),
                a => Assertion::And(vec![a], span),
            });
        }
        let left = self.expr()?;
        if self.eat_sym("=") {
            if self.call_paren() {
                return err(self.here(), NOT_A_FUNCTION);
            }
            let right = self.expr()?;
            if self.call_paren() {
                return err(self.here(), NOT_A_FUNCTION);
            }
            let span = left.span().to(right.span());
            return Ok(Assertion::Eq(left, right, span));
        }
        if self.call_paren() {
            self.i += 1;
            let mut args = vec![];
            if !self.eat_sym(")") {
                loop {
                    args.push(self.expr()?);
                    if self.eat_sym(")") {
                        break;
                    }
                    self.expect_sym(",", "between arguments, or `)`")?;
                }
            }
            let span = left.span().to(self.last());
            if let Expr::Name(n, _) = &left {
                if n == "tout" {
                    if args.len() != 2 {
                        return err(span, "tout takes a label and a term: tout(\"U\", U)");
                    }
                    let term = args.pop().unwrap();
                    let label = args.pop().unwrap();
                    return Ok(Assertion::Tout(label, term, span));
                }
                if n == "set" {
                    return err(span, "set is written {set(X), {…}, S}, as on p. 15");
                }
            }
            return Ok(Assertion::Call(left, args, span));
        }
        err(
            left.span().to(self.here()),
            format!(
                "expected an assertion: an equation `=`, or an application `p(…)`; found {} after the term",
                self.describe()
            ),
        )
    }

    /// `{set(X), {mem(X, S1), not(mem(X, S2))}, S}` (p. 15), from just after the `{`.
    fn set_form(&mut self, start: Span) -> Result<Assertion, Error> {
        self.i += 2; // set (
        let of = self.expr()?;
        self.expect_sym(")", "after the term of set(…)")?;
        self.expect_sym(",", "after set(X): {set(X), {…}, S}")?;
        let goal = self.unit()?;
        self.expect_sym(",", "after the condition of a set: {set(X), {…}, S}")?;
        let into = self.expr()?;
        self.expect_sym("}", "to close {set(X), {…}, S}")?;
        Ok(Assertion::Set(of, Box::new(goal), into, start.to(self.last())))
    }

    fn expr(&mut self) -> Result<Expr, Error> {
        if self.is_kw("lambda") {
            return self.lambda();
        }
        let span = self.here();
        let e = match self.peek().cloned() {
            Some(Tok::Var(v)) => Expr::Var(v, span),
            Some(Tok::Name(v)) => Expr::Name(v, span),
            Some(Tok::Num(n)) => Expr::Num(n, span),
            Some(Tok::Atom(s)) => Expr::Atom(s, span),
            Some(Tok::Str(s)) => Expr::Str(s, span),
            Some(Tok::Sym("[")) => {
                self.i += 1;
                let mut items = vec![];
                let mut tail = None;
                if !self.eat_sym("]") {
                    loop {
                        items.push(self.expr()?);
                        if self.eat_sym("|") {
                            tail = Some(Box::new(self.expr()?));
                            self.expect_sym("]", "after the tail of the list")?;
                            break;
                        }
                        if self.eat_sym("]") {
                            break;
                        }
                        self.expect_sym(",", "between the members of a list, or `]`")?;
                    }
                }
                return Ok(Expr::List(items, tail, span.to(self.last())));
            }
            Some(Tok::Sym("(")) => {
                self.i += 1;
                let e = self.expr()?;
                self.expect_sym(")", "to close the `(`")?;
                return Ok(e);
            }
            _ => return err(span, format!("expected a term, found {}", self.describe())),
        };
        self.i += 1;
        Ok(e)
    }

    fn lambda(&mut self) -> Result<Expr, Error> {
        let start = self.here();
        self.i += 1;
        let mut params = vec![];
        loop {
            match self.peek() {
                Some(Tok::Var(v) | Tok::Name(v)) => {
                    params.push((v.clone(), self.here()));
                    self.i += 1;
                }
                _ => {
                    return err(
                        self.here(),
                        format!("expected a bound variable of the lambda, found {}", self.describe()),
                    )
                }
            }
            // a variable not followed by a comma is the last
            if !self.eat_sym(",") {
                break;
            }
        }
        let body = self.disjunction()?;
        let span = start.to(body.span());
        Ok(Expr::Lambda(Rc::new(Lambda {
            params,
            result: None,
            key: vec![],
            primed: None,
            body,
            span,
        })))
    }
}

const NOT_A_FUNCTION: &str =
    "an application is an assertion, not a value: what a procedure relates is among its arguments (`rev(X, Y)`, not `Y = rev(X)`)";

/// Reads a whole text: every assertion, or the first error.
pub fn parse(text: &str) -> Result<Vec<Item>, Error> {
    let toks = lex(text)?;
    let mut p = Parser {
        toks: &toks,
        i: 0,
        end: text.len(),
    };
    let mut items = vec![];
    loop {
        while p.eat_sym(";") {}
        if p.i >= toks.len() {
            return Ok(items);
        }
        let line = toks[p.i].line;
        let a = p.disjunction()?;
        items.push(Item {
            span: a.span(),
            assertion: a,
            line,
        });
        if p.i < toks.len() && toks[p.i].line == toks[p.i - 1].line && !p.is_sym(";") {
            return err(
                p.here(),
                format!(
                    "{} cannot continue the assertion: start the next on a new line, or after `;`",
                    p.describe()
                ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_as_the_retrospective_prints_it() {
        let src = "app = lambda R, S, T\n    {R = [], S = T} or\n    {R = [X|L], T = [X|N], app(L, S, N)}\napp([1], U, [1,2])\n";
        let items = parse(src).unwrap();
        assert_eq!(items.len(), 2);
        match &items[0].assertion {
            Assertion::Eq(Expr::Name(n, _), Expr::Lambda(l), _) => {
                assert_eq!(n, "app");
                assert_eq!(l.params.len(), 3);
                assert!(matches!(l.body, Assertion::Or(ref v, _) if v.len() == 2));
            }
            a => panic!("{a:?}"),
        }
        assert!(matches!(items[1].assertion, Assertion::Call(..)));
    }

    #[test]
    fn the_forms_of_page_15() {
        let src = "non-maths-major = lambda X\n {maths-course(C), not(takes(X, C))}\n\
                   setdiff = lambda S1, S2, S\n {set(X), {mem(X, S1), not(mem(X, S2))}, S}\n\
                   student = lambda X\n {X = 'BROWN'} or {X = 'SMITH'}\ntout(\"U\", U)";
        let items = parse(src).unwrap();
        assert_eq!(items.len(), 4);
        let Assertion::Eq(_, Expr::Lambda(l), _) = &items[1].assertion else {
            panic!()
        };
        assert!(matches!(l.body, Assertion::Set(..)));
        assert!(matches!(items[3].assertion, Assertion::Tout(..)));
    }

    #[test]
    fn what_is_refused() {
        assert!(parse("Y = f(X)").unwrap_err().message.contains("not a value"));
        assert!(parse("X = 1 Y = 2").is_err());
        assert_eq!(parse("X = 1; Y = 2").unwrap().len(), 2);
        assert!(parse("X = Y + 1").unwrap_err().message.contains("no arithmetic"));
    }
}
