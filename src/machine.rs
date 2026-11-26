//! The machine: derivations, and the processes they elaborate.
//!
//! What it does, and the page of the sources each rule rests on, is listed by
//! `absys decisions` (and in `docs/decisions.md`). In short:
//!
//! * A **computation** is a set of **derivations**, each with its own binding
//!   environment and its own goal: "parallel non-interacting programs"
//!   (Foster and Elcock 1969). An `or` distributes through the goal of the
//!   derivation it is in, making one new derivation per alternative (Elcock
//!   1990 p. 9); a derivation that is found unsatisfiable disappears.
//! * **Equality is a process** (Elcock 1990 pp. 10-11). Lists: equate heads
//!   and tails. The same constant or the same variable: success. A list and
//!   a constant, or different constants: failure. A variable and a term: the
//!   variable is instantiated, and the processes waiting on it are added to
//!   the goal again. **Two different uninstantiated variables: the equation is
//!   suspended**, and waits on both. No occur check.
//! * Other processes wait in the same way: an application of an identifier
//!   with no value yet, an application whose `key` variables are not yet
//!   instantiated (1969), arithmetic until two of its three numbers are known
//!   (Foster 1968's "pass sets" for plus), and `tout` until its term is
//!   complete.
//! * A derivation with nothing left to do **hibernates** (Elcock 1990 p. 11,
//!   note 13). A new assertion is added to every derivation, hibernating or
//!   not, and wakes it.
//! * **Fair search**: every derivation is elaborated a step at a time, in
//!   turn, "until found unsatisfiable" (p. 10). `--search depth-first` instead
//!   elaborates the first derivation until it fails or hibernates.
//! * `not` starts an **independent computation** from the derivation's
//!   bindings as they stand, whose criteria are reversed (p. 14); `set` starts
//!   one and collects (p. 15).

use crate::compile::{Proc, Scope, G, T};
use crate::syntax::ArOp;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum V {
    Var(u32),
    Num(i128),
    Atom(Rc<str>),
    Str(Rc<str>),
    Nil,
    Cons(Rc<(V, V)>),
    Lam(Rc<Proc>),
}

pub type Env = Rc<Vec<Option<V>>>;

