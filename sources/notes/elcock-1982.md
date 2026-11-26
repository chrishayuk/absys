# Elcock 1982: what was read, and how

E. W. Elcock, "Goal selection strategies in Horn clause programming",
*Proceedings of the Fourth Biennial Conference of the Canadian Society for
Computational Studies of Intelligence* (CSCSI/SCEIO), University of
Saskatchewan, Saskatoon, 17-19 May 1982, pp. 121-124.

The proceedings are a 250-page scan, put online by CAIAC:
https://www.caiac.ca/sites/default/files/shared/canai-archives/1982_CSCSI_Proceedings-opt.pdf
The paper is on PDF pages 129-132.

The scan's text layer is OCR, and it is bad ("an auertat ive prograrmdng
system"). Everything below was read from the page images at 130 dpi. The
copy is in `sources/downloads/elcock-1982/`, which git ignores. This source
was pointed out after the first version of the reconstruction was made.
Earlier work did not have it.

## What the paper is

It is a short paper comparing the "incompleteness" of Prolog's evaluation
with that of Absys. Its account of Absys "draws heavily on Foster and Elcock
(1969)". It is the only source seen that walks through Absys's list of
relations step by step. For the mechanism "in detail" it points to Elcock,
McGregor and Murray, "Data directed control and operating systems",
*British Computer Journal* 15(2) (1972) 125-129, not yet read here.

## Passages used, verbatim

p. 121:

> Absys (standing for Aberdeen System) was an experimental working on-line
> incremental compiler for assertions, developed by the Computer Research
> Group at the University of Aberdeen and essentially completed in 1968.

E90 says "essentially completed in 1967" (p. 2).

p. 122, the notation:

```text
L = [X|M]

al << a2 or a3 >> a4      is equivalent to      << al a2 a4 >> or << al a3 a4 >> .

mem = lambda m,s key  s
      << s = [p|sl] and
         << m = p or mem(m,sl) >> >>
```

`or` and `and` are underlined in the print, as reserved words. The text
around them reads:

