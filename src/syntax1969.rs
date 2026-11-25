//! The notation of Foster and Elcock (1969), "Absys 1: an incremental
//! compiler for assertions", Machine Intelligence 4, as far as the surviving
//! extracts of that paper show it (`sources/README.md`):
//!
//! ```text
//! x=y   y = 2                                  assertions side by side: and
//! p=[1;2] {p = [a;b] or p = [b;a]}             lists [a;b;c], alternatives {… or …}
//! a=b+1                                        arithmetic, in any direction
//! map'=lambda p, f key p
//!   begin {null(p) or p = [r'&s'] f(r) map(s,f)} end
//! item'=lambda x=>z key x
//!   begin x = [p'&q'] {z=p or z=item(q)} end
//! ```
//!
//! `[h&t]` is a list with head h and tail t. A prime introduces a new name
//! (`r'`), local to the lambda expression it is in (the paper scopes it by
//! `begin … end`; see docs/decisions.md). `=> z` names the result of a
//! function, so that `i = item(q)` is the application item(q, i). `key p`
//! says p must be present before an application is elaborated. `null(p)` is
//! `p = []`. Nothing separates assertions.
//!
//! The tree is the same as the 1990 notation's ([`crate::syntax`]), so both
//! run on the same machine.

use crate::syntax::{ArOp, Assertion, Error, Expr, Item, Lambda, Span};
use std::collections::HashSet;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    /// an identifier, and whether it was primed
    Id(String, bool),
    Num(i128),
    Quote(String),
    Str(String),
    Kw(&'static str),
    Sym(&'static str),
}

struct Token {
    tok: Tok,
    span: Span,
    line: usize,
}

const KEYWORDS: &[&str] = &["lambda", "begin", "end", "or", "key"];
const SYMBOLS: &[&str] = &[
    "=>", "=", "(", ")", "[", "]", "{", "}", ",", ";", "&", "+", "-", "*", "/",
];

fn err<T>(span: Span, m: impl Into<String>) -> Result<T, Error> {
    Err(Error {
        span,
        message: m.into(),
    })
}

fn lex(text: &str) -> Result<Vec<Token>, Error> {
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
        let tok = if c.is_ascii_alphabetic() {
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            let w = text[start..i].to_string();
            let primed = i < b.len() && b[i] == b'\'';
            let span_end = i;
            if primed {
                i += 1;
            }
            match KEYWORDS.iter().find(|k| **k == w) {
                Some(k) if !primed => Tok::Kw(k),
                _ => {
                    out.push(Token {
                        tok: Tok::Id(w, primed),
                        span: Span { start, end: span_end },
                        line,
                    });
                    continue;
                }
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
                return err(Span { start, end: i }, "a constant with no closing quote");
            }
            i += 1;
            let s = text[start + 1..i - 1].to_string();
            if c == b'\'' {
                Tok::Quote(s)
            } else {
                Tok::Str(s)
            }
        } else if let Some(s) = SYMBOLS.iter().find(|s| text[i..].starts_with(**s)) {
            i += s.len();
            Tok::Sym(s)
        } else {
            let ch = text[i..].chars().next().unwrap();
            return err(
                Span {
                    start,
                    end: i + ch.len_utf8(),
                },
                format!("`{ch}` is not in the 1969 notation"),
            );
        };
        out.push(Token {
            tok,
            span: Span { start, end: i },
            line,
        });
    }
    Ok(out)
}

struct P<'a> {
    toks: &'a [Token],
    i: usize,
    end: usize,
    /// the names primed in each lambda being read
    primes: Vec<HashSet<String>>,
}

