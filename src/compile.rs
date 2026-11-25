//! From the tree of an assertion to what the machine elaborates.
//!
//! Each assertion of the text, and each lambda expression, becomes a [`Proc`]:
//! its variables numbered as slots, its body a tree of [`G`]. A variable that
//! is not bound by the lambda is *local* ("existentially quantified", Elcock
//! 1990 p. 8) and belongs to the innermost alternative of an `or` (or the body
//! of a `not` or a `set`) that contains every occurrence of it: it is created
//! when that alternative is entered. So the second alternative of `app` gets
//! fresh X, L and N each time it is chosen, and the first gets none, which is
//! how the retrospective numbers them (X₁, L₁, N₁; and in the reverse example
//! X₄ and L₄ for the variables of the next assertion, p. 13).
//!
//! Terms with arithmetic or a function's result in them (the 1969 notation)
//! are flattened here: `a = b + 1` is the primitive relation plus(b, 1, a),
//! and `i = item(q)` is the application item(q, i).

use crate::syntax::{ArOp, Assertion, Error, Expr, Lambda, Span};
use std::cell::OnceCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub enum T {
    Slot(u32),
    Global(Rc<str>),
    Num(i128),
    Atom(Rc<str>),
    Str(Rc<str>),
    Nil,
    Cons(Box<T>, Box<T>),
    Lam(Rc<Proc>),
}

#[derive(Debug)]
pub enum G {
    Eq(T, T),
    /// a op b = c
    Arith(ArOp, T, T, T),
    Call(T, Vec<T>),
    And(Vec<Rc<G>>),
    Or(Vec<Rc<Scope>>),
    Not(Rc<Scope>),
    /// set of `of` (read inside the scope) such that the scope holds, equal to `into`
    Set(T, Rc<Scope>, T),
    Tout(T, T),
}

/// A part of a body whose local variables are created when it is entered.
#[derive(Debug)]
pub struct Scope {
    pub intro: Vec<u32>,
    pub goal: Rc<G>,
    /// the names of the procedure's slots, for naming the variables it creates
    pub names: OnceCell<Rc<[Rc<str>]>>,
}

/// Gives every scope of a body the names of the procedure's slots.
fn name_scopes(s: &Scope, names: &Rc<[Rc<str>]>) {
    let _ = s.names.set(names.clone());
    fn walk(g: &G, names: &Rc<[Rc<str>]>) {
        match g {
            G::And(v) => v.iter().for_each(|c| walk(c, names)),
            G::Or(alts) => alts.iter().for_each(|a| name_scopes(a, names)),
            G::Not(s) | G::Set(_, s, _) => name_scopes(s, names),
            _ => {}
        }
    }
    walk(&s.goal, names);
}

/// A lambda expression, or one assertion of the text.
#[derive(Debug)]
pub struct Proc {
    pub params: Vec<u32>,
    /// the bound variables that must be instantiated before the body is elaborated (`key`)
    pub key: Vec<u32>,
    /// whether the last bound variable is a result (`=> z`)
    pub result: bool,
    pub slot_names: Vec<Rc<str>>,
    pub body: Rc<Scope>,
    /// the source, for display
    pub text: Rc<str>,
    pub span: Span,
}

impl Proc {
    pub fn arity(&self) -> usize {
        self.params.len()
    }
}

struct Cx<'a> {
    text: &'a str,
    top: bool,
    names: Vec<Rc<str>>,
    params: HashMap<String, u32>,
    locals: HashMap<String, u32>,
    /// for each local slot, the scope it is created in
    owner: HashMap<u32, usize>,
    /// pass 1: each occurrence's path of scopes
    paths: HashMap<u32, Vec<usize>>,
    path: Vec<usize>,
    next_scope: usize,
    /// pass 2: slots made for arithmetic and results, per open scope
    temps: Vec<Vec<u32>>,
    /// goals a term needs (plus, an application for a result), emitted before the assertion
    pending: Vec<Rc<G>>,
}

fn err<X>(span: Span, m: impl Into<String>) -> Result<X, Error> {
    Err(Error {
        span,
        message: m.into(),
    })
}

