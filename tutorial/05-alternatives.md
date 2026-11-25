# 5. Alternatives, and derivations

`A or B` asserts alternatives. When an `or` is reached, it distributes
through the goal of the computation it is in. Elcock (1990, p. 9) says it
creates "two new goals which can notionally be regarded as initiating
parallel derivations". Foster and Elcock (1969) put it this way: "the
conjunctions in effect constitute parallel non-interacting programs.
Unsatisfiable conjunctions disappear when unsatisfiability is detected."

So the computation is a set of **derivations**. Each has its own bindings,
its own suspended processes, and its own rest of the goal. A derivation in
which an equation fails disappears.

```absys --each
P = [1, 2]
{P = [A, B]} or {P = [B, A]}
A = 2
```

```output
» P = [1, 2]
  1 derivation:
    derivation #0 (hibernating)
      P = [1,2]

» {P = [A, B]} or {P = [B, A]}
  2 derivations:
    derivation #1 (hibernating)
      P = [1,2]
      A = 1
      B = 2
    derivation #2 (hibernating)
      P = [1,2]
      B = 1
      A = 2

» A = 2
  1 derivation:
    derivation #2 (hibernating)
      P = [1,2]
      B = 1
      A = 2
```

This is the 1969 paper's own example, written in the 1990 notation.

## Fair search

Every derivation is elaborated a step at a time, in turn. Elcock (1990, p.
10): "the implementation of Absys used a 'fair' search rule, in that all
derivations continue to be incrementally elaborated until found
unsatisfiable". So a derivation that never ends cannot hide one that does:

```absys --steps 2000
loop = lambda X
    {loop(X)}
{loop(A)} or {A = 1}
```

```output
  (the budget of 2000 steps ran out with derivations still elaborating)
2 derivations (1 still elaborating when the budget ran out):
  derivation #1 (still elaborating)
    procedures  loop
    A  uninstantiated
    to do       loop(A)
  derivation #2 (hibernating)
    procedures  loop
    A = 1
```

A backtracking, depth-first search, as in Prolog, would never leave the first
alternative. `absys --search depth-first` does that, for comparison
(`docs/research.md`, E04). Every assertion has a budget of steps (100000 by
default, `--steps N`), because a derivation can go on for ever, in Absys as
in Prolog.

## The order within a derivation

Within one derivation, equations are elaborated as they arise, before any
further application (Elcock 1990, pp. 6 and 9). The body of an application
takes the place of the literal applied. How much this matters, and the
evidence for it, is in `docs/decisions.md` (rules 12 and 15).