- "The assertion `mem(x,[1,2,3])` is equivalent to asserting `<< x=1 or x=2
  or x=3 >>` . If in addition we were now to assert `mem(x,[2,4,6])`
  equivalent to `<< x=2 or x=4 or x=6 >>` , then distribution would lead to
  nine computational branches of which only one, that associated with x=2,
  would be satisfiable and hence remain active."

p. 122, the implementation of alternatives:

> The system attempts to construct data to satisfy each conjunction of
> assertions, each conjunction notionally constituting a separate (parallel)
> computation branch of the total program. In practice, of course, the
> non-determinism was handled by appropriate differential record keeping and
> backtracking in a similar spirit to Prolog implementations. The
> distribution of and and or connectives was handled in a way which
> attempted to minimize duplication of processing. A particular
> computational branch terminates when unsatisfiability is detected.

E90 says instead that "the implementation of Absys used a 'fair' search
rule, in that all derivations continue to be incrementally elaborated until
found unsatisfiable" (p. 10). See `docs/decisions.md`, rule 14.

p. 122, `key`:

> The "key" statement in effect says "don't elaborate this call of mem
> unless the actual parameter which is to be bound to the indicated formal
> parameter "s" of mem has a value".

p. 122, `set` and `not`:

> Anticipating Prolog, Absys had a primitive aggregation operator set: It
> takes as parameters a prototype set element and an assertion and produces
> the set of prototype elements satisfying the assertion. The assertion is
> typically a disjunction. The set aggregator initiates the "parallel"
> computations and then extracts the datum corresponding to the prototype
> from those computations which do not terminate because of
> unsatisfiability.

> Absys negation acts like a degenerate or in that it initiates an
> independent computational branch, but one in which the criteria for
> termination are reversed in that not (<<a>>) is satisfiable iff <<a>> is
> unsatisfiable.

p. 123, the evaluation mechanism:

> Absys maintains a list of relations still to be elaborated. Suppose we
> activate the first such relation f(x,y,z) , say, on this list. It is
> expected that the functor f , whether primitive or user defined, has been
> defined in such a way that the header for f specifies, by means of its
> "key" statement, a constraint on its argument set which indicates whether
> or not it is "worthwhile" elaborating the body of f (e.g. "plus(x,y,z)" is
> only worthwhile elaborating if two of its arguments already have values in
> the domain of "plus"). If the relation is not deemed worth elaborating,
> then it is "associated" with each of its arguments which are currently
> uninstantiated variables. The evaluator now continues processing the list
> of relations awaiting elaboration. Suppose, on the other hand, that the
> relation f(x,y,z) is worth processing in the sense that enough is known
> about some of the arguments to allow others to be inferred through
> elaboration of the body of the function. The body is now elaborated and the
> binding environment necessarily augmented. The process of changing a
> variable binding in the binding environment automatically returns any
> relations associated with that variable to the list of relations still to
> be elaborated.

p. 123, the example, which is ABSYS-E06:

> Suppose we have the Absys program
>
>     u+v = 16  and  w*u = v  and  u+10 = 12
>
> and suppose the list of relations is initially processed in this order.
> The relation u+v = 16 is examined and, for the reasons mentioned,
> associated with u and v . The relation w*u = v is now examined and
> associated with w,u and v . We now have the association lists (u: u+v =
> 16; w*u = v) ; (v: u+v = 16; w*u = v) and (w: w*u = v). The relation
> u+10 = 12 is now examined, elaborated and the binding environment
> augmented so that u is bound to 2 (12-10). As a result of this binding to
> u , the list of relations (2+v = 16; w*2 = v) associated with u is
> appended to the list of relations to be elaborated. The relation 2+v = 16
> is now examined and elaborated and the binding environment augmented so
> that v is bound to 14 , and the list associated with v appended to the
> list of relations to be elaborated. This list is now (2+14 = 16; w*2 = 14;
> w*2 = 14). The first relation is examined, elaborated and found
> satisfied. The second is examined, elaborated and the binding environment
> augmented so that w is bound to 7 , and the list associated with w
> appended to the list of relations to be processed. This list is now (7*2 =
> 14; 7*2 = 14). Both these remaining relations are examined, elaborated and
> found satisfied. The list of relations to be elaborated is now empty and
> the (examinable) state of the binding environment reflects what the
> system has been able to infer from the original conjunction of
> assertions.

> Indeed, the action of the Absys evaluator could be viewed as dynamically
> rearranging the order of elaboration of the relations under the influence
> of the changing binding environment.

p. 124, on its limits:

> However, the Absys dynamic data directed elaboration of the conjunction is
> gained only at a high price of more elaborate runtime processing
> structures. In any case, a naive dynamic data directed flow of
> elaboration, although elegant in certain well circumscribed contexts, of
> itself rapidly runs out of steam. For example, and staying within
> arithmetic for pedagogic simplicity, much more sophisticated aggregation
> and solution methods would be necessary to deal with a conjunction of a
> general set of linear equations in n variables. Even the task of
> recognizing what aggregations of individual conjuncts might lend
> themselves to reorganization as a specified "higher" relation (e.g. the
> relation "simult-eqs(L)" where L is a list of lists of coefficients say)
> is challenging to say the least.

The summary on p. 124 says "a comparison of the elaboration strategies of
Abset and Prolog". This is presumably a slip for Absys.

## What the example shows, read closely

1. **A relation stays associated with every variable it was associated
   with.** When u is bound, both relations come back. When v is then bound,
   both come back *again*: `u+v=16` (just elaborated, as 2+v=16) and `w*u=v`
   (still on the list). So the list holds w*2=14 twice. Each relation
   comes back once for each of its variables that is bound, and runs again,
   to be "found satisfied". The first version of this reconstruction
   returned a suspended relation only once, which cannot give these lists.
   It now follows the paper for primitive relations (`docs/decisions.md`, rule 10).

2. **"Appended" does not give the list he prints.** After 2+v=16 binds v,
   the list holds one relation, w*2=14. Appending v's two relations to the
   end gives (w*2=14; 2+14=16; w*2=14). Elcock prints (2+14=16; w*2=14;
   w*2=14), and says the first is "found satisfied" and the second binds w.
   Taken literally, the first would bind w instead. Every list he prints
   comes out exactly if the returned relations are put at the head of the
   list. The binding environment at the end is the same either way:
   u = 2, v = 14, w = 7. The reconstruction puts returned relations at the
   head (`--wake front`, the default). `--wake back` appends them, as the
   word says (`docs/decisions.md`, rule 10).