impl Cx<'_> {
    fn new_slot(&mut self, name: &str) -> u32 {
        self.names.push(Rc::from(name));
        (self.names.len() - 1) as u32
    }

    /// Which slot a variable is, if it is not global.
    fn slot_of(&mut self, e: &Expr) -> Option<u32> {
        let (name, capital) = match e {
            Expr::Var(n, _) => (n, true),
            Expr::Name(n, _) => (n, false),
            _ => return None,
        };
        if let Some(s) = self.params.get(name) {
            return Some(*s);
        }
        if self.top || !capital {
            return None;
        }
        if let Some(s) = self.locals.get(name) {
            return Some(*s);
        }
        let s = self.new_slot(name);
        self.locals.insert(name.clone(), s);
        Some(s)
    }

    // ---- pass 1: where each local variable belongs ----

    fn enter_scope1(&mut self) {
        self.path.push(self.next_scope);
        self.next_scope += 1;
    }

    fn walk_expr(&mut self, e: &Expr) {
        match e {
            Expr::Var(..) | Expr::Name(..) => {
                if let Some(s) = self.slot_of(e) {
                    if !self.params.values().any(|p| *p == s) {
                        let path = self.path.clone();
                        self.paths
                            .entry(s)
                            .and_modify(|p| {
                                let n = p.iter().zip(&path).take_while(|(a, b)| a == b).count();
                                p.truncate(n);
                            })
                            .or_insert(path);
                    }
                }
            }
            Expr::List(items, tail, _) => {
                items.iter().for_each(|i| self.walk_expr(i));
                if let Some(t) = tail {
                    self.walk_expr(t);
                }
            }
            Expr::Ar(_, a, b, _) => {
                self.walk_expr(a);
                self.walk_expr(b);
            }
            Expr::App(f, args, _) => {
                self.walk_expr(f);
                args.iter().for_each(|a| self.walk_expr(a));
            }
            Expr::Num(..) | Expr::Atom(..) | Expr::Str(..) | Expr::Lambda(_) => {}
        }
    }

    fn walk(&mut self, a: &Assertion) {
        match a {
            Assertion::Eq(l, r, _) => {
                self.walk_expr(l);
                self.walk_expr(r);
            }
            Assertion::Call(f, args, _) => {
                self.walk_expr(f);
                args.iter().for_each(|x| self.walk_expr(x));
            }
            Assertion::Tout(l, t, _) => {
                self.walk_expr(l);
                self.walk_expr(t);
            }
            Assertion::And(v, _) => v.iter().for_each(|x| self.walk(x)),
            Assertion::Or(v, _) => {
                for alt in v {
                    self.enter_scope1();
                    self.walk(alt);
                    self.path.pop();
                }
            }
            Assertion::Not(x, _) => {
                self.enter_scope1();
                self.walk(x);
                self.path.pop();
            }
            Assertion::Set(of, g, into, _) => {
                self.walk_expr(into);
                self.enter_scope1();
                self.walk_expr(of);
                self.walk(g);
                self.path.pop();
            }
        }
    }

    // ---- pass 2: the tree ----

    fn open_scope(&mut self) -> usize {
        let id = self.next_scope;
        self.next_scope += 1;
        self.temps.push(vec![]);
        id
    }

    fn close_scope(&mut self, id: usize, goal: Rc<G>) -> Rc<Scope> {
        let mut intro: Vec<u32> = self.owner.iter().filter(|(_, o)| **o == id).map(|(s, _)| *s).collect();
        intro.sort();
        intro.extend(self.temps.pop().unwrap());
        Rc::new(Scope {
            intro,
            goal,
            names: OnceCell::new(),
        })
    }

    fn temp(&mut self) -> u32 {
        let n = self.names.len();
        let s = self.new_slot(&format!("_{n}"));
        self.temps.last_mut().unwrap().push(s);
        s
    }

    fn term(&mut self, e: &Expr) -> Result<T, Error> {
        Ok(match e {
            Expr::Var(n, _) | Expr::Name(n, _) => match self.slot_of(e) {
                Some(s) => T::Slot(s),
                None => T::Global(Rc::from(n.as_str())),
            },
            Expr::Num(n, _) => T::Num(*n),
            Expr::Atom(s, _) => T::Atom(Rc::from(s.as_str())),
            Expr::Str(s, _) => T::Str(Rc::from(s.as_str())),
            Expr::List(items, tail, _) => {
                let mut t = match tail {
                    Some(t) => self.term(t)?,
                    None => T::Nil,
                };
                for i in items.iter().rev() {
                    t = T::Cons(Box::new(self.term(i)?), Box::new(t));
                }
                t
            }
            Expr::Lambda(l) => {
                if !self.top {
                    return err(
                        l.span,
                        "a lambda expression inside a lambda expression: the higher-order features of Absys-4 are not reconstructed",
                    );
                }
                T::Lam(proc_of(self.text, l)?)
            }
            Expr::Ar(..) | Expr::App(..) => {
                let t = self.temp();
                self.result_into(e, T::Slot(t))?;
                T::Slot(t)
            }
        })
    }

    /// Emits the goal that makes `e` (arithmetic or an application) equal to `into`.
    fn result_into(&mut self, e: &Expr, into: T) -> Result<(), Error> {
        let g = match e {
            Expr::Ar(op, a, b, _) => {
                let (a, b) = (self.term(a)?, self.term(b)?);
                match op {
                    ArOp::Add | ArOp::Mul => G::Arith(*op, a, b, into),
                    // a - b = c  is  c + b = a;  a / b = c  is  c * b = a
                    ArOp::Sub => G::Arith(ArOp::Add, into, b, a),
                    ArOp::Div => G::Arith(ArOp::Mul, into, b, a),
                }
            }
            Expr::App(f, args, _) => {
                let f = self.term(f)?;
                let mut v = args.iter().map(|a| self.term(a)).collect::<Result<Vec<_>, _>>()?;
                v.push(into);
                G::Call(f, v)
            }
            _ => unreachable!(),
        };
        self.pending.push(Rc::new(g));
        Ok(())
    }

    fn with_pending(&mut self, g: G) -> Rc<G> {
        if self.pending.is_empty() {
            return Rc::new(g);
        }
        let mut v = std::mem::take(&mut self.pending);
        v.push(Rc::new(g));
        Rc::new(G::And(v))
    }

    fn goal(&mut self, a: &Assertion) -> Result<Rc<G>, Error> {
        let computed = |e: &Expr| matches!(e, Expr::Ar(..) | Expr::App(..));
        Ok(match a {
            Assertion::Eq(l, r, _) if computed(r) && !computed(l) => {
                let into = self.term(l)?;
                self.result_into(r, into)?;
                let v = std::mem::take(&mut self.pending);
                if v.len() == 1 {
                    v.into_iter().next().unwrap()
                } else {
                    Rc::new(G::And(v))
                }
            }
            Assertion::Eq(l, r, _) if computed(l) && !computed(r) => {
                let swapped = Assertion::Eq(r.clone(), l.clone(), a.span());
                return self.goal(&swapped);
            }
            Assertion::Eq(l, r, _) => {
                let (l, r) = (self.term(l)?, self.term(r)?);
                self.with_pending(G::Eq(l, r))
            }
            Assertion::Call(f, args, _) => {
                let f = self.term(f)?;
                let args = args.iter().map(|x| self.term(x)).collect::<Result<Vec<_>, _>>()?;
                self.with_pending(G::Call(f, args))
            }
            Assertion::Tout(l, t, _) => {
                let (l, t) = (self.term(l)?, self.term(t)?);
                self.with_pending(G::Tout(l, t))
            }
            Assertion::And(v, _) => {
                let mut out = vec![];
                for x in v {
                    out.push(self.goal(x)?);
                }
                Rc::new(G::And(out))
            }
            Assertion::Or(v, _) => {
                let mut alts = vec![];
                for x in v {
                    let id = self.open_scope();
                    let g = self.goal(x)?;
                    alts.push(self.close_scope(id, g));
                }
                Rc::new(G::Or(alts))
            }
            Assertion::Not(x, _) => {
                let id = self.open_scope();
                let g = self.goal(x)?;
                Rc::new(G::Not(self.close_scope(id, g)))
            }
            Assertion::Set(of, g, into, _) => {
                let into = self.term(into)?;
                let outer = std::mem::take(&mut self.pending);
                let id = self.open_scope();
                let of = self.term(of)?;
                let g = self.goal(g)?;
                let g = self.with_pending_rc(g);
                let scope = self.close_scope(id, g);
                self.pending = outer;
                self.with_pending(G::Set(of, scope, into))
            }
        })
    }

    fn with_pending_rc(&mut self, g: Rc<G>) -> Rc<G> {
        if self.pending.is_empty() {
            return g;
        }
        let mut v = std::mem::take(&mut self.pending);
        v.push(g);
        Rc::new(G::And(v))
    }
}

