# 4. Procedures, in every direction

A procedure is a lambda expression, and defining one is an equation:

```text
app = lambda R, S, T
    {R = [], S = T} or
    {R = [X|L], T = [X|N], app(L, S, N)}
```

This is the retrospective's append (p. 8).

- `R`, `S` and `T` are bound by the lambda.
- `X`, `L` and `N` are local to the second alternative. They are made new
  each time that alternative is chosen.
- `app` is an identifier like any other. The equation instantiates it to
  the lambda expression.

In the 1990 notation, a variable written with a capital is local to the
lambda it is in, unless the lambda binds it. A name written with a small
letter, such as `app` or `math-course`, belongs to the whole text.

An application, `app([1], U, [1,2])`, asserts that the relation holds of
those three things. Elcock (1990, p. 9) describes what happens next. The
application is replaced by the body, with the arguments in place of R, S
and T:

```text
{[1] = [], U = [1,2]} or {[1] = [X₁|L₁], [1,2] = [X₁|N₁], app(L₁, U, N₁)}
```

The first alternative fails at once. In the second, X₁, L₁ and N₁ are
instantiated, and `app` is applied again.

```absys --env
app = lambda R, S, T
    {R = [], S = T} or
    {R = [X|L], T = [X|N], app(L, S, N)}
app([1], U, [1,2])
```

```output
1 derivation:
  derivation #3 (hibernating)
    procedures  app
    U = [2]
    environment U/[2], X₁/1, L₁/[], N₁/[2]
```

The binding environment is the retrospective's own: "X₁/1, L₁/[], N₁/[2],
U/[2]" (p. 10).

## No designated result

`app` has no inputs and outputs. All three of its arguments take part in the
relation, so the one definition answers questions in several directions:

```absys
app = lambda R, S, T
    {R = [], S = T} or
    {R = [X|L], T = [X|N], app(L, S, N)}
app(R, S, [1, 2])
```

```output
3 derivations:
  derivation #1 (hibernating)
    procedures  app
    R = []
    S = [1,2]
  derivation #3 (hibernating)
    procedures  app
    R = [1]
    S = [2]
  derivation #5 (hibernating)
    procedures  app
    R = [1,2]
    S = []
```

Three derivations answer: three ways of splitting `[1,2]`. Chapter 5 is about
derivations.

## A procedure is not a function

There is no `T = app(R, S)`. What a procedure relates is among its
arguments:

```absys
Y = app([1], [2])
```

```output
1:8: an application is an assertion, not a value: what a procedure relates is among its arguments (`rev(X, Y)`, not `Y = rev(X)`)
```

The 1969 notation did have functions with results (chapter 8).

## A procedure can be applied before it is defined

`app` is a variable. An application of a variable that has no value yet
waits, like a suspended equation, until the variable is instantiated:

```absys --each
app([1], [2], T)
app = lambda R, S, T
    {R = [], S = T} or
    {R = [X|L], T = [X|N], app(L, S, N)}
```

```output
» app([1], [2], T)
  1 derivation:
    derivation #0 (hibernating)
      app  uninstantiated
      T  uninstantiated
      suspended   app([1], [2], T)  (waits for a definition: app)

» app = lambda R, S, T
      {R = [], S = T} or
      {R = [X|L], T = [X|N], app(L, S, N)}
  1 derivation:
    derivation #3 (hibernating)
      procedures  app
      T = [1,2]
```
