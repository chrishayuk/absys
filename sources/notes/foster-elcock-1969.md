# Foster and Elcock 1969: what was read, and how

The text was not seen as a scan. It was read only as the OCR extracts quoted
on HOPL (exp=357), which are noisy. Where the OCR is broken, the reading
used is given beside it, and how it was decided. The extracts stop after the
section "Recursive Functions and Keys". The last pages of the paper (about
pp. 428-429) have not been seen, and may say more about negation, sets or
ordering.

| OCR | read as | why |
|---|---|---|
| `a «■ h -I-1, a<* ] + /;, I + b?a, h + I «■ a` | `a = b + 1, a = 1 + b, 1 + b = a, b + 1 = a` | four arrangements of one equation, which "are equivalent" |
| `x=y   y = 2` | as is | clean |
| `y=x, x?2 and 3=y` | `y=x, x=2 and 3=y` | the text says no data can satisfy them |
| `z=[x8cy]` | `z=[x&y]` | "a list whose head is x and whose tail is y"; `8c` is a common OCR reading of `&` |
| `zl = [ai[b;c]] z2=[[a;6];[c;c]]` | not used | too damaged to read with confidence |
| `/'=lambdax, j=>z begin x = [z &q']y = [z &/¡ö'] end` | `f'=lambda x,y=>z begin x=[z&q'] y=[z&r'] end` | "f(m,n) = b is equivalent to … begin m=[b&q'] n=[b&r'] end" |
| `/>=[1;2] {p = [ct;h] orp = [b;a]}` | `p=[1;2] {p = [a;b] or p = [b;a]}` | "In one set a and b are the data 1 and 2 respectively whilst in the other … 2 and 1" |
| `a=b+\` | `a=b+1` | "one of the conjunctions will be unsatisfiable" |
| `map'=lambda p, /key p begin {null(p) orp = [r' 8is']f{r) map(s,f)} end` | `map'=lambda p, f key p begin {null(p) or p = [r'&s'] f(r) map(s,f)} end` | the first version, printed cleanly above it, has `p,f` |
| `item'=lambda x=>z key x begin x = [p' &q'] {z=p or z= item (q)} end` | as is | clean apart from spacing |
| `i=item ([I ;2;3])` … `{/=1 or i=2 or i=3}` | `i=item([1;2;3])` … `{i=1 or i=2 or i=3}` | |
| `r'il/eni(pl)   s ? item(p2)   t = itcni(p?>)   r h.V"/` | `r'=item(p1)   s'=item(p2)   t'=item(p3)   r+s=t` | the problem statement before it: "the triples r, s, t … satisfy the relation r+s=t"; whether s and t were primed cannot be read |
| `[l;2;5;61, \|3;7;l\|, 11();2;7;4J` | `[1;2;5;6], [3;7;1], [10;2;7;4]` | with these lists, exactly the three triples the paper states satisfy r+s=t, in the order it states them |

The partial-application example ("Functions may be introduced by partial
application of other functions") lost the line that applies `f`. It is not
reconstructed.
