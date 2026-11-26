# ABSYS-E01 to E06, and E09: experiments on the reconstruction

These are experiments on the reconstructed machine. They are not claims about
the Elliott 4120 program, which does not survive. Three kinds of statement
are kept apart:

- **Historical**: what the sources say. It is listed in
  `docs/decisions.md`, with pages.
- **Reconstruction**: what this machine does where the sources are silent.
  These are the "Decision" entries of `docs/decisions.md`.
- **Finding**: what running the machine shows. Each finding is reproducible
  with the command given beside it, and the numbers are from this
  repository's build.

No Prolog system was installed where these were run. Where Prolog is
mentioned, it is described from its well-known standard behaviour, and
nothing was run. A side-by-side comparison with a running Prolog is still to
do.

The texts are in `research/`, except where an example from `examples/` is
named.

## E01. Can one procedure operate in several directions?

**Yes, for every direction tried. In some directions the computation finds
its answer and does not stop.**

`app` from Elcock 1990 p. 8, with one goal at a time
(`tests/historical.rs`, `e90_p8_one_procedure_in_every_direction`):

| goal | derivations left | answer |
|---|---|---|
| `app([1,2], [3], T)` | 1 | `T = [1,2,3]` |
| `app([1], S, [1,2,3])` | 1 | `S = [2,3]` |
| `app(R, [3], [1,2,3])` | 1 | `R = [1,2]` |
| `app(R, S, [1,2])` | 3 | `R = [], S = [1,2]`; `R = [1], S = [2]`; `R = [1,2], S = []` |
| `app([1,2], S, T)` | 1 | `T = [1,2\|N₂]`, with `S = N₂` **suspended** |
| `app(R, S, T)` | grows without end | each derivation hibernates with `S = Nₖ` suspended, `T = [X₁,…,Xₖ\|Nₖ]` |
| `rev([1,2,3], S)` | 1 | `S = [3,2,1]` |
| `rev(R, [1,2,3])` | 1 hibernating, and 2 still going when the budget ran out | `R = [3,2,1]` |

Findings:

- **The answer to `app([1,2], S, T)` is a partially solved form.** Prolog
  would answer `T = [1,2|S]` by making S and the tail one variable. Absys
  keeps the equation between them suspended. This is Elcock's "caveat" (p.
  11), seen in a case he does not give.
- **Run backwards, `rev` finds its answer and goes on for ever.** The
  derivation that keeps lengthening `R` never fails, because nothing bounds
  the length of `R` until the equation at the end of the recursion. Under
  fair search the answer is reached, and found, while that derivation runs
  on. Prolog's `reverse` run backwards finds the answer and then loops on
  backtracking. Fairness changes when the answer appears, not whether the
  computation stops.
- **Arithmetic in the 1969 notation is invertible as the paper says.** `a =
  b + 1`, `a = 1 + b`, `1 + b = a` and `b + 1 = a` each give b from a, or a
  from b (`fe69_arithmetic_in_every_direction`).

## E02. How much does the order of the assertions matter?

**For what a text computes: not at all, with one exception, `not`. For what
it costs: a great deal.**

`absys permute FILE` accepts the assertions of a text in every order, each
assertion with a budget of 20000 steps, and groups the orders by outcome:

| text | orders | outcomes | steps |
|---|---|---|---|
| `examples/reverse.abs` | 24 | 1 (`L = [2,1], U = 5, V = 5`) | 35–40 |
| `research/e02-order.abs` (`app` defined, `app(R, S, T)`, `T = [1,2]`) | 6 | 1 (the three splits of `[1,2]`) | 25–22868 |
| `research/e03-incremental.abs` (7 assertions) | 5040 | 1 | 34–22021 |
| `examples/students.abs` | 120 | 1 (Brown is a non-maths-major) | 42–64 |
| `examples/setdiff.abs` | 6 | 1 (`D = [1,3]`) | 198–212 |
| `research/e02-negation-order.abs` (`not(X = 1)`, `X = 2`) | 2 | **2** | 5–6 |

