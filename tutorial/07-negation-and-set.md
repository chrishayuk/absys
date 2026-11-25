# 7. Negation and sets

These are in the 1990 notation only. The retrospective attributes both to
Absys-4, circa 1968.

## not

Elcock (1990, p. 14): `not` "acts like a degenerate or in that during goal
evaluation it initiates an independent computational branch, but one in
which the satisfiability criteria for termination are reversed". So
`not(A)` holds if every derivation of A fails, and fails if any derivation
of A succeeds.

The retrospective renders Clark's student database as a completion. It is
transcribed here with two misprints corrected (`sources/notes/elcock-1990.md`):

```absys
student = lambda X
    {X = 'BROWN'} or {X = 'SMITH'}
math-course = lambda C
    {C = 'C101'} or {C = 'C301'}
takes = lambda X, C
    {X = 'BROWN', C = 'C101'} or
    {X = 'SMITH', C = 'C101'} or
    {X = 'SMITH', C = 'C301'}
non-maths-major = lambda X
    {math-course(C), not(takes(X, C))}
non-maths-major('SMITH')
```

```output
No derivation survives: the assertions are unsatisfiable.
```

Smith takes both maths courses. Brown does not take C301, and
`non-maths-major('BROWN')` holds (`examples/students.abs`).

## not is not safe

`not` is decided on the bindings as they stand, and it does not wait for its
variables. Elcock (note 18): "both Absys and PROLOG have the shortcoming that
they do not use a safe computation rule". So, unlike everything else in
Absys, the order matters:

```absys
X = 2
not(X = 1)
```

```output
1 derivation:
  derivation #0 (hibernating)
    X = 2
```

```absys
not(X = 1)
X = 2
```

```output
No derivation survives: the assertions are unsatisfiable.
```

In the second text, X has no value when `not` is elaborated, so `X = 1` can
hold, and the `not` fails. `absys --negation delayed` makes `not` wait until
its variables are instantiated. That is an experiment, not Absys
(`docs/research.md`, E02).

## set

"The Absys primitive aggregation operator set" (p. 15) collects. The
retrospective's example is from "a demonstration of Absys-4 circa 1968":

```absys
mem = lambda X, S
    {S = [X|T]} or {S = [Y|T], mem(X, T)}
setdiff = lambda S1, S2, S
    {set(X), {mem(X, S1), not(mem(X, S2))}, S}
setdiff([1,2,3,4], [2,4,6], D)
```

```output
1 derivation:
  derivation #0 (hibernating)
    procedures  mem, setdiff
    D = [1,3]
```

`{set(X), {…}, S}` is written as the retrospective prints it. It starts an
independent computation, as `not` does. When that computation is finished, S
is equated with the list of the distinct values X took in the derivations
that succeeded. The paper does not define `mem`; the definition above is
ours.
