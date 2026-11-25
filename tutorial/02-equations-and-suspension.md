# 2. Equations, and equations that wait

Elcock's retrospective (1990, pp. 10-11) describes equality as a *process*
that takes two terms:

- **Two lists**: their heads are equated, and their tails.
- **The same constant, or the same variable**: it holds.
- **A list and a constant, or two different constants**: it fails, and so
  does the derivation it is in.
- **An unknown variable and a term**: the variable is instantiated to the
  term. Anything that was waiting on the variable is taken up again.
- **Two different unknown variables**: the equation is **suspended**, and
  waits on both.

The last case is what makes Absys different from Prolog. A Prolog system
unifies two unbound variables at once, making them one variable. Absys keeps
the equation, and waits.

```absys --each
X = Y
Y = Z
Z = 42
A = [X, Y, Z]
```

```output
» X = Y
  1 derivation:
    derivation #0 (hibernating)
      X = Y   (suspended: X = Y)
      Y = X   (suspended: X = Y)
      suspended   X = Y

» Y = Z
  1 derivation:
    derivation #0 (hibernating)
      X = Y = Z   (suspended: X = Y, Y = Z)
      Y = X = Z   (suspended: X = Y, Y = Z)
      Z = Y = X   (suspended: X = Y, Y = Z)
      suspended   X = Y;  Y = Z

» Z = 42
  1 derivation:
    derivation #0 (hibernating)
      X = 42
      Y = 42
      Z = 42

» A = [X, Y, Z]
  1 derivation:
    derivation #0 (hibernating)
      X = 42
      Y = 42
      Z = 42
      A = [42,42,42]
```

`--each` shows the state after every assertion.

- After `Y = Z`, two equations are suspended. Asked for `X`, the system can
  only say that X must equal Y and Z.
- `Z = 42` instantiates Z. That takes up `Y = Z` again, which instantiates
  Y. That in turn takes up `X = Y`.

## Constants

The only things in Absys are lists and constants (Elcock 1990, p. 10). A
constant is one of these:

- a number, such as `42`;
- a quoted word, as in the retrospective's `'BROWN'`;
- a label, such as the `"U"` of the output assertion.

A constant equals only itself:

```absys
X = 'BROWN'
X = 'SMITH'
```

```output
No derivation survives: the assertions are unsatisfiable.
```

There are no other data structures. Elcock suggests writing a term
`f(t1, …, tn)` as the list `[f, t1, …, tn]`.

## No occur check

Absys, like the Prolog of its day, does not check whether a variable occurs
in the term it is being equated with (p. 11). `X = [1|X]` makes a list that
contains itself. The reconstruction allows it too, but does not try to print
one.