Findings:

- **Order-independence holds for every text without `not` that was tried**,
  including the orders in which a procedure is applied before it is defined,
  and the orders in which a goal runs out of budget before the assertion that
  bounds it arrives. In `e03-incremental.abs`, 1260 of the 5040 orders ran
  out of budget on some assertion. Each of them still reached the same
  answer, because the later assertion was added to the derivations still
  running and cut them down. This is what Foster and Elcock (1969) claim: "if
  data can be constructed to satisfy all the assertions about them, then
  these data are independent of the order in which the operations are
  performed".
- **The cost is not independent of the order.** The same three answers cost
  25 steps or 22868, depending on whether `T = [1,2]` arrives before or after
  `app(R, S, T)`.
- **`not` makes the order matter**, as Elcock says it can (note 18: Absys had
  no safe computation rule):
  - `not(X = 1)` then `X = 2` leaves no derivation.
  - `X = 2` then `not(X = 1)` leaves `X = 2`.
  - With `--negation delayed`, which is not Absys, both orders give `X = 2`.
- **The literal rule matters through `not` too.** Elcock's own `setdiff`
  (p. 15) gives `D = [1,3]` when the body of an application takes the
  literal's place, the reconstruction's default. Strictly first-in-first-out
  (`--select fifo`) gives `D = [1]`, a wrong answer. In FIFO, `not(mem(X,
  S2))` is elaborated before the recursion of `mem(X, S1)` has instantiated
  X, in the derivations that recurse (`tests/historical.rs`,
  `decision_15_fifo_breaks_setdiff`). This finding is the evidence for
  decision 15. The paper says Absys "used a fair computation rule", and also
  shows `setdiff` working. Under this machine those two statements cannot
  both hold, unless `not` was delayed in some way the paper does not
  describe.
- **A reconstruction decision that the permutations forced.** In a first
  version of the machine, 60 of the 120 orders of the student text gave no
  derivation, and 2 of the 6 orders of `setdiff` gave `D = []`. Those were
  the orders in which `takes` or `mem` was defined after the goal. The
  independent computation of `not` and `set` waited for the definition, and
  a derivation that only waited was counted as a refutation. Elcock counts a
  derivation that ends with suspended equations as a refutation because they
  "are always satisfiable" (p. 12). A waiting application is not. The
  machine now counts as refutations only derivations whose suspended
  processes are equations between variables. A `not` or `set` that cannot be
  decided waits, and starts again when what it waited for is known
  (`docs/decisions.md`, rule 18).

## E03. Can partially resolved computation survive additional assertions?

**Yes. It is the normal mode of the system, not a special case.**

- `examples/reverse.abs` is the retrospective's example. The derivation
  hibernates with `U = X₃` and `X₃ = V` suspended. `rev([5], [V])` is added,
  and its binding environment becomes the retrospective's, variable for
  variable, including the names `X₄` and `L₄` (`tests/historical.rs`,
  `e90_p13_an_assertion_added_later`).
