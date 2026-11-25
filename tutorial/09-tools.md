# 9. Watching a computation

## The command line

| | |
|---|---|
| `absys FILE` | every derivation left at the end |
| `absys FILE --each` | the state after each assertion |
| `absys FILE --env` | each derivation's whole binding environment, as the retrospective writes it |
| `absys FILE --trace` | every step: each equation, suspension, application, distribution of an `or`, failure, hibernation |
| `absys repl [FILE]` | the incremental system, at the keyboard |
| `absys permute FILE` | accepts the assertions in every order, and says which orders give which answers |
| `absys graph FILE` | the derivation tree, and what is suspended on what, for Graphviz |
| `absys decisions` | the rules of the machine, and what each rests on |

A trace of the append refutation:

```absys --trace
app = lambda R, S, T
    {R = [], S = T} or
    {R = [X|L], T = [X|N], app(L, S, N)}
app([1], U, [1,2])
```

```output
» app = lambda R, S, T
      {R = [], S = T} or
      {R = [X|L], T = [X|N], app(L, S, N)}
  #0    equation  app = lambda R, S, T: app/lambda R, S, T
  #0    hibernate nothing more can be done: the derivation hibernates
» app([1], U, [1,2])
  #0    apply     app([1], U, [1,2]): applied
  #0    or        or distributes: 2 alternatives, derivations #1, #2
  #1    fail      [1] = []: [1] cannot equal []
  #2    equation  [1] = [X₁|L₁]: X₁/1, L₁/[]
  #2    equation  [1,2] = [1|N₁]: N₁/[2]
  #2    apply     app([], U, [2]): applied
  #2    or        or distributes: 2 alternatives, derivations #3, #4
  #3    equation  [] = []: holds
  #4    fail      [] = [X₂|L₂]: [] cannot equal [X₂|L₂]
  #3    equation  U = [2]: U/[2]
  #3    hibernate nothing more can be done: the derivation hibernates
1 derivation:
  derivation #3 (hibernating)
    procedures  app
    U = [2]
```

Compare Elcock's refutation (1990, pp. 9-10): "The first two equations give
rise to the bindings X₁/1, L₁/[], and the third equation now gives (rule 1)
the new goal 1 = 1, [2] = N₁, app(L₁, U, N₁)".

## Output

`tout("U", U)` writes `U = …` when U is complete (Elcock 1990, note 11):

```absys
app = lambda R, S, T
    {R = [], S = T} or
    {R = [X|L], T = [X|N], app(L, S, N)}
tout("U", U)
app([1], U, [1,2])
```

```output
1 derivation:
  derivation #3 (hibernating)
    output      U = [2]
    procedures  app
    U = [2]
```

## The playground and editors

The [playground](../web/index.html) shows the same things in a browser:

- the text, with the assertion being looked at marked;
- the list of assertions, which you can step through;
- each derivation, with its answers, its suspended equations and what is
  still to do;
- the tree of derivations the `or`s made;
- every step.

`absys lsp` is a language server: problems as you type, hover with the state
after each assertion, and colours. The Visual Studio Code extension is in
`editors/vscode`. See `editors/README.md`.
