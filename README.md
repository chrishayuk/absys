# absys

Absys, the assertional language J. M. Foster and E. W. Elcock built at the
University of Aberdeen around 1967, reconstructed from what its designers
published about it.

An Absys text is a conjunction of assertions, not a sequence of
instructions. As each assertion is accepted, whatever can be done is done:

- An equation between two unknowns is **suspended** until one of them is
  known.
- A procedure is a relation, used in whichever direction the data allow.
- An `or` splits the computation into **derivations** that are all
  elaborated, fairly, until each fails or **hibernates**. A hibernating
  derivation wakes when a later assertion gives it something to do.

```
$ absys examples/reverse.abs --each --env
…
» rev([1,2,U], [V|L])
  1 derivation:
    derivation #7 (hibernating)
      procedures  rev1, rev
      U = V   (suspended: U = X₃, X₃ = V)
      V = U   (suspended: U = X₃, X₃ = V)
      L = [2,1]
      suspended   U = X₃;  X₃ = V
      environment L/[2,1], X₁/1, L₁/[2,U], X₂/2, L₂/[U], L₃/[]

» rev([5], [V])
  1 derivation:
    derivation #11 (hibernating)
      procedures  rev1, rev
      U = 5
      V = 5
      L = [2,1]
      environment U/5, V/5, L/[2,1], X₁/1, L₁/[2,5], X₂/2, L₂/[5], X₃/5, L₃/[], X₄/5, L₄/[]
```

Those are Elcock's own binding environments (*J. Logic Programming* 9, 1990,
pp. 12-13), variable for variable.

## Status

- **The sources.** No Absys program text, listing or manual from Aberdeen is
  known to us. The reconstruction rests on:
  - Elcock's 1990 retrospective, read in full from the page scans;
  - Elcock's 1982 conference paper comparing Absys with Prolog, which walks
    through Absys's list of relations step by step;
  - Foster's 1968 *Assertions* paper, read in full;
  - OCR extracts of most of the sections of Foster and Elcock's 1969
    introduction to Absys 1.

  See [`sources/README.md`](sources/README.md).
- **Every rule of the machine is listed with its evidence** in
  [`docs/decisions.md`](docs/decisions.md). Each is marked as one of three
  kinds:
  - what a source says, with its page;
  - a decision, where the sources are silent or conflict, with the reason;
  - an experiment, which is not historical.
- **Every computation the sources give a result for comes out as they
  say.** There are 17 acceptance tests, each naming its page
  ([`tests/historical.rs`](tests/historical.rs)). They include every list of
  relations awaiting elaboration that Elcock printed in 1982.
- **Three notations run on the one machine:**
  - the retrospective's (`.abs`), which Elcock says takes liberties with
    the original;
  - the 1969 notation of Absys 1 (`.ab1`), as far as the extracts show it;
  - the 1982 paper's (`.ab2`), with `<< … >>` and `and`.
- **Not reconstructed.** The higher-order features and "simple control
  mechanisms" of Absys-4 are described only in a report not seen, so they
  are not reconstructed.
- **Not yet checked.** Nothing has been checked by a person against the
  sources. No literature search beyond the sources listed has been made, so
  nothing here is claimed as new.

## What reconstructing it found

- **Absys does not unify two unknowns. It waits.**
  - `app([1,2], S, T)` ends with `T = [1,2|N₂]`, and the equation `S = N₂`
    suspended. Asked for U after the reverse example, the system says `U =
    V`, by the suspended `U = X₃` and `X₃ = V`.
  - This is Elcock's "partially solved form", and it is the ordinary result
    of a computation here, not a corner case.
- **The order of the assertions does not change what a text computes,
  except through `not`.**
  - The seven assertions of `research/e03-incremental.abs` were accepted in
    all 5040 orders, and gave one outcome. That held although the procedure
    `rev` is applied before it is defined in many of the orders, and 1260 of
    the orders run out of budget on some assertion before a later one
    bounds it.
  - The cost does change with the order: the same three answers of `app`
    cost 25 steps or 22868.
  - `not(X = 1)` then `X = 2` has no derivation; the other order has `X = 2`.
    That is Elcock's "Absys and PROLOG … do not use a safe computation
    rule".