fn build(text: &str, top: bool, params: &[(String, Span)], body: &Assertion, span: Span) -> Result<Proc, Error> {
    let mut cx = Cx {
        text,
        top,
        names: vec![],
        params: HashMap::new(),
        locals: HashMap::new(),
        owner: HashMap::new(),
        paths: HashMap::new(),
        path: vec![0],
        next_scope: 1,
        temps: vec![],
        pending: vec![],
    };
    let mut ps = vec![];
    for (p, sp) in params {
        if cx.params.contains_key(p) {
            return err(*sp, format!("`{p}` is bound twice by the same lambda"));
        }
        let s = cx.new_slot(p);
        cx.params.insert(p.clone(), s);
        ps.push(s);
    }
    cx.walk(body);
    cx.owner = cx.paths.iter().map(|(s, p)| (*s, *p.last().unwrap())).collect();
    cx.next_scope = 0;
    let id = cx.open_scope();
    let g = cx.goal(body)?;
    let g = cx.with_pending_rc(g);
    let root = cx.close_scope(id, g);
    let names: Rc<[Rc<str>]> = cx.names.clone().into();
    name_scopes(&root, &names);
    Ok(Proc {
        params: ps,
        key: vec![],
        result: false,
        slot_names: cx.names,
        body: root,
        text: Rc::from(&text[span.start.min(text.len())..span.end.min(text.len())]),
        span,
    })
}