#[derive(Clone, Debug)]
pub enum Goal {
    Eq(V, V),
    Arith(ArOp, V, V, V),
    Call(V, Vec<V>),
    Tout(V, V),
    /// a part of a body, with the values of its variables
    Do(Rc<G>, Env),
    /// a suspended process returned to the goal because a variable it was
    /// associated with has been instantiated
    Woken(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Why {
    /// an equation between two uninstantiated variables
    Equation,
    /// an application of an identifier with no value yet
    Undefined,
    /// an application whose key is not yet instantiated
    Key,
    Arith,
    Tout,
    /// `not` held back until its variables are instantiated (--negation delayed only)
    Negation,
    /// a `not` or `set` whose computation waited on something outside it: tried again when that is known
    Undecided,
}

#[derive(Clone, Debug)]
pub struct Susp {
    pub goal: Goal,
    pub on: Vec<u32>,
    pub done: bool,
    pub why: Why,
}

#[derive(Clone, Debug)]
pub struct Cell {
    pub name: Rc<str>,
    pub val: Option<V>,
    waiting: Vec<usize>,
}

#[derive(Clone, Debug)]
pub enum SubKind {
    Not,
    Set { of: V, into: V },
}

/// A computation a derivation has started for `not` or `set`.
#[derive(Clone, Debug)]
pub struct Sub {
    pub kind: SubKind,
    pub comp: Comp,
    /// how many variables the derivation had when it started: the computation shares these
    base: usize,
    pub text: String,
    /// the `not` or `set` itself, to suspend again if it cannot yet be decided
    goal: Goal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Running,
    Hibernating,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Search {
    Fair,
    DepthFirst,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Select {
    /// the body of an application takes the place of the literal (Elcock 1990 p. 9); the default
    InPlace,
    /// an experiment: the body joins the end of the derivation's goal
    Fifo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wake {
    /// returned relations go to the head of the goal: Elcock 1982's printed
    /// lists come out this way (sources/notes/elcock-1982.md); the default
    Front,
    /// returned relations are appended, as the word in Elcock 1982 says
    Back,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Negation {
    /// as Absys: at once, on the bindings as they stand (p. 15, note 18: not safe)
    Immediate,
    /// an experiment: wait until every variable of the negated assertion is instantiated
    Delayed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Notation {
    /// Elcock 1990: [1,2,U], [X|L]
    Retrospective,
    /// Foster and Elcock 1969: [1;2;u], [x&l]
    Absys1,
    /// Elcock 1982: mem = lambda m,s key s << s = [p|sl] and … >>
    Elcock1982,
}

#[derive(Clone, Copy, Debug)]
pub struct Opts {
    pub search: Search,
    pub select: Select,
    pub wake: Wake,
    pub negation: Negation,
    pub notation: Notation,
}

impl Default for Opts {
    fn default() -> Opts {
        Opts {
            search: Search::Fair,
            select: Select::InPlace,
            wake: Wake::Front,
            negation: Negation::Immediate,
            notation: Notation::Retrospective,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Event {
    pub deriv: usize,
    /// 0 for the computation of the text; 1 inside a `not` or `set`, and so on
    pub depth: usize,
    pub kind: &'static str,
    pub text: String,
}

pub struct Ctx {
    pub opts: Opts,
    pub next_id: usize,
    pub events: Vec<Event>,
    pub depth: usize,
    pub steps: usize,
    /// derivations that failed, and why (the last few)
    pub failed: Vec<(usize, String)>,
    /// every distribution of an `or`: the derivation, and the derivations it became
    pub tree: Vec<(usize, Vec<usize>)>,
}

impl Ctx {
    pub fn new(opts: Opts) -> Ctx {
        Ctx {
            opts,
            next_id: 1,
            events: vec![],
            depth: 0,
            steps: 0,
            failed: vec![],
            tree: vec![],
        }
    }
    fn id(&mut self) -> usize {
        self.next_id += 1;
        self.next_id - 1
    }
    fn ev(&mut self, deriv: usize, kind: &'static str, text: String) {
        self.events.push(Event {
            deriv,
            depth: self.depth,
            kind,
            text,
        });
    }
}

pub enum Out {
    Progress,
    Split(Vec<Deriv>),
    Fail(String),
    Idle,
}

#[derive(Clone, Debug)]
pub struct Deriv {
    pub id: usize,
    pub parent: Option<usize>,
    pub cells: Vec<Cell>,
    /// the identifiers of the text, in the order they were first met
    pub globals: Vec<(Rc<str>, u32)>,
    pub susps: Vec<Susp>,
    ors: VecDeque<(Vec<Rc<Scope>>, Env)>,
    eqs: VecDeque<Goal>,
    lits: VecDeque<Goal>,
    pub subs: Vec<Sub>,
    pub outputs: Vec<String>,
    local_no: u32,
    sub_turn: bool,
    pub status: Status,
    /// the returned primitive relation being elaborated, if any, so that if
    /// it must wait again it keeps its associations rather than gaining new ones
    current: Option<usize>,
}

/// Lists longer than this are shown cut short, with `…`.
pub const MAX_ITEMS: usize = 32;

const SUBSCRIPTS: [char; 10] = ['₀', '₁', '₂', '₃', '₄', '₅', '₆', '₇', '₈', '₉'];

fn subscript(n: u32) -> String {
    n.to_string()
        .chars()
        .map(|c| SUBSCRIPTS[c.to_digit(10).unwrap() as usize])
        .collect()
}

impl Deriv {
    pub fn new(id: usize) -> Deriv {
        Deriv {
            id,
            parent: None,
            cells: vec![],
            globals: vec![],
            susps: vec![],
            ors: VecDeque::new(),
            eqs: VecDeque::new(),
            lits: VecDeque::new(),
            subs: vec![],
            outputs: vec![],
            local_no: 0,
            sub_turn: false,
            status: Status::Running,
            current: None,
        }
    }

    pub fn new_var(&mut self, name: Rc<str>) -> u32 {
        self.cells.push(Cell {
            name,
            val: None,
            waiting: vec![],
        });
        (self.cells.len() - 1) as u32
    }

    pub fn global(&mut self, name: &Rc<str>) -> u32 {
        if let Some((_, v)) = self.globals.iter().find(|(n, _)| n == name) {
            return *v;
        }
        let v = self.new_var(name.clone());
        self.globals.push((name.clone(), v));
        v
    }

    pub fn lookup(&self, name: &str) -> Option<u32> {
        self.globals.iter().find(|(n, _)| &**n == name).map(|(_, v)| *v)
    }

    pub fn deref(&self, v: &V) -> V {
        let mut v = v.clone();
        while let V::Var(x) = v {
            match &self.cells[x as usize].val {
                Some(t) => v = t.clone(),
                None => return v,
            }
        }
        v
    }

    /// The value with every instantiated variable in it replaced, as far as it goes.
    pub fn resolve(&self, v: &V) -> V {
        match self.deref(v) {
            V::Cons(p) => V::Cons(Rc::new((self.resolve(&p.0), self.resolve(&p.1)))),
            v => v,
        }
    }

    fn free_vars(&self, v: &V, out: &mut Vec<u32>) {
        match self.deref(v) {
            V::Var(x) => {
                if !out.contains(&x) {
                    out.push(x)
                }
            }
            V::Cons(p) => {
                self.free_vars(&p.0, out);
                self.free_vars(&p.1, out);
            }
            _ => {}
        }
    }

    pub fn is_running(&self) -> bool {
        self.status == Status::Running
    }

    /// Whether a derivation that has hibernated is a refutation: nothing it
    /// leaves suspended is more than an equation between variables, or output.
    pub fn is_refutation(&self) -> bool {
        self.waiting().all(|w| matches!(w.why, Why::Equation | Why::Tout))
    }

    /// Whether anything is left to elaborate.
    pub fn busy(&self) -> bool {
        !(self.ors.is_empty() && self.eqs.is_empty() && self.lits.is_empty() && self.subs.is_empty())
    }

    fn inst(&mut self, t: &T, env: &Env) -> V {
        match t {
            T::Slot(s) => env[*s as usize].clone().expect("a variable used before its scope"),
            T::Global(n) => V::Var(self.global(n)),
            T::Num(n) => V::Num(*n),
            T::Atom(s) => V::Atom(s.clone()),
            T::Str(s) => V::Str(s.clone()),
            T::Nil => V::Nil,
            T::Cons(h, t) => {
                let h = self.inst(h, env);
                let t = self.inst(t, env);
                V::Cons(Rc::new((h, t)))
            }
            T::Lam(p) => V::Lam(p.clone()),
        }
    }

    /// Creates the local variables of a scope; returns the environment inside it.
    fn open(&mut self, scope: &Scope, env: &Env) -> Env {
        if scope.intro.is_empty() {
            return env.clone();
        }
        let names = scope.names.get().expect("a scope without names").clone();
        let named = scope.intro.iter().any(|s| !names[*s as usize].starts_with('_'));
        if named {
            self.local_no += 1;
        }
        let sub = subscript(self.local_no);
        let mut e = (**env).clone();
        for s in &scope.intro {
            let n = &names[*s as usize];
            let name: Rc<str> = if n.starts_with('_') {
                Rc::from(format!("_{}", self.cells.len()))
            } else {
                Rc::from(format!("{n}{sub}"))
            };
            e[*s as usize] = Some(V::Var(self.new_var(name)));
        }
        Rc::new(e)
    }

    fn enter(&mut self, scope: &Scope, env: &Env) {
        let env = self.open(scope, env);
        self.add(Goal::Do(scope.goal.clone(), env));
    }

    /// Enters a scope that replaces the literal just elaborated: its literals
    /// go before the rest of the goal, unless `--select fifo`.
    fn enter_in_place(&mut self, scope: &Scope, env: &Env, select: Select) {
        if select == Select::Fifo {
            return self.enter(scope, env);
        }
        let rest = std::mem::take(&mut self.lits);
        self.enter(scope, env);
        self.lits.extend(rest);
    }

    /// Adds a process to the goal. Equations are elaborated as they arise,
    /// before any application; an `or` distributes at the next step.
    pub fn add(&mut self, g: Goal) {
        if let Goal::Woken(id) = g {
            match &self.susps[id].goal {
                Goal::Eq(..) | Goal::Arith(..) => self.eqs.push_back(g),
                Goal::Do(node, _) if matches!(&**node, G::Eq(..) | G::Arith(..)) => self.eqs.push_back(g),
                _ => self.lits.push_back(g),
            }
            return;
        }
        match g {
            Goal::Eq(..) | Goal::Arith(..) => self.eqs.push_back(g),
            Goal::Call(..) | Goal::Tout(..) => self.lits.push_back(g),
            Goal::Woken(_) => unreachable!("handled above"),
            Goal::Do(ref node, ref env) => match &**node {
                G::Eq(..) | G::Arith(..) => self.eqs.push_back(g),
                G::And(v) => {
                    for c in v {
                        self.add(Goal::Do(c.clone(), env.clone()));
                    }
                }
                G::Or(alts) => self.ors.push_back((alts.clone(), env.clone())),
                G::Call(..) | G::Tout(..) | G::Not(_) | G::Set(..) => self.lits.push_back(g),
            },
        }
    }

    /// Adds one assertion of the text.
    pub fn assert(&mut self, p: &Rc<Proc>) {
        let env: Env = Rc::new(vec![None; p.slot_names.len()]);
        self.enter(&p.body, &env);
        self.status = Status::Running;
    }

    fn suspend(&mut self, goal: Goal, on: Vec<u32>, why: Why) {
        // A returned relation that still cannot be elaborated: it is already
        // associated with its variables (Elcock 1982 p. 123), so it is only
        // associated with any it was not associated with before.
        if why == Why::Arith {
            if let Some(id) = self.current.take() {
                for x in &on {
                    if !self.cells[*x as usize].waiting.contains(&id) {
                        self.cells[*x as usize].waiting.push(id);
                    }
                    if !self.susps[id].on.contains(x) {
                        self.susps[id].on.push(*x);
                    }
                }
                self.susps[id].goal = goal;
                return;
            }
        }
        let id = self.susps.len();
        for x in &on {
            self.cells[*x as usize].waiting.push(id);
        }
        self.susps.push(Susp {
            goal,
            on,
            done: false,
            why,
        });
    }

    /// Whether a suspended process is a primitive relation: an equation, or
    /// arithmetic. These stay associated with each of their variables, and
    /// return to the goal each time one of them is instantiated, even after
    /// they have been elaborated (Elcock 1982 p. 123, where w*2 = 14 is on the
    /// list twice). Anything else returns once.
    fn primitive(&self, id: usize) -> bool {
        matches!(self.susps[id].why, Why::Equation | Why::Arith)
    }

    fn bind(&mut self, x: u32, t: V, woke: &mut Vec<usize>, wake: Wake) {
        let cell = &mut self.cells[x as usize];
        cell.val = Some(t);
        let mut back = vec![];
        for id in std::mem::take(&mut cell.waiting) {
            if self.susps[id].done {
                continue;
            }
            if !self.primitive(id) {
                self.susps[id].done = true;
            }
            woke.push(id);
            back.push(Goal::Woken(id));
        }
        match wake {
            Wake::Back => back.into_iter().for_each(|g| self.add(g)),
            Wake::Front => {
                let (eqs, lits) = (std::mem::take(&mut self.eqs), std::mem::take(&mut self.lits));
                back.into_iter().for_each(|g| self.add(g));
                self.eqs.extend(eqs);
                self.lits.extend(lits);
            }
        }
    }

    // ---- one step ----

    pub fn step(&mut self, cx: &mut Ctx) -> Out {
        if let Some((alts, env)) = self.ors.pop_front() {
            let mut kids = vec![];
            for scope in alts.iter() {
                let mut d = self.clone();
                d.id = cx.id();
                d.parent = Some(self.id);
                d.enter_in_place(scope, &env, cx.opts.select);
                kids.push(d);
            }
            let ids: Vec<String> = kids.iter().map(|k| format!("#{}", k.id)).collect();
            cx.ev(
                self.id,
                "or",
                format!(
                    "or distributes: {} alternatives, derivations {}",
                    alts.len(),
                    ids.join(", ")
                ),
            );
            return Out::Split(kids);
        }
        if let Some(g) = self.eqs.pop_front() {
            return self.process(g, cx);
        }
        let lits = !self.lits.is_empty();
        if !self.subs.is_empty() && (!lits || self.sub_turn) {
            self.sub_turn = false;
            return self.step_sub(cx);
        }
        if let Some(g) = self.lits.pop_front() {
            self.sub_turn = true;
            return self.process(g, cx);
        }
        Out::Idle
    }

    fn process(&mut self, g: Goal, cx: &mut Ctx) -> Out {
        match g {
            Goal::Eq(a, b) => self.equate(a, b, cx),
            Goal::Arith(op, a, b, c) => self.arith(op, a, b, c, cx),
            Goal::Call(f, args) => self.call(f, args, cx),
            Goal::Tout(l, t) => self.tout(l, t, cx),
            Goal::Woken(id) => {
                let g = self.susps[id].goal.clone();
                let arith = matches!(g, Goal::Arith(..));
                self.current = arith.then_some(id);
                let out = self.process(g, cx);
                self.current = None;
                out
            }
            Goal::Do(node, env) => match &*node {
                G::Eq(a, b) => {
                    let (a, b) = (self.inst(a, &env), self.inst(b, &env));
                    self.equate(a, b, cx)
                }
                G::Arith(op, a, b, c) => {
                    let (a, b, c) = (self.inst(a, &env), self.inst(b, &env), self.inst(c, &env));
                    self.arith(*op, a, b, c, cx)
                }
                G::Call(f, args) => {
                    let f = self.inst(f, &env);
                    let args = args.iter().map(|a| self.inst(a, &env)).collect();
                    self.call(f, args, cx)
                }
                G::Tout(l, t) => {
                    let (l, t) = (self.inst(l, &env), self.inst(t, &env));
                    self.tout(l, t, cx)
                }
                G::Not(scope) => self.not(node.clone(), scope, env, cx),
                G::Set(of, scope, into) => self.set(node.clone(), of, scope, into, &env, cx),
                G::And(_) | G::Or(_) => {
                    self.add(Goal::Do(node.clone(), env));
                    Out::Progress
                }
            },
        }
    }

    /// Equality, as Elcock 1990 pp. 10-11 describes it.
    pub fn equate(&mut self, a: V, b: V, cx: &mut Ctx) -> Out {
        let text = format!(
            "{} = {}",
            self.show(&a, cx.opts.notation),
            self.show(&b, cx.opts.notation)
        );
        let mut work = vec![(a, b)];
        let mut bound = vec![];
        let mut waits = vec![];
        let mut woke = vec![];
        while let Some((a, b)) = work.pop() {
            let (a, b) = (self.deref(&a), self.deref(&b));
            match (&a, &b) {
                (V::Var(x), V::Var(y)) if x == y => {}
                (V::Var(x), V::Var(y)) => {
                    waits.push(format!(
                        "{} = {}",
                        self.cells[*x as usize].name, self.cells[*y as usize].name
                    ));
                    self.suspend(Goal::Eq(a.clone(), b.clone()), vec![*x, *y], Why::Equation);
                }
                (V::Var(x), t) | (t, V::Var(x)) => {
                    bound.push(*x);
                    self.bind(*x, t.clone(), &mut woke, cx.opts.wake);
                }
                (V::Cons(p), V::Cons(q)) => {
                    work.push((p.1.clone(), q.1.clone()));
                    work.push((p.0.clone(), q.0.clone()));
                }
                (V::Nil, V::Nil) => {}
                (V::Num(m), V::Num(n)) if m == n => {}
                (V::Atom(s), V::Atom(t)) | (V::Str(s), V::Str(t)) if s == t => {}
                (V::Lam(p), V::Lam(q)) if Rc::ptr_eq(p, q) => {}
                _ => {
                    let why = format!(
                        "{}: {} cannot equal {}",
                        text,
                        self.show(&a, cx.opts.notation),
                        self.show(&b, cx.opts.notation)
                    );
                    return Out::Fail(why);
                }
            }
        }
        let mut parts = vec![];
        if !bound.is_empty() {
            let n = cx.opts.notation;
            parts.push(
                bound
                    .iter()
                    .map(|x| format!("{}/{}", self.cells[*x as usize].name, self.show_val(&V::Var(*x), n)))
                    .collect::<Vec<_>>()
                    .join(", "),
            );
        }
        if !waits.is_empty() {
            parts.push(format!("suspended {}", waits.join(", ")));
        }
        if !woke.is_empty() {
            let n = cx.opts.notation;
            let w: Vec<String> = woke.iter().map(|i| self.show_goal(&self.susps[*i].goal, n)).collect();
            parts.push(format!("reactivated {}", w.join(", ")));
        }
        let kind = if !waits.is_empty() && bound.is_empty() {
            "suspend"
        } else {
            "equation"
        };
        let detail = if parts.is_empty() {
            "holds".to_string()
        } else {
            parts.join("; ")
        };
        cx.ev(self.id, kind, format!("{text}: {detail}"));
        Out::Progress
    }

    fn arith(&mut self, op: ArOp, a: V, b: V, c: V, cx: &mut Ctx) -> Out {
        let n = cx.opts.notation;
        let text = format!(
            "{} {} {} = {}",
            self.show(&a, n),
            op.sym(),
            self.show(&b, n),
            self.show(&c, n)
        );
        let num = |d: &Deriv, v: &V| -> Result<Option<i128>, String> {
            match d.deref(v) {
                V::Num(k) => Ok(Some(k)),
                V::Var(_) => Ok(None),
                other => Err(format!("{} is not a number", d.show(&other, n))),
            }
        };
        let (x, y, z) = match (num(self, &a), num(self, &b), num(self, &c)) {
            (Ok(x), Ok(y), Ok(z)) => (x, y, z),
            (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => return Out::Fail(format!("{text}: {e}")),
        };
        let overflow = || Out::Fail(format!("{text}: a number too large"));
        let (target, value) = match (op, x, y, z) {
            (ArOp::Add, Some(x), Some(y), _) => match x.checked_add(y) {
                Some(v) => (c, v),
                None => return overflow(),
            },
            (ArOp::Add, Some(x), None, Some(z)) => (b, z - x),
            (ArOp::Add, None, Some(y), Some(z)) => (a, z - y),
            (ArOp::Mul, Some(x), Some(y), _) => match x.checked_mul(y) {
                Some(v) => (c, v),
                None => return overflow(),
            },
            (ArOp::Mul, Some(k), None, Some(z)) | (ArOp::Mul, None, Some(k), Some(z)) => {
                let other = if x.is_some() { b } else { a };
                if k == 0 {
                    if z == 0 {
                        if let Some(id) = self.current.take() {
                            self.susps[id].done = true;
                        }
                        cx.ev(
                            self.id,
                            "equation",
                            format!("{text}: holds whatever {} is", self.show(&other, n)),
                        );
                        return Out::Progress;
                    }
                    return Out::Fail(format!("{text}: no number times 0 is {z}"));
                }
                if z % k != 0 {
                    return Out::Fail(format!("{text}: {z} is not a whole multiple of {k}"));
                }
                (other, z / k)
            }
            _ => {
                let mut on = vec![];
                for v in [&a, &b, &c] {
                    self.free_vars(v, &mut on);
                }
                cx.ev(self.id, "suspend", format!("{text}: waits for two of its numbers"));
                self.suspend(Goal::Arith(op, a, b, c), on, Why::Arith);
                return Out::Progress;
            }
        };
        let _ = op;
        cx.ev(self.id, "equation", format!("{text}: gives {}", value));
        self.equate(target, V::Num(value), cx)
    }

    fn call(&mut self, f: V, args: Vec<V>, cx: &mut Ctx) -> Out {
        let n = cx.opts.notation;
        let text = self.show_goal(&Goal::Call(f.clone(), args.clone()), n);
        match self.deref(&f) {
            V::Var(x) => {
                cx.ev(
                    self.id,
                    "suspend",
                    format!(
                        "{text}: {} has no value yet; the application waits",
                        self.cells[x as usize].name
                    ),
                );
                self.suspend(Goal::Call(f, args), vec![x], Why::Undefined);
                Out::Progress
            }
            V::Lam(p) => {
                if p.arity() != args.len() {
                    return Out::Fail(format!(
                        "{text}: the procedure binds {} variables, and is given {}",
                        p.arity(),
                        args.len()
                    ));
                }
                let mut env = vec![None; p.slot_names.len()];
                for (s, a) in p.params.iter().zip(&args) {
                    env[*s as usize] = Some(a.clone());
                }
                let mut waiting = vec![];
                for k in &p.key {
                    if let V::Var(x) = self.deref(env[*k as usize].as_ref().unwrap()) {
                        waiting.push(x);
                    }
                }
                if !waiting.is_empty() {
                    let names: Vec<String> = waiting
                        .iter()
                        .map(|x| self.cells[*x as usize].name.to_string())
                        .collect();
                    cx.ev(
                        self.id,
                        "suspend",
                        format!("{text}: waits for its key {}", names.join(", ")),
                    );
                    self.suspend(Goal::Call(f, args), waiting, Why::Key);
                    return Out::Progress;
                }
                cx.ev(self.id, "apply", format!("{text}: applied"));
                self.enter_in_place(&p.body, &Rc::new(env), cx.opts.select);
                Out::Progress
            }
            other => Out::Fail(format!("{text}: {} is not a procedure", self.show(&other, n))),
        }
    }

    fn tout(&mut self, l: V, t: V, cx: &mut Ctx) -> Out {
        let n = cx.opts.notation;
        let label = match self.deref(&l) {
            V::Str(s) | V::Atom(s) => s.to_string(),
            other => self.show(&other, n),
        };
        let mut free = vec![];
        self.free_vars(&t, &mut free);
        if free.is_empty() {
            let line = format!("{} = {}", label.trim(), self.show_val(&t, n));
            cx.ev(self.id, "output", line.clone());
            self.outputs.push(line);
        } else {
            cx.ev(
                self.id,
                "suspend",
                format!("tout(\"{label}\", …): waits until its term is complete"),
            );
            self.suspend(Goal::Tout(l, t), free, Why::Tout);
        }
        Out::Progress
    }

    /// The variables a negated assertion refers to from outside it.
    fn outer_vars(&mut self, g: &G, env: &Env, own: &mut Vec<u32>, out: &mut Vec<u32>) {
        let term = |d: &mut Deriv, t: &T, own: &Vec<u32>, out: &mut Vec<u32>| {
            fn walk(d: &mut Deriv, t: &T, env: &Env, own: &Vec<u32>, out: &mut Vec<u32>) {
                match t {
                    T::Slot(s) if own.contains(s) => {}
                    T::Slot(_) | T::Global(_) => {
                        let v = d.inst(t, env);
                        d.free_vars(&v, out);
                    }
                    T::Cons(h, tl) => {
                        walk(d, h, env, own, out);
                        walk(d, tl, env, own, out);
                    }
                    _ => {}
                }
            }
            walk(d, t, env, own, out)
        };
        match g {
            G::Eq(a, b) => {
                term(self, a, own, out);
                term(self, b, own, out);
            }
            G::Arith(_, a, b, c) => {
                for t in [a, b, c] {
                    term(self, t, own, out);
                }
            }
            G::Call(f, args) => {
                term(self, f, own, out);
                for a in args {
                    term(self, a, own, out);
                }
            }
            G::Tout(_, t) => term(self, t, own, out),
            G::And(v) => {
                for c in v {
                    self.outer_vars(c, env, own, out);
                }
            }
            G::Or(alts) => {
                for s in alts {
                    own.extend(&s.intro);
                    self.outer_vars(&s.goal, env, own, out);
                }
            }
            G::Not(s) => {
                own.extend(&s.intro);
                self.outer_vars(&s.goal, env, own, out);
            }
            G::Set(of, s, into) => {
                term(self, into, own, out);
                own.extend(&s.intro);
                term(self, of, own, out);
                self.outer_vars(&s.goal, env, own, out);
            }
        }
    }

    /// A copy of this derivation with nothing to do: the start of an independent computation.
    fn snapshot(&self, cx: &mut Ctx) -> Deriv {
        let mut d = self.clone();
        d.id = cx.id();
        d.parent = Some(self.id);
        d.ors.clear();
        d.eqs.clear();
        d.lits.clear();
        d.subs.clear();
        d.outputs.clear();
        d.status = Status::Running;
        d
    }

    fn not(&mut self, node: Rc<G>, scope: &Rc<Scope>, env: Env, cx: &mut Ctx) -> Out {
        let n = cx.opts.notation;
        let text = format!("not({})", self.show_g(&scope.goal, &env, n));
        if cx.opts.negation == Negation::Delayed {
            let mut own = scope.intro.clone();
            let mut free = vec![];
            self.outer_vars(&scope.goal, &env, &mut own, &mut free);
            if !free.is_empty() {
                cx.ev(
                    self.id,
                    "suspend",
                    format!("{text}: waits until its variables are instantiated"),
                );
                self.suspend(Goal::Do(node, env), free, Why::Negation);
                return Out::Progress;
            }
        }
        let mut d = self.snapshot(cx);
        d.enter(scope, &env);
        cx.ev(
            self.id,
            "not",
            format!(
                "{text}: an independent computation begins (#{}), with the criteria reversed",
                d.id
            ),
        );
        self.subs.push(Sub {
            kind: SubKind::Not,
            comp: Comp::new(vec![d]),
            base: self.cells.len(),
            text,
            goal: Goal::Do(node, env),
        });
        Out::Progress
    }

    fn set(&mut self, node: Rc<G>, of: &T, scope: &Rc<Scope>, into: &T, env: &Env, cx: &mut Ctx) -> Out {
        let n = cx.opts.notation;
        let into_v = self.inst(into, env);
        let base = self.cells.len();
        let mut d = self.snapshot(cx);
        let inner = d.open(scope, env);
        let of_v = d.inst(of, &inner);
        d.add(Goal::Do(scope.goal.clone(), inner.clone()));
        let text = format!(
            "{{set({}), {}, {}}}",
            d.show(&of_v, n),
            d.show_g(&scope.goal, &inner, n),
            self.show(&into_v, n)
        );
        cx.ev(
            self.id,
            "set",
            format!("{text}: an independent computation begins (#{})", d.id),
        );
        self.subs.push(Sub {
            kind: SubKind::Set { of: of_v, into: into_v },
            comp: Comp::new(vec![d]),
            base,
            text,
            goal: Goal::Do(node, env.clone()),
        });
        Out::Progress
    }

    fn step_sub(&mut self, cx: &mut Ctx) -> Out {
        let i = 0;
        cx.depth += 1;
        self.subs[i].comp.step(cx);
        cx.depth -= 1;
        let sub = &self.subs[i];
        let running = sub.comp.derivs.iter().any(|d| d.is_running());
        // A derivation that hibernates counts as a refutation only if what it
        // leaves suspended is equations between variables (and output): those
        // are always satisfiable (Elcock 1990 p. 12). One that waits for
        // anything else has not been decided.
        let refuted = sub.comp.derivs.iter().find(|d| !d.is_running() && d.is_refutation());
        match &sub.kind {
            SubKind::Not => {
                if let Some(d) = refuted {
                    return Out::Fail(format!(
                        "{}: the negated assertions hold (derivation #{} found no contradiction)",
                        sub.text, d.id
                    ));
                }
                if sub.comp.derivs.is_empty() {
                    let text = sub.text.clone();
                    self.subs.remove(i);
                    cx.ev(self.id, "not", format!("{text} holds: every derivation of it failed"));
                } else if !running {
                    self.undecided(i, cx);
                }
            }
            SubKind::Set { of, into } => {
                if !running && sub.comp.derivs.iter().any(|d| !d.is_refutation()) {
                    self.undecided(i, cx);
                } else if !running {
                    let (of, into) = (of.clone(), into.clone());
                    let sub = self.subs.remove(i);
                    let mut found: Vec<V> = vec![];
                    let mut map = HashMap::new();
                    for d in &sub.comp.derivs {
                        let v = d.resolve(&of);
                        let v = self.import(&v, sub.base, &mut map);
                        if !found.iter().any(|f| same(f, &v)) {
                            found.push(v);
                        }
                    }
                    let mut list = V::Nil;
                    for v in found.into_iter().rev() {
                        list = V::Cons(Rc::new((v, list)));
                    }
                    cx.ev(
                        self.id,
                        "set",
                        format!("{}: complete, {}", sub.text, self.show(&list, cx.opts.notation)),
                    );
                    self.add(Goal::Eq(into, list));
                }
            }
        }
        Out::Progress
    }

    /// A `not` or `set` whose computation is waiting for something: it is set
    /// aside, to be tried again from the start when what it waited for from
    /// outside is known. Waiting only for its own variables, it waits for ever.
    fn undecided(&mut self, i: usize, cx: &mut Ctx) {
        let sub = self.subs.remove(i);
        let mut on = vec![];
        for d in &sub.comp.derivs {
            for w in d.waiting().filter(|w| !matches!(w.why, Why::Equation | Why::Tout)) {
                for v in &w.on {
                    if !matches!(d.deref(&V::Var(*v)), V::Var(_)) {
                        continue;
                    }
                    // a variable the computation shares; or an identifier of the
                    // text it met first (a procedure not yet defined, say), which
                    // is the same identifier here
                    let outer = if (*v as usize) < sub.base {
                        Some(*v)
                    } else {
                        d.globals
                            .iter()
                            .find(|(_, g)| g == v)
                            .map(|(name, _)| name.clone())
                            .map(|name| self.global(&name))
                    };
                    if let Some(o) = outer {
                        if !on.contains(&o) {
                            on.push(o);
                        }
                    }
                }
            }
        }
        let names: Vec<String> = on.iter().map(|v| self.cells[*v as usize].name.to_string()).collect();
        cx.ev(
            self.id,
            "suspend",
            if names.is_empty() {
                format!("{}: cannot be decided; it waits on its own variables only", sub.text)
            } else {
                format!("{}: cannot be decided yet; waits for {}", sub.text, names.join(", "))
            },
        );
        self.suspend(sub.goal, on, Why::Undecided);
    }

    /// A value from an independent computation, in this derivation's terms.
    fn import(&mut self, v: &V, base: usize, map: &mut HashMap<u32, u32>) -> V {
        match v {
            V::Var(x) if (*x as usize) < base => v.clone(),
            V::Var(x) => {
                if let Some(y) = map.get(x) {
                    return V::Var(*y);
                }
                let y = self.new_var(Rc::from(format!("_{}", self.cells.len())));
                map.insert(*x, y);
                V::Var(y)
            }
            V::Cons(p) => {
                let h = self.import(&p.0, base, map);
                let t = self.import(&p.1, base, map);
                V::Cons(Rc::new((h, t)))
            }
            _ => v.clone(),
        }
    }

    // ---- display ----

    /// A term, with instantiated variables shown by their values.
    pub fn show(&self, v: &V, n: Notation) -> String {
        let mut s = String::new();
        self.write(v, n, &mut s);
        s
    }

    /// The same, for the value of a variable (shows what it is bound to, not its name).
    pub fn show_val(&self, v: &V, n: Notation) -> String {
        self.show(v, n)
    }

    fn write(&self, v: &V, n: Notation, out: &mut String) {
        match self.deref(v) {
            V::Var(x) => out.push_str(&self.cells[x as usize].name),
            V::Num(k) => out.push_str(&k.to_string()),
            V::Atom(s) => {
                out.push('\'');
                out.push_str(&s);
                out.push('\'');
            }
            V::Str(s) => {
                out.push('"');
                out.push_str(&s);
                out.push('"');
            }
            V::Nil => out.push_str("[]"),
            V::Lam(p) => {
                out.push_str("lambda ");
                let names: Vec<&str> = p.params.iter().map(|s| &*p.slot_names[*s as usize]).collect();
                if p.result {
                    let (last, rest) = names.split_last().unwrap();
                    out.push_str(&rest.join(", "));
                    out.push_str(" => ");
                    out.push_str(last);
                } else {
                    out.push_str(&names.join(", "));
                }
            }
            V::Cons(_) => {
                let (sep, bar) = match n {
                    Notation::Retrospective | Notation::Elcock1982 => (",", "|"),
                    Notation::Absys1 => (";", "&"),
                };
                out.push('[');
                let mut cur = self.deref(v);
                let mut count = 0;
                loop {
                    match cur {
                        V::Cons(p) => {
                            if count > 0 {
                                out.push_str(sep);
                            }
                            count += 1;
                            if count > MAX_ITEMS {
                                out.push('…');
                                break;
                            }
                            self.write(&p.0, n, out);
                            cur = self.deref(&p.1);
                        }
                        V::Nil => break,
                        other => {
                            out.push_str(bar);
                            self.write(&other, n, out);
                            break;
                        }
                    }
                }
                out.push(']');
            }
        }
    }

    pub fn show_goal(&self, g: &Goal, n: Notation) -> String {
        match g {
            Goal::Eq(a, b) => format!("{} = {}", self.show(a, n), self.show(b, n)),
            Goal::Arith(op, a, b, c) => {
                format!(
                    "{} {} {} = {}",
                    self.show(a, n),
                    op.sym(),
                    self.show(b, n),
                    self.show(c, n)
                )
            }
            Goal::Call(f, args) => {
                let f = match self.deref(f) {
                    V::Lam(p) => self
                        .globals
                        .iter()
                        .find(|(_, v)| matches!(&self.cells[*v as usize].val, Some(V::Lam(q)) if Rc::ptr_eq(&p, q)))
                        .map(|(name, _)| name.to_string())
                        .unwrap_or_else(|| self.show(f, n)),
                    _ => self.show(f, n),
                };
                let a: Vec<String> = args.iter().map(|a| self.show(a, n)).collect();
                format!("{f}({})", a.join(", "))
            }
            Goal::Tout(l, t) => format!("tout({}, {})", self.show(l, n), self.show(t, n)),
            Goal::Do(g, env) => self.show_g(g, env, n),
            Goal::Woken(id) => self.show_goal(&self.susps[*id].goal, n),
        }
    }

    /// A part of a body as it stands in this derivation (variables not yet created shown by name).
    pub fn show_g(&self, g: &G, env: &Env, n: Notation) -> String {
        let t = |t: &T| self.show_t(t, env, n);
        match g {
            G::Eq(a, b) => format!("{} = {}", t(a), t(b)),
            G::Arith(op, a, b, c) => format!("{} {} {} = {}", t(a), op.sym(), t(b), t(c)),
            G::Call(f, args) => {
                let a: Vec<String> = args.iter().map(t).collect();
                let f = match f {
                    T::Global(name) => name.to_string(),
                    _ => t(f),
                };
                format!("{f}({})", a.join(", "))
            }
            G::Tout(l, x) => format!("tout({}, {})", t(l), t(x)),
            G::And(v) => {
                let parts: Vec<String> = v.iter().map(|c| self.show_g(c, env, n)).collect();
                format!("{{{}}}", parts.join(", "))
            }
            G::Or(alts) => {
                let parts: Vec<String> = alts.iter().map(|s| self.show_g(&s.goal, env, n)).collect();
                parts.join(" or ")
            }
            G::Not(s) => format!("not({})", self.show_g(&s.goal, env, n)),
            G::Set(of, s, into) => format!("{{set({}), {}, {}}}", t(of), self.show_g(&s.goal, env, n), t(into)),
        }
    }

    fn show_t(&self, t: &T, env: &Env, n: Notation) -> String {
        match t {
            T::Slot(s) => match env.get(*s as usize).cloned().flatten() {
                Some(v) => self.show(&v, n),
                None => "?".into(),
            },
            T::Global(name) => match self.lookup(name) {
                Some(v) => self.show(&V::Var(v), n),
                None => name.to_string(),
            },
            T::Num(k) => k.to_string(),
            T::Atom(s) => format!("'{s}'"),
            T::Str(s) => format!("\"{s}\""),
            T::Nil => "[]".into(),
            T::Lam(_) => "lambda …".into(),
            T::Cons(..) => {
                let (sep, bar) = match n {
                    Notation::Retrospective | Notation::Elcock1982 => (",", "|"),
                    Notation::Absys1 => (";", "&"),
                };
                let mut parts = vec![];
                let mut cur = t;
                let mut tail = None;
                loop {
                    match cur {
                        T::Cons(h, tl) => {
                            parts.push(self.show_t(h, env, n));
                            cur = tl;
                        }
                        T::Nil => break,
                        other => {
                            tail = Some(self.show_t(other, env, n));
                            break;
                        }
                    }
                }
                match tail {
                    Some(tl) => format!("[{}{bar}{tl}]", parts.join(sep)),
                    None => format!("[{}]", parts.join(sep)),
                }
            }
        }
    }

    /// Processes waiting: suspended equations and the rest.
    pub fn waiting(&self) -> impl Iterator<Item = &Susp> {
        self.susps
            .iter()
            .filter(|s| !s.done && s.on.iter().any(|v| matches!(self.deref(&V::Var(*v)), V::Var(_))))
    }

    /// The relations associated with an identifier of the text and still
    /// waiting on it, as Elcock 1982 lists them: (u: u+v=16; w*u=v).
    pub fn associations(&self, name: &str, n: Notation) -> Vec<String> {
        let Some(v) = self.lookup(name) else { return vec![] };
        self.cells[v as usize]
            .waiting
            .iter()
            .filter(|id| !self.susps[**id].done)
            .map(|id| self.show_goal(&self.susps[*id].goal, n))
            .collect()
    }

    /// The binding environment, as the retrospective writes it: X₁/1, L₁/[2,U], …
    pub fn environment(&self, n: Notation) -> Vec<(String, String)> {
        self.cells
            .iter()
            .enumerate()
            .filter(|(_, c)| c.val.is_some() && !c.name.starts_with('_'))
            .filter(|(_, c)| !matches!(c.val, Some(V::Lam(_))))
            .map(|(i, c)| (c.name.to_string(), self.show(&V::Var(i as u32), n)))
            .collect()
    }

    /// What is left to do, for display.
    pub fn pending(&self, n: Notation) -> Vec<String> {
        let mut out = vec![];
        for (alts, env) in &self.ors {
            let parts: Vec<String> = alts.iter().map(|s| self.show_g(&s.goal, env, n)).collect();
            out.push(parts.join(" or "));
        }
        for g in self.eqs.iter().chain(self.lits.iter()) {
            out.push(self.show_goal(g, n));
        }
        for s in &self.subs {
            out.push(format!("{} (in progress)", s.text));
        }
        out
    }
}

/// Two values the same, as `set` counts them.
fn same(a: &V, b: &V) -> bool {
    match (a, b) {
        (V::Var(x), V::Var(y)) => x == y,
        (V::Num(x), V::Num(y)) => x == y,
        (V::Atom(x), V::Atom(y)) | (V::Str(x), V::Str(y)) => x == y,
        (V::Nil, V::Nil) => true,
        (V::Cons(p), V::Cons(q)) => same(&p.0, &q.0) && same(&p.1, &q.1),
        (V::Lam(p), V::Lam(q)) => Rc::ptr_eq(p, q),
        _ => false,
    }
}

/// A set of derivations, elaborated in turn.
#[derive(Clone, Debug)]
pub struct Comp {
    pub derivs: Vec<Deriv>,
    turn: usize,
}

impl Comp {
    pub fn new(derivs: Vec<Deriv>) -> Comp {
        Comp { derivs, turn: 0 }
    }

    pub fn running(&self) -> usize {
        self.derivs.iter().filter(|d| d.is_running()).count()
    }

    /// One step of one derivation. False when every derivation hibernates (or none is left).
    pub fn step(&mut self, cx: &mut Ctx) -> bool {
        let n = self.derivs.len();
        if n == 0 {
            return false;
        }
        let pick = match cx.opts.search {
            Search::Fair => (0..n)
                .map(|k| (self.turn + k) % n)
                .find(|i| self.derivs[*i].is_running()),
            Search::DepthFirst => (0..n).find(|i| self.derivs[*i].is_running()),
        };
        let Some(i) = pick else { return false };
        cx.steps += 1;
        let out = self.derivs[i].step(cx);
        match out {
            Out::Progress => self.turn = i + 1,
            Out::Idle => {
                let d = &mut self.derivs[i];
                d.status = Status::Hibernating;
                cx.ev(
                    d.id,
                    "hibernate",
                    "nothing more can be done: the derivation hibernates".into(),
                );
                self.turn = i + 1;
            }
            Out::Fail(why) => {
                let d = self.derivs.remove(i);
                cx.ev(d.id, "fail", why.clone());
                if cx.depth == 0 {
                    cx.failed.push((d.id, why));
                    if cx.failed.len() > 200 {
                        cx.failed.remove(0);
                    }
                }
                self.turn = i;
            }
            Out::Split(kids) => {
                let k = kids.len();
                if cx.depth == 0 {
                    cx.tree.push((self.derivs[i].id, kids.iter().map(|d| d.id).collect()));
                }
                self.derivs.splice(i..i + 1, kids);
                self.turn = i + k;
            }
        }
        true
    }
}