impl P<'_> {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.i).map(|t| &t.tok)
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
        let y = self.is_sym(s);
        self.i += y as usize;
        y
    }
    fn eat_kw(&mut self, k: &str) -> bool {
        let y = self.is_kw(k);
        self.i += y as usize;
        y
    }
    fn describe(&self) -> String {
        match self.peek() {
            None => "the end of the text".into(),
            Some(Tok::Id(v, p)) => format!("`{v}{}`", if *p { "'" } else { "" }),
            Some(Tok::Num(n)) => format!("`{n}`"),
            Some(Tok::Quote(s)) => format!("'{s}'"),
            Some(Tok::Str(s)) => format!("\"{s}\""),
            Some(Tok::Kw(k) | Tok::Sym(k)) => format!("`{k}`"),
        }
    }
    fn expect_sym(&mut self, s: &str, what: &str) -> Result<(), Error> {
        if self.eat_sym(s) {
            Ok(())
        } else {
            err(self.here(), format!("expected `{s}` {what}, found {}", self.describe()))
        }
    }
    /// Whether the next token can begin an assertion.
    fn starts_assertion(&self) -> bool {
        matches!(
            self.peek(),
            Some(
                Tok::Id(..) | Tok::Num(_) | Tok::Quote(_) | Tok::Str(_) | Tok::Sym("[" | "{" | "(") | Tok::Kw("begin")
            )
        )
    }

    /// Assertions side by side, until `or`, `}`, `end` or the end.
    fn conjunction(&mut self) -> Result<Assertion, Error> {
        let start = self.here();
        let mut parts = vec![];
        while self.starts_assertion() {
            parts.push(self.assertion()?);
        }
        if parts.len() == 1 {
            return Ok(parts.pop().unwrap());
        }
        let span = parts
            .first()
            .map(|p| p.span().to(parts.last().unwrap().span()))
            .unwrap_or(start);
        Ok(Assertion::And(parts, span))
    }

    fn assertion(&mut self) -> Result<Assertion, Error> {
        let start = self.here();
        if self.eat_kw("begin") {
            let c = self.conjunction()?;
            if !self.eat_kw("end") {
                return err(
                    self.here(),
                    format!("expected `end` to close `begin`, found {}", self.describe()),
                );
            }
            let span = start.to(self.last());
            return Ok(match c {
                Assertion::And(v, _) => Assertion::And(v, span),
                a => Assertion::And(vec![a], span),
            });
        }
        if self.eat_sym("{") {
            let mut alts = vec![self.conjunction()?];
            while self.eat_kw("or") {
                alts.push(self.conjunction()?);
            }
            self.expect_sym("}", "to close the `{`")?;
            let span = start.to(self.last());
            return Ok(if alts.len() == 1 {
                alts.pop().unwrap()
            } else {
                Assertion::Or(alts, span)
            });
        }
        let left = self.term()?;
        if self.eat_sym("=") {
            let right = self.term()?;
            let span = left.span().to(right.span());
            return Ok(Assertion::Eq(left, right, span));
        }
        // an application standing alone is an assertion
        if let Expr::App(f, mut args, span) = left {
            if let Expr::Name(n, _) = &*f {
                match (n.as_str(), args.len()) {
                    ("null", 1) => return Ok(Assertion::Eq(args.pop().unwrap(), Expr::List(vec![], None, span), span)),
                    ("tout", 2) => {
                        let t = args.pop().unwrap();
                        let l = args.pop().unwrap();
                        return Ok(Assertion::Tout(l, t, span));
                    }
                    ("not" | "set", _) => {
                        return err(
                            span,
                            format!("`{n}` is not in the 1969 description; it is in the 1990 notation (.abs)"),
                        )
                    }
                    _ => {}
                }
            }
            return Ok(Assertion::Call(*f, args, span));
        }
        err(
            left.span().to(self.here()),
            format!(
                "expected `=` or an application, found {} after the term",
                self.describe()
            ),
        )
    }

    fn term(&mut self) -> Result<Expr, Error> {
        if self.is_kw("lambda") {
            return self.lambda();
        }
        let mut e = self.product()?;
        loop {
            let op = if self.eat_sym("+") {
                ArOp::Add
            } else if self.eat_sym("-") {
                ArOp::Sub
            } else {
                return Ok(e);
            };
            let r = self.product()?;
            let span = e.span().to(r.span());
            e = Expr::Ar(op, Box::new(e), Box::new(r), span);
        }
    }

    fn product(&mut self) -> Result<Expr, Error> {
        let mut e = self.primary()?;
        loop {
            let op = if self.eat_sym("*") {
                ArOp::Mul
            } else if self.eat_sym("/") {
                ArOp::Div
            } else {
                return Ok(e);
            };
            let r = self.primary()?;
            let span = e.span().to(r.span());
            e = Expr::Ar(op, Box::new(e), Box::new(r), span);
        }
    }

    fn primary(&mut self) -> Result<Expr, Error> {
        let span = self.here();
        let e = match self.peek().cloned() {
            Some(Tok::Id(n, primed)) => {
                self.i += 1;
                if primed {
                    if let Some(set) = self.primes.last_mut() {
                        set.insert(n.clone());
                    }
                }
                let id = Expr::Name(n, span);
                if self.eat_sym("(") {
                    let mut args = vec![];
                    if !self.eat_sym(")") {
                        loop {
                            args.push(self.term()?);
                            if self.eat_sym(")") {
                                break;
                            }
                            self.expect_sym(",", "between arguments, or `)`")?;
                        }
                    }
                    return Ok(Expr::App(Box::new(id), args, span.to(self.last())));
                }
                return Ok(id);
            }
            Some(Tok::Num(n)) => Expr::Num(n, span),
            Some(Tok::Quote(s)) => Expr::Atom(s, span),
            Some(Tok::Str(s)) => Expr::Str(s, span),
            Some(Tok::Sym("-")) => {
                self.i += 1;
                if let Some(Tok::Num(n)) = self.peek().cloned() {
                    self.i += 1;
                    return Ok(Expr::Num(-n, span.to(self.last())));
                }
                return err(self.here(), "expected a number after `-`");
            }
            Some(Tok::Sym("(")) => {
                self.i += 1;
                let e = self.term()?;
                self.expect_sym(")", "to close the `(`")?;
                return Ok(e);
            }
            Some(Tok::Sym("[")) => {
                self.i += 1;
                let mut items = vec![];
                let mut tail = None;
                if !self.eat_sym("]") {
                    loop {
                        items.push(self.term()?);
                        if self.eat_sym("&") {
                            tail = Some(Box::new(self.term()?));
                            self.expect_sym("]", "after the tail of the list")?;
                            break;
                        }
                        if self.eat_sym("]") {
                            break;
                        }
                        self.expect_sym(";", "between the items of a list, or `]`")?;
                    }
                }
                return Ok(Expr::List(items, tail, span.to(self.last())));
            }
            _ => return err(span, format!("expected a term, found {}", self.describe())),
        };
        self.i += 1;
        Ok(e)
    }

    fn ident(&mut self, what: &str) -> Result<(String, Span), Error> {
        match self.peek().cloned() {
            Some(Tok::Id(n, _)) => {
                let s = self.here();
                self.i += 1;
                Ok((n, s))
            }
            _ => err(self.here(), format!("expected {what}, found {}", self.describe())),
        }
    }

    fn lambda(&mut self) -> Result<Expr, Error> {
        let start = self.here();
        self.i += 1;
        let mut params = vec![self.ident("a bound variable of the lambda")?];
        while self.eat_sym(",") {
            params.push(self.ident("a bound variable of the lambda")?);
        }
        let result = if self.eat_sym("=>") {
            Some(self.ident("the result after `=>`")?)
        } else {
            None
        };
        let mut key = vec![];
        if self.eat_kw("key") {
            key.push(self.ident("a variable after `key`")?);
            while self.eat_sym(",") {
                key.push(self.ident("a variable after `key`")?);
            }
        }
        if !(self.is_kw("begin") || self.is_sym("{")) {
            return err(
                self.here(),
                format!(
                    "expected the body of the lambda (`begin` or `{{`), found {}",
                    self.describe()
                ),
            );
        }
        self.primes.push(HashSet::new());
        let body = self.assertion()?;
        let primed = self.primes.pop().unwrap();
        let body = localise(body, &primed);
        let span = start.to(body.span());
        Ok(Expr::Lambda(Rc::new(Lambda {
            params,
            result,
            key,
            primed: Some(primed.into_iter().collect()),
            body,
            span,
        })))
    }
}