pub fn proc_of(text: &str, l: &Lambda) -> Result<Rc<Proc>, Error> {
    let mut params = l.params.clone();
    if let Some(r) = &l.result {
        params.push(r.clone());
    }
    let mut p = build(text, false, &params, &l.body, l.span)?;
    p.result = l.result.is_some();
    for (k, sp) in &l.key {
        match params.iter().position(|(n, _)| n == k) {
            Some(i) => p.key.push(p.params[i]),
            None => return err(*sp, format!("key `{k}` is not one of the lambda's bound variables")),
        }
    }
    Ok(Rc::new(p))
}

/// One assertion of the text, ready to be added to a computation.
pub fn assertion(text: &str, a: &Assertion) -> Result<Rc<Proc>, Error> {
    Ok(Rc::new(build(text, true, &[], a, a.span())?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::parse;

    #[test]
    fn locals_belong_to_the_second_alternative() {
        let src = "app = lambda R, S, T\n {R = [], S = T} or\n {R = [X|L], T = [X|N], app(L, S, N)}";
        let items = parse(src).unwrap();
        let Assertion::Eq(_, Expr::Lambda(l), _) = &items[0].assertion else {
            panic!()
        };
        let p = proc_of(src, l).unwrap();
        assert_eq!(p.params, vec![0, 1, 2]);
        assert!(p.body.intro.is_empty());
        let G::Or(alts) = &*p.body.goal else { panic!() };
        assert!(alts[0].intro.is_empty());
        let names: Vec<&str> = alts[1].intro.iter().map(|s| &*p.slot_names[*s as usize]).collect();
        assert_eq!(names, vec!["X", "L", "N"]);
    }

    #[test]
    fn a_local_used_across_alternatives_belongs_to_the_body() {
        let src = "f = lambda A\n {A = [C]} or {A = [C, C]}";
        let items = parse(src).unwrap();
        let Assertion::Eq(_, Expr::Lambda(l), _) = &items[0].assertion else {
            panic!()
        };
        let p = proc_of(src, l).unwrap();
        assert_eq!(p.body.intro.len(), 1);
    }
}
