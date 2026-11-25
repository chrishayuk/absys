# 10. Reference

## The 1990 notation (`.abs`)

```text
text        := assertion*                     (one after another; ; may separate)
assertion   := conj ('or' conj)*
conj        := unit (',' unit)*
unit        := term '=' term
             | term '(' terms ')'             an application; ( on the same line
             | '{' assertion '}'  |  '{' '}'
             | 'not' '(' assertion ')'  |  'not' unit
             | '{' 'set' '(' term ')' ',' unit ',' term '}'
             | 'tout' '(' term ',' term ')'
term        := Variable | name | number | 'word' | "word"
             | '[' ']' | '[' term (',' term)* ('|' term)? ']'
             | 'lambda' Variable (',' Variable)* assertion
```

- A `Variable` begins with a capital, or with `_`.
- A `name` begins with a small letter, and may have hyphens between letters
  (`math-course`).
- `%` begins a note, which runs to the end of the line.

## The 1969 notation (`.ab1`)

```text
text        := assertion*
assertion   := term '=' term | name '(' terms ')'
             | 'begin' assertion* 'end'  |  '{' assertion* ('or' assertion*)* '}'
term        := sum;  sum := product (('+'|'-') product)*;  product := primary (('*'|'/') primary)*
primary     := name | name'  | number | 'word' | name '(' terms ')'    (a function's result)
             | '[' ']' | '[' term (';' term)* ('&' term)? ']' | '(' term ')'
             | 'lambda' name (',' name)* ('=>' name)? ('key' name (',' name)*)? (begin … end | { … })
```

`null(p)` is `p = []`.

## The machine

`absys decisions` lists every rule, with the page of the sources it rests
on, or the reason for the decision where they are silent. The same list is in
[`docs/decisions.md`](../docs/decisions.md).

## Options that are not Absys

These exist so that the rules beside them can be studied
([`docs/research.md`](../docs/research.md)):

| option | instead of |
|---|---|
| `--search depth-first` | fair search: elaborate the first derivation until it fails or hibernates |
| `--select fifo` | the body of an application in the literal's place: every literal joins the end of the goal |
| `--negation delayed` | `not` at once: `not` waits until its variables are instantiated |
