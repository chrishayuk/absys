# 8. The 1969 notation: Absys 1

The retrospective modernises the syntax. Foster and Elcock's 1969 paper on
Absys 1 shows the original, as far as the surviving extracts of it go. Files
ending `.ab1` are read in it. Here are the differences:

| 1990 (`.abs`) | 1969 (`.ab1`) |
|---|---|
| `{A, B}` | `begin A B end`: assertions side by side are a conjunction |
| `{A} or {B}` | `{A or B}` |
| `[1, 2, 3]`, `[H\|T]` | `[1;2;3]`, `[h&t]` |
| capitals are local | a prime introduces a new name: `r'` |
| no arithmetic | `a = b + 1`, in any direction |
| — | functions with a result: `lambda x => z` |
| — | `key`: wait until an argument is present |

The 1969 paper's first example:

```absys 1969
x=y   y = 2
```

```output
1 derivation:
  derivation #0 (hibernating)
    x = 2
    y = 2
```

## Arithmetic is an assertion

"The assertions a = b + 1, a = 1 + b, 1 + b = a, b + 1 = a are equivalent"
(Foster and Elcock 1969). Each of them works whichever of a and b is known.
`a + b = c` is taken up when two of the three are numbers, and waits
otherwise. Foster (1968) calls these "pass sets":

```absys 1969 --each
a = b + 1
a = 5
```

```output
» a = b + 1
  1 derivation:
    derivation #0 (hibernating)
      b  uninstantiated
      a  uninstantiated
      suspended   b + 1 = a  (waits for two of its numbers: b, a)

» a = 5
  1 derivation:
    derivation #0 (hibernating)
      b = 4
      a = 5
```

## Functions, and key

`item` gives each member of a list, one derivation for each:

```absys 1969
item'=lambda x=>z key x
    begin x = [p'&q'] {z=p or z=item(q)} end
i=item([1;2;3])
```

```output
3 derivations:
  derivation #1 (hibernating)
    procedures  item
    i = 1
  derivation #3 (hibernating)
    procedures  item
    i = 2
  derivation #5 (hibernating)
    procedures  item
    i = 3
```

- `=> z` makes `z` the result, so `i=item(q)` is the application
  `item(q, i)`.
- `key x` says that x must be present before an application is elaborated.

Foster and Elcock explain why `key` is needed. Without it, an application of
`map` or `item` to a list not yet known would elaborate its alternatives at
once, making lists of every length, and never stop.

With `item`, the paper's triples problem asks for an r from p1, an s from p2
and a t from p3 such that r+s=t. The result "would be three distinct
conjunctions of assertions with associated data r, s, t 1, 3, 4, 1, 1, 2 and
6, 1, 7 respectively":

```absys 1969
item'=lambda x=>z key x
    begin x = [p'&q'] {z=p or z=item(q)} end
p1 = [1;2;5;6]   p2 = [3;7;1]   p3 = [10;2;7;4]
r'=item(p1)   s'=item(p2)   t'=item(p3)   r+s=t
```

```output
3 derivations:
  derivation #115 (hibernating)
    procedures  item
    p1 = [1;2;5;6]
    p2 = [3;7;1]
    p3 = [10;2;7;4]
    r = 1
    s = 3
    t = 4
  derivation #71 (hibernating)
    procedures  item
    p1 = [1;2;5;6]
    p2 = [3;7;1]
    p3 = [10;2;7;4]
    r = 1
    s = 1
    t = 2
  derivation #113 (hibernating)
    procedures  item
    p1 = [1;2;5;6]
    p2 = [3;7;1]
    p3 = [10;2;7;4]
    r = 6
    s = 1
    t = 7
```

These are the paper's three, in the paper's order.

## The 1982 notation

In 1982 Elcock wrote Absys a third way, for a short paper comparing it with
Prolog. It is the 1969 notation with:

- `<< … >>` for `{ … }`;
- `and` written between assertions;
- lists written `[a,b]` and `[h|t]`;
- no primes. A name a lambda's body uses but never applies is local to it.

Files ending `.ab2` are read in it. Its example walks through Absys's list
of relations still to be elaborated, step by step. `--trace` shows the same
steps:

```absys 1982 --trace
u+v = 16  and  w*u = v  and  u+10 = 12
```

```output
» u+v = 16  and  w*u = v  and  u+10 = 12
  #0    suspend   u + v = 16: waits for two of its numbers
  #0    suspend   w * u = v: waits for two of its numbers
  #0    equation  u + 10 = 12: gives 2
  #0    equation  u = 2: u/2; reactivated 2 + v = 16, w * 2 = v
  #0    equation  2 + v = 16: gives 14
  #0    equation  v = 14: v/14; reactivated 2 + 14 = 16, w * 2 = 14
  #0    equation  2 + 14 = 16: gives 16
  #0    equation  16 = 16: holds
  #0    equation  w * 2 = 14: gives 7
  #0    equation  w = 7: w/7; reactivated 7 * 2 = 14
  #0    equation  7 * 2 = 14: gives 14
  #0    equation  14 = 14: holds
  #0    equation  7 * 2 = 14: gives 14
  #0    equation  14 = 14: holds
  #0    hibernate nothing more can be done: the derivation hibernates
1 derivation:
  derivation #0 (hibernating)
    u = 2
    v = 14
    w = 7
```

The two relations that cannot be elaborated are associated with their
unknowns. When `u+10 = 12` binds u, both come back. When `2+v = 16` binds
v, both come back again, so `w*2 = 14` is examined twice and `2+14 = 16` is
checked after it has already run. That is what Elcock's lists show
(`docs/research.md`, E06).