- **Elcock's own `setdiff` favours one answer to how a derivation chose its
  next literal.** The paper prints only its definition; the inputs and the
  definition of `mem` used here are ours.
  - Taking literals strictly in turn, which is a "fair computation rule",
    computes `[1]` for the difference of `[1,2,3,4]` and `[2,4,6]`.
    `not(mem(X, S2))` is then decided before the recursion has instantiated
    X.
  - Putting an applied body in the place of its literal, which is what the
    paper's "replaced by" says, computes `[1,3]`.
  - Taking literals strictly in turn with `not` delayed until its
    variables are instantiated also computes `[1,3]`, and Elcock mentions
    delays he does not describe. So the example favours the body in the
    literal's place without proving it (`docs/decisions.md`, rule 15).
- **Elcock's 1982 walk-through fixes how relations come back.** A relation
  goes back on the list once for each of its variables that is bound, even
  after it has run, and goes to the head of the list. The paper says
  "appended", but its printed lists come out only if they go to the head.
  The same paper says the alternatives were implemented by "backtracking in
  a similar spirit to Prolog"; the 1990 one says the search was fair
  (`docs/research.md`, E06).
- **Data-directed elaboration stops at the first system that needs
  elimination**, as Elcock said in 1982: x + y = 10 and x - y = 4 leave both
  relations waiting (E09).
- **Run on SWI-Prolog, the same programs give the same answers where both
  answer (E07).** Prolog's left-to-right rule loses three:
  - an endless first alternative;
  - left recursion over a known list;
  - Elcock's arithmetic in his own order.

  Absys gets all three. Coroutines on Foster's pass sets (`when/2`) and
  CLP(FD) recover them in modern Prolog. Only CLP(Q), which treats the
  equations as one system, solves x + y = 10, x - y = 4.
- **Fair search finds what depth-first search never reaches.**
  `{loop(A)} or {A = 1}` gives `A = 1` while the loop runs on. Left
  recursion over known data terminates under either search, because
  equations are elaborated as they arise.
- **Two misprints in the retrospective's example programs**:
  - `C - 'C101'` for `C = 'C101'`;
  - `maths-course` applied where `math-course` is defined.

  Transcribed literally, the second would leave Clark's query waiting for a
  procedure that is never defined. See
  [`sources/notes/elcock-1990.md`](sources/notes/elcock-1990.md).

The experiments, with their numbers, are in
[`docs/research.md`](docs/research.md) (ABSYS-E01 to E07, and E09).

## Install and use

You need [Rust](https://rustup.rs). There are no dependencies.

```bash
cargo install --path .
absys examples/append.abs           # every derivation left at the end
absys examples/reverse.abs --each   # the state after each assertion
absys examples/append.abs --trace   # every step
absys examples/append.abs --env     # whole binding environments, as Elcock writes them
absys repl                          # the incremental system at the keyboard
absys permute FILE                  # accept the assertions in every order
absys graph FILE | dot -Tsvg        # the derivation tree and the suspended processes
absys decisions                     # the rules of the machine, and what each rests on
absys verify                        # every example against its .out
cargo test                          # the acceptance tests, the examples, the tutorial
```

Options, for experiments, not history:

- `--search depth-first`
- `--select fifo`
- `--negation delayed`
- `--steps N`

## A text

```
% Elcock 1990, p. 8
app = lambda R, S, T
    {R = [], S = T} or
    {R = [X|L], T = [X|N], app(L, S, N)}

app(R, S, [1,2])          % three derivations: the three ways of splitting [1,2]
```

The same thing in the 1969 notation looks quite different. Here is Foster
and Elcock's own example:

```
item'=lambda x=>z key x
    begin x = [p'&q'] {z=p or z=item(q)} end
p1 = [1;2;5;6]   p2 = [3;7;1]   p3 = [10;2;7;4]
r'=item(p1)   s'=item(p2)   t'=item(p3)   r+s=t
```

It leaves three derivations: r, s, t are 1, 3, 4; 1, 1, 2; and 6, 1, 7. Those
are the paper's three, in its order.

## In this repository

- [`tutorial/`](tutorial): the language, in ten chapters. Every example in
  it is run by `cargo test`.
- [`examples/`](examples): every program the sources give, each with its
  `.out`.
- [`research/`](research): the texts of the experiments.
- [`web/`](web): the playground.
  - `node web/build.mjs` builds it.
  - Serve `web/` and open `index.html`.
  - For any text it shows: each assertion; the derivations after it, with
    their bindings, suspended equations and waiting processes; the tree of
    alternatives; and every step.
  - `node web/test.mjs` checks the WebAssembly build against the native
    one, and `node web/browser-test.mjs` drives the page in headless
    Chrome.
- [`editors/`](editors): `absys lsp`, a language server, and a Visual Studio
  Code extension with a grammar and a run button.

## Licence

Apache-2.0. The papers belong to their publishers and are not included.
