# Elcock 1990: what was read, and how

The copy used is the 17-page PDF of the Elsevier open archive (CORE
82537271), a scan made with Acrobat Capture in 2001. Its text layer is OCR,
and it garbles every program (`app(Ll, U, NJ`, `x,/l*`). Every program and
binding environment below was therefore read from the page images at 110 and
260 dpi, not from the text layer.

The retrospective's own warning applies to everything here: "some slight
liberties have been taken with the original syntax of the Absys text", and
"what is referred to here as 'Absys' is a subset of the language Absys-4,
circa 1968" (p. 3).

## The programs, as printed

p. 8, append:

```text
app = lambda R, S, T
{R = [ ], S = T} or
{R = [X|L], T = [X|N], app(L, S, N)}
```

p. 9, the application of `app` to `app([1], U, [1,2])`:

```text
{[1] = [ ], U = [1,2]} or
{[1] = [X₁|L₁], [T₁ = [X₁|N₁]], app(L₁, U, N₁)}
```

**Misprint.** The middle equation of the second alternative should be
`[1,2] = [X₁|N₁]`. That is what substituting the arguments gives, and the
next display on the page has it. As printed, it has a stray `T₁` and extra
brackets.

pp. 9-10, the refutation:

```text
[1] = [X₁|L₁], [1,2] = [X₁|N₁], app(L₁, U, N₁)
1 = X₁, [ ] = L₁, [1,2] = [X₁|N₁], app(L₁, U, N₁)        by rule 1
1 = 1, [2] = N₁, app(L₁, U, N₁)
X₁/1, L₁/[ ], N₁/[2]   and the goal app([ ], U, [2])
[ ] = [ ], U = [2]
X₁/1, L₁/[ ], N₁/[2], U/[2]
```

Note 11: "the example refutation would cause the output 'assertion'
*tout*("U", U) to produce 'U = [2]'".

p. 11, reverse:

```text
rev1 = lambda R, S, T
{R = [ ], S = T} or
{R = [X|L], rev1(L, [X|S], T)}
rev = lambda R, S
{rev1(R, [ ], S)}
```

p. 12, for the goal `rev([1, 2, U], [V|L])`:

```text
rev1([1,2,U], [ ], [V|L])
[1,2,U] = [X₁|L₁],  rev1(L₁, [X₁], [V|L])          X₁/1, L₁/[2,U]
[2,U] = [X₂|L₂],  rev1(L₂[X₂,1], [V|L])            X₁/1, L₁/[2,U], X₂/2, L₂/[U]
[U] = [X₃|L₃],  rev1(L₃, [X₃,2,1], [V|L])          … L₃/[ ], and U = X₃ associated with U and X₃
[ ] = [ ],  [X₃,2,1] = [V|L]                        X₃ = V, [2,1] = L
binding environment   X₁/1, L₁/[2,U], X₂/2, L₂/[U], L₃/[ ], L/[2,1]
suspended equations   U = X₃ and X₃ = V
```

**Misprint.** `rev1(L₂[X₂,1], [V|L])` is missing a comma after `L₂`.

p. 13, after `rev([5], [V])` is added:

```text
X₁/1, L₁/[2,5], X₂/2, L₂/[5], L₃/[ ], L/[2,1],
X₄/5, L₄/[ ], V/5, X₃/5, U/5
```

p. 15, Clark's student database "rendered in Absys" ("with apologies for
the flip in syntax"):

```text
student = lambda X
    {X = 'BROWN'} or {X = 'SMITH'}
math-course = lambda C
    {C = 'C101'} or {C = 'C301'}
takes = lambda X, C
    {X = 'BROWN', C = 'C101'} or
    {X - 'SMITH', C - 'C101'} or
    {X = 'SMITH', C = 'C301'}
non-maths-major = lambda X
    {maths-course(C), not(takes(X, C))}
```

There are two misprints here, and `examples/students.abs` corrects both:

- **`-` for `=`.** The second alternative of `takes` prints `X - 'SMITH'`
  and `C - 'C101'`. Compare the line above and the line below it.
- **`maths-course` for `math-course`.** The body of `non-maths-major`
  applies `maths-course`, but the procedure defined is `math-course`, as in
  Clark's clauses printed above it. Transcribed literally, `maths-course`
  is never defined, and the reconstruction would leave the application
  waiting for a definition.

The quotes are typographic (‘BROWN’) in the print. They are transcribed as
ASCII `'`.

p. 15, from "a demonstration of Absys-4 circa 1968":

```text
setdiff = lambda S₁, S₂, S
    {set(X), {mem(X, S₁), not(mem(X, S2))}, S}
```

- The subscripts are written `S1` and `S2`.
- `S2` in the `not` is printed without its subscript.
- `mem` is not defined in the paper. `examples/setdiff.abs` supplies a
  definition, marked as ours.
- The form `{set(X), {…}, S}` is transcribed as printed. Whether the
  original was this, or something like `set(X, {…}, S)` that was
  re-bracketed in printing, cannot be told from the paper.

## What the paper says about the machine

The passages that the rules of `docs/decisions.md` cite, verbatim:

- p. 3: "The computation rule. The concept of fairness [19] was recognized
  in Absys, and Absys used a fair computation rule."
- p. 3: "Delay mechanisms. Both primitive and user declared delay
  constructs were used in Absys …"
- p. 3: "Aggregation operators. A set-of aggregation operator was
  implemented in Absys-4 (circa 1968)."
- p. 6: "although Absys normally processes 'unification equations' as they
  arise, occasionally completion of this processing is, for good reasons,
  deferred."
- p. 9: "rather than elaborating all goal atoms before beginning equation
  solving …, Absys interleaves equation solving and procedure application.
  Further, the ongoing equation solving establishes bindings which are
  immediately used."
- p. 9, note 9: "Absys does not guarantee a particular order of processing
  of conjuncts."
- p. 9: "The or is distributed through the goal conjunct in the computation
  in which the application takes place, essentially creating two new goals
  which can notionally be regarded as initiating parallel derivations."
- p. 10: "the only functor in Absys is the list constructor … A noteworthy
  difference is that the implementation of Absys used a 'fair' [19] search
  rule, in that all derivations continue to be incrementally elaborated
  until found unsatisfiable."
- pp. 10-11: the treatment of equality, quoted in decisions 9 and 10.
- p. 11, note 13: "'Hibernate' is a better word, since the computational
  process associated with the derivation can be reactivated."
- p. 11, note 14: "Absys was an interactive, incremental system, and was
  designed with this in mind."
- p. 14: "The operator not distributes with respect to or and the implicit
  and in the expected way. The operator acts like a degenerate or in that
  during goal evaluation it initiates an independent computational branch,
  but one in which the satisfiability criteria for termination are
  reversed."
- p. 15, note 18: "As implementations both Absys and PROLOG have the
  shortcoming that they do not use a safe computation rule, and so soundness
  and completeness are not guaranteed in either."

What the paper does not say:

- how `or` was implemented (copying, or backtracking with interleaving);
- in what order a derivation's literals were selected;
- when `tout` writes;
- whether `set` sorts its answers;
- what the "primitive and user declared delay constructs" were, beyond the
  suspension of equations.

Each of these is a decision in `docs/decisions.md`.
