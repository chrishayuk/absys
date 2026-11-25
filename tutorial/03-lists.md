# 3. Lists

Lists are written `[]`, `[1, 2, 3]`, and `[H|T]` for a list with head H and
tail T. An equation between lists equates them member by member, so the same
equation can build a list or take one apart. It depends only on what is
known when the equation is taken up. Foster and Elcock (1969) make this
point about `z = [x&y]` in the 1969 notation (chapter 8):

> Whether it acts to construct z, or to select x and y, or to test whether x,
> y and z satisfy the asserted relation, depends solely on the data in the
> system at the time this assertion is processed.

Here it selects:

```absys
Z = [1, 2, 3]
Z = [H|T]
```

```output
1 derivation:
  derivation #0 (hibernating)
    Z = [1,2,3]
    H = 1
    T = [2,3]
```

Here it constructs:

```absys
Z = [H|T]
H = 1
T = [2, 3]
```

```output
1 derivation:
  derivation #0 (hibernating)
    Z = [1,2,3]
    H = 1
    T = [2,3]
```

Both texts end with the same data, although the assertions came in a
different order.

When two lists have unknowns in the same places, the equation between them
leaves suspended equations behind:

```absys
[A, 2] = [B, C]
```

```output
1 derivation:
  derivation #0 (hibernating)
    A = B   (suspended: A = B)
    B = A   (suspended: A = B)
    C = 2
    suspended   A = B
```