- `research/e03-incremental.abs` applies `rev` before `rev` or `rev1` is
  defined, and before the list's members are known (`absys
  research/e03-incremental.abs --each`):
  - the application waits for `rev`;
  - when `rev` is defined, it waits for `rev1`;
  - when `rev1` is defined, it runs, and leaves the members of B tied to
    those of A by suspended equations;
  - `Y = Z` and `B = [42|C]` then fill everything in.
- What survives between assertions is not only bindings. Suspended
  equations, waiting applications, `not`s and `set`s that cannot yet be
  decided, and derivations still running all survive. The last kind is the
  1260 runs above that ran out of budget and were cut down by a later
  assertion.

## E04. Can fair scheduling resolve problems where depth-first search stalls?

**Yes, when an alternative that never ends comes before one that succeeds.
It is not needed for left recursion when the data is known, because the
equations-first rule already handles that.**

| text | `--search fair` (Absys) | `--search depth-first` |
|---|---|---|
| `research/e04-fair.abs`: `{loop(A)} or {A = 1}` | `A = 1` found; the loop still running at 100000 steps | after 100000 steps the second alternative has not been touched (`to do A = 1`) |
| `rev(R, [1,2,3])` | `R = [3,2,1]` found, 2 derivations still going | the same, because the answering alternative comes first in that program |
| `research/e04-left-recursion.abs`: `as = lambda L {as(M), L = ['a'\|M]} or {L = []}`, with L known | `L = ['a','a']`, terminates | terminates too |

Findings:

- **Fairness is what lets a later alternative be reached past an endless
  earlier one.** That is what Elcock's "all derivations continue to be
  incrementally elaborated until found unsatisfiable" buys (p. 10).
- **Left recursion over known data terminates in Absys under either
  search.** Equations are elaborated as they arise (decision 12), so `L =
  ['a'|M]` instantiates M before `as(M)` is applied, and the recursion is
  bounded. Prolog's leftmost rule calls `as(M)` with M unbound first, and
  descends for ever (expected from Prolog's standard rule, not run here).
  This holds under `--select fifo` too.

## E05. Can a computation be represented as an evolving graph of constraints?

**Yes. Two graphs come straight out of the machine's own state.**

`absys graph FILE` writes Graphviz dot.
`research/e05-graph.abs` is the reverse example:

```sh
absys graph research/e05-graph.abs | dot -Tsvg > e05.svg
```

- **The derivation tree.** There is a node for each derivation, with an
  edge from each derivation to the derivations its `or` distributed into.
  Failed derivations are marked with the equation that failed. The ones
  alive are marked with their answers. For the reverse example the tree is a
  spine, `#0 → #2 → #4 → #6 → #7`, and each step leaves one failed sibling,
  `R = []` against a list that is not empty. The last one, `#8`, fails on
  `[] = [X₄|L₄]`.
- **The suspension graph of each derivation left.** The nodes are the
  uninstantiated variables. An edge joins two variables for each suspended
  equation between them, and a note marks each other waiting process, with
  edges to what it waits on. For the reverse example the graph is the path
  `U — X₃ — V`, which is Elcock's "partially solved form constrained by the
  suspended equations" (p. 12). The connected parts of this graph are what
  `absys` uses to answer a query: asked for U, it says `U = V`, by `U = X₃`
  and `X₃ = V`.

This is an analysis of the reconstruction. Elcock (1990) and Elcock,
McGregor and Murray (1972) describe the implementation as identifiers with
lists of suspended processes, which is a graph of this kind. Nothing here
claims the Aberdeen program drew or used it as one. The playground draws the
derivation tree and the suspended equations after each assertion.

## E06. Does the machine keep the list of relations Elcock printed in 1982?

**Yes, every list and every association list in it, once two rules are
changed to follow it. One of the two contradicts a word of the same
passage.**

Elcock (1982, p. 123; `sources/notes/elcock-1982.md`) walks through
`u+v = 16 and w*u = v and u+10 = 12` relation by relation, printing the
association lists and the list of relations still to be elaborated. The
1982 notation reads that program as written (`examples/relation-list.ab2`).
`tests/historical.rs` (`e82_p123_the_list_of_relations`) steps the machine
one relation at a time and compares each state:

| after | Elcock prints | the machine |
|---|---|---|
| u+v = 16 and w*u = v examined | (u: u+v = 16; w*u = v); (v: u+v = 16; w*u = v); (w: w*u = v) | the same |
| u+10 = 12: u is 2 | (2+v = 16; w*2 = v) | the same |
| 2+v = 16: v is 14 | (2+14 = 16; w*2 = 14; w*2 = 14) | the same |
| 2+14 = 16 "found satisfied"; w*2 = 14: w is 7 | (7*2 = 14; 7*2 = 14) | the same |
| both "found satisfied" | empty | empty; u = 2, v = 14, w = 7 |

