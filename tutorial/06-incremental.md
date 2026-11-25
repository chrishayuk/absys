# 6. Hibernation, and assertions added later

Absys "was an interactive, incremental system, and was designed with this in
mind" (Elcock 1990, note 14). A derivation that has done all it can
*hibernates*. When another assertion is typed, it is added to every
derivation still alive, and they wake.

The retrospective's example (pp. 11-13) is a reverse procedure:

```absys --each --env
rev1 = lambda R, S, T
    {R = [], S = T} or
    {R = [X|L], rev1(L, [X|S], T)}
rev = lambda R, S
    {rev1(R, [], S)}
rev([1,2,U], [V|L])
rev([5], [V])
```

```output
» rev1 = lambda R, S, T
      {R = [], S = T} or
      {R = [X|L], rev1(L, [X|S], T)}
  1 derivation:
    derivation #0 (hibernating)
      procedures  rev1
      environment 

» rev = lambda R, S
      {rev1(R, [], S)}
  1 derivation:
    derivation #0 (hibernating)
      procedures  rev1, rev
      environment 

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

After the goal, the derivation hibernates. Its binding environment is the
one the retrospective prints, "X₁/1, L₁/[2,U], X₂/2, L₂/[U], L₃/[],
L/[2,1]", and so are its suspended equations, U = X₃ and X₃ = V. Elcock: "If
we query the computed answer substitution for U, we will be told that U = X₃,
whereupon querying X₃ tells us that X₃ = V, i.e. that the computed answer
substitution for U is constrained to be U/V".

Then `rev([5], [V])` is added. V becomes 5, which wakes X₃ = V, which wakes
U = X₃. The environment is again the retrospective's, down to the names X₄
and L₄ of the variables the new assertion made.

## At the keyboard

`absys repl` is the incremental system at a terminal. Type assertions, and
see what each one does:

```text
$ absys repl
> rev1 = lambda R, S, T
…  {R = [], S = T} or
…  {R = [X|L], rev1(L, [X|S], T)}
> rev = lambda R, S {rev1(R, [], S)}
> rev([1,2,U], [V|L])
  derivation #7 (hibernating)
    U = V   (suspended: U = X₃, X₃ = V)
    …
> ? U
#7: U = V   (suspended: U = X₃, X₃ = V)
> rev([5], [V])
    U = 5
    …
```

`?` shows every derivation, `? X` the value of X in each, `.env` the binding
environments, and `.trace` every step from then on.