/// Marks the names introduced with a prime as local (the compiler's `Var`).
fn localise(a: Assertion, primed: &HashSet<String>) -> Assertion {
    fn e(x: Expr, p: &HashSet<String>) -> Expr {
        match x {
            Expr::Name(n, s) if p.contains(&n) => Expr::Var(n, s),
            Expr::List(items, tail, s) => Expr::List(
                items.into_iter().map(|i| e(i, p)).collect(),
                tail.map(|t| Box::new(e(*t, p))),
                s,
            ),
            Expr::Ar(op, a, b, s) => Expr::Ar(op, Box::new(e(*a, p)), Box::new(e(*b, p)), s),
            Expr::App(f, args, s) => Expr::App(Box::new(e(*f, p)), args.into_iter().map(|x| e(x, p)).collect(), s),
            x => x,
        }
    }
    match a {
        Assertion::Eq(l, r, s) => Assertion::Eq(e(l, primed), e(r, primed), s),
        Assertion::Call(f, args, s) => {
            Assertion::Call(e(f, primed), args.into_iter().map(|x| e(x, primed)).collect(), s)
        }
        Assertion::Tout(l, t, s) => Assertion::Tout(e(l, primed), e(t, primed), s),
        Assertion::And(v, s) => Assertion::And(v.into_iter().map(|x| localise(x, primed)).collect(), s),
        Assertion::Or(v, s) => Assertion::Or(v.into_iter().map(|x| localise(x, primed)).collect(), s),
        Assertion::Not(x, s) => Assertion::Not(Box::new(localise(*x, primed)), s),
        Assertion::Set(of, g, into, s) => {
            Assertion::Set(e(of, primed), Box::new(localise(*g, primed)), e(into, primed), s)
        }
    }
}