Findings:

- **A relation goes back on the list for each of its variables that is
  bound, even after it has been elaborated.** That is why w*2 = 14 is on the
  list twice, and why 2+14 = 16 is examined again and "found satisfied".
  The first version of the machine returned a suspended relation only once,
  and printed different lists. It now follows E82 for equations and
  arithmetic (`docs/decisions.md`, rule 10).
- **The printed lists contradict the word "appended".** Elcock says each
  returned list is "appended", but his third list comes out only if the
  returned relations go to the head of the list. Appended to the end it
  would be (w*2 = 14; 2+14 = 16; w*2 = 14), and the next relation examined
  would bind w, not be "found satisfied" (`e82_p123_appended_literally`).
  The final bindings are the same either way. The example settles where
  the relations go, if it was transcribed from a run; the word settles it
  the other way, if the example was written out by hand.
- **The 1982 paper contradicts the 1990 one on the search.** In 1982, "the
  non-determinism was handled by appropriate differential record keeping
  and backtracking in a similar spirit to Prolog implementations"; in 1990,
  "the implementation of Absys used a 'fair' search rule". It also dates
  Absys "essentially completed in 1968", where 1990 has 1967. Neither
  example in either paper tells a fair search from a backtracking one.
- E82's second example, `mem(x,[1,2,3])` then `mem(x,[2,4,6])` with `mem`
  keyed on its list, comes out as it says: three derivations, then one,
  x = 2 (`e82_p122_mem_with_a_key`).

## E09. Where does data-directed elaboration stop?

**At the first system of equations that needs elimination, as Elcock said
in 1982.**

Each arithmetic relation is taken up when two of its three numbers are
known (Foster's "pass sets"). So a system is solved exactly when it can be
ordered so that each equation, in turn, has one unknown left:

| text | result |
|---|---|
| `research/e09-triangular.ab1`: x + y + z = 12, y + z = 7, z = 3 | x = 5, y = 4, z = 3, in all 6 orders |
| `research/e09-simultaneous.ab1`: x + y = 10, x - y = 4 | nothing inferred: both relations hibernate, each waiting for a second number |

Elcock (1982, p. 124): "a naive dynamic data directed flow of elaboration
... of itself rapidly runs out of steam ... much more sophisticated
aggregation and solution methods would be necessary to deal with a
conjunction of a general set of linear equations in n variables. Even the
task of recognizing what aggregations of individual conjuncts might lend
themselves to reorganization as a specified 'higher' relation (e.g. the
relation 'simult-eqs(L)' ...) is challenging to say the least."

The machine shows the boundary but does not cross it. The suspended
relations it is left holding are the input such an aggregation would need:
in the simultaneous case, two relations sharing the same two unknowns, which
the suspension graph of `absys graph` draws as a cycle. Recognising that
shape and handing it to an eliminator, as the Nine Chapters' counting board
or Babbage's program of 1837 would solve it, is the step Elcock named and
did not take. It is not attempted here.

## Still to do

- Compare against running Prolog programs, the same programs in their
  homogeneous form (Elcock 1990, §2.3), now that the reconstruction passes
  its acceptance tests.
- The full text of Foster and Elcock 1969. Only extracts have been seen. Its
  last pages may describe negation and sets in Absys 1, or the order of
  processing.
- Elcock and Gray (1988), *Absys, equation solving and logic programming*,
  the "fuller description" the retrospective cites (its reference [9]). It
  may settle decisions 10, 14, 15, 19 and 20.
- Elcock, McGregor and Murray (1972), "Data directed control and operating
  systems", which E82 cites for the evaluation mechanism "in detail".
- Cost hints (F68's "a number may be associated with an assertion to
  indicate the cost of attempting it"), as an experiment, and ABSET.