pub fn parse(text: &str) -> Result<Vec<Item>, Error> {
    let toks = lex(text)?;
    let mut p = P {
        toks: &toks,
        i: 0,
        end: text.len(),
        primes: vec![],
    };
    let mut items = vec![];
    while p.i < toks.len() {
        if !p.starts_assertion() {
            return err(p.here(), format!("{} cannot begin an assertion", p.describe()));
        }
        let line = toks[p.i].line;
        let a = p.assertion()?;
        items.push(Item {
            span: a.span(),
            assertion: a,
            line,
        });
    }
    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn side_by_side() {
        assert_eq!(parse("x=y   y = 2").unwrap().len(), 2);
        assert_eq!(parse("p=[1;2] {p = [a;b] or p = [b;a]}").unwrap().len(), 2);
    }

    #[test]
    fn map_and_item() {
        let src = "map'=lambda p, f key p\nbegin {null(p) or p = [r'&s'] f(r) map(s,f)} end\n\
                   item'=lambda x=>z key x\nbegin x = [p'&q'] {z=p or z=item(q)} end\ni=item([1;2;3])";
        let items = parse(src).unwrap();
        assert_eq!(items.len(), 3);
        let Assertion::Eq(_, Expr::Lambda(l), _) = &items[1].assertion else {
            panic!()
        };
        assert_eq!(l.result.as_ref().unwrap().0, "z");
        assert_eq!(l.key[0].0, "x");
        let mut primed = l.primed.clone().unwrap();
        primed.sort();
        assert_eq!(primed, vec!["p", "q"]);
    }
}
