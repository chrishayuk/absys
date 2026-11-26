# What the reconstruction does, and what each rule rests on

Every rule of the machine is listed here with its evidence. Three kinds of
statement are kept apart:

- **Source**: what a primary source says. The page is given. The sources
  are listed in `sources/README.md`.
- **Decision**: what the reconstruction does where the sources are silent,
  ambiguous or in conflict. The reason is given.
- **Experiment**: an option that is not historical. It exists only so the
  rule beside it can be studied.

The abbreviations are:

- **E90**: Elcock, "Absys: the first logic programming language: a
  retrospective and a commentary", *J. Logic Programming* 9 (1990) 1-17.
- **FE69**: Foster and Elcock, "Absys 1: an incremental compiler for
  assertions; an introduction", *Machine Intelligence 4* (1969). Only OCR
  extracts of most of its sections have been seen, not its last pages.
- **F68**: Foster, "Assertions: programs written without specifying
  unnecessary order", *Machine Intelligence 3* (1968).
- **E82**: Elcock, "Goal selection strategies in Horn clause programming",
  *Proc. 4th CSCSI/SCEIO Conference* (1982) 121-124, read from the page
  scans (`sources/notes/elcock-1982.md`). It was found after the first
  version of this reconstruction, and the rules it changed say so.

## Three notations, one machine

1. **The 1990 notation** (`.abs`) is the one E90 uses. E90 says it takes
   "some slight liberties ... with the original syntax" (p. 3), and calls
   what it describes "a subset of the language Absys-4, circa 1968". The
   source is E90 pp. 3, 8, 11, 15.
   - Procedures are written `name = lambda R, S, T` and a body.
   - `{…}` is a conjunction, with its members separated by commas.
   - `or` gives alternatives.
   - Lists are written `[]`, `[1,2]` and `[X|L]`.
   - Constants are quoted: `'BROWN'`.
   - `not(…)`, `{set(X), {…}, S}` and `tout("U", U)` are available.

2. **The 1969 notation** (`.ab1`) is the notation of Absys 1, as far as the
   FE69 extracts show it. The source is FE69.
   - Assertions are written side by side, with no separator.
   - `begin … end` groups assertions.
   - `{A1 or A2}` gives alternatives.
   - Lists are written `[a;b;c]`, and `[h&t]` for a head and a tail.
   - `x'` introduces a new name.
   - `lambda x, y => z key x begin … end` defines a function with a result
     and a key.
   - `null(p)` and arithmetic `a = b + 1` are available.

   The 1982 notation (`.ab2`) is the one E82 uses for its examples: the
   1969 notation, with `<< … >>` for `{ … }`, `and` written between
   assertions, lists written `[a,b]` and `[h|t]`, and no primes. The source
   is E82 pp. 122-123.

3. *Decision.* **All three notations read into the same tree and run on the
   same machine.** E90 presents its Absys as the language of FE69 with a
   different surface, and E82 draws "heavily on Foster and Elcock (1969)".
   Nothing in the sources suggests that they evaluate differently.

## Terms

4. *Source.* **The only functor is the list constructor**, so the Herbrand
   universe is lists and constants (E90 p. 10). There are no `f(t1, …, tn)`
   terms. E90 suggests writing `[f, t1, …, tn]` instead.

5. *Decision.* **Constants are numbers and quoted words.** In the 1990
   notation `'BROWN'` and `"U"` are both constants. They are equal only to
   themselves, and the two kinds of quote are never equal to each other.
   - E90 p. 15 quotes the student data as `'BROWN'`.
   - E90 note 11 writes the label of `tout` as `"U"`.
   - FE69 and F68 quote constants with `'`, as in `'='` and `'+'`.

6. *Source.* **An identifier is a variable**, and `app = lambda …` is an
   equation that instantiates `app` (E90 p. 8: "The syntax of procedure
   definitions is borrowed from the lambda calculus"). E90 p. 10 says "When
   created, an identifier is associated with ... an uninstantiated
   variable".
   - *Decision.* An application of an identifier that has no value yet
     waits until it has one. E90 says an uninstantiated identifier holds "a
     list of references to suspended processes which ... require the
     identified variable to be instantiated for completion" (p. 10). So a
     procedure can be applied before it is defined.
   - *Decision.* In the 1990 notation:
     - Inside a lambda, a variable written with a capital is *local* when
       it is not bound by the lambda.
     - An identifier written with a small letter is a name the whole text
       shares, such as `app` or `math-course`.
     - Outside any lambda, every identifier is shared by the whole text,
       which is what lets `rev([5],[V])` refer to the `V` of an earlier
       assertion (E90 p. 13).
   - *Source.* In the 1969 notation, the names introduced with a prime are
     local (FE69: "The primes in the above serve to introduce new names").
   - *Decision.* A prime's scope is the whole lambda, not the `begin … end`
     around it. Each name is primed only once in every example.
   - *Decision.* The 1982 notation has no primes. In its `mem`, `p` and
     `sl` are local and `mem` is not. Here, inside a lambda, a name that the
     body uses but never applies, and that the lambda does not bind, is
     local; a name it applies belongs to the whole text.

7. *Source.* **The local variables of an alternative are created when it is
   chosen.** E90 p. 8 calls them existentially quantified.
   - *Decision.* A local variable belongs to the innermost alternative (or
     body of `not` or `set`) that contains every occurrence of it.
   - *Decision.* The variables a derivation creates are numbered with
     subscripts, `X₁, L₁, N₁`, which number increasing each time a
     derivation enters a part of a body that has local variables.
   - Together these reproduce E90's own numbering, including `X₄` and `L₄`
     for the variables of the later assertion `rev([5],[V])` (p. 13).

8. *Source.* **There is no occur check** (E90 p. 11).

## Equality, and suspension

9. *Source.* **Equality is a process taking two references** (E90 pp. 10-11):
   - Two lists: equate their heads and their tails.
   - The same constant, or the same variable: succeed.
   - A list and a constant, or two different constants: fail, which ends
     the derivation.
   - An uninstantiated variable and a term: instantiate the variable, and
     add "any suspended equality processes associated with x ... to the
     goal of the current computation".

10. *Source.* **An equation between two different uninstantiated variables
    is suspended** "and associated with both x and y" (E90 p. 11). It is
    processed again when either variable is instantiated.
    - *Source.* **A primitive relation stays associated with every variable
      it was associated with**, and comes back each time one of them is
      bound, even after it has been elaborated. E82 p. 123 says binding "a
      variable ... automatically returns any relations associated with that
      variable to the list of relations still to be elaborated", and its
      example shows w*2 = 14 on that list twice, and 2+14 = 16 "found
      satisfied" after it was already elaborated. Equations and arithmetic
      behave this way here. Anything else that waits (an application, a
      `not`, `tout`) comes back once. The first version of this
      reconstruction returned every process once. E82 changed it.
    - *Decision, on conflicting evidence.* **A returned relation goes to the
      head of the list.** E82 says the relations "appended", but the lists
      it prints come out only if they go to the head. Appended to the end,
      the list it prints as (2+14 = 16; w*2 = 14; w*2 = 14) would be (w*2 =
      14; 2+14 = 16; w*2 = 14), and its next step, "The first relation is
      examined ... and found satisfied", would be false. The bindings at the
      end are the same either way (`tests/historical.rs`, `e82_p123_*`).
      *Experiment.* `--wake back` appends them. So a derivation
    can finish with suspended equations: its "binding environment exhibits
    a 'partially' solved form constrained by the suspended equations" (p.
    12). `absys` shows these. Asked for `U` after `rev([1,2,U],[V|L])`, it
    says `U = V` and gives the equations `U = X₃` and `X₃ = V` that make it
    so.

11. *Decision.* **One equation is elaborated in one step.** E90 says the
    list case "in effect spawns equality processes between the heads and
    tails". Here they are worked through at once, inside the same step,
    and the processes they wake are added to the goal. Nothing the sources
    show depends on the difference.

12. *Source.* **Equations are elaborated as they arise, before the next
    application.**
    - E90 p. 6: "Absys normally processes 'unification equations' as they
      arise".
    - E90 p. 9: "the ongoing equation solving establishes bindings which
      are immediately used".
    - *Decision.* A derivation therefore keeps two queues. It always takes
      an equation first if it has one, and otherwise a literal.

## Alternatives, and the search

13. *Source.* **`or` distributes through the goal of the derivation it is
    in.** E90 p. 9: it is "essentially creating two new goals which can
    notionally be regarded as initiating parallel derivations". FE69:
    "Unsatisfiable conjunctions disappear when unsatisfiability is
    detected", and, a few sentences later, the conjunctions "in effect
    constitute parallel non-interacting programs".
    - Each alternative gets a copy of the derivation: its bindings, its
      suspended processes and the rest of its goal.
    - *Decision.* The copy is made when the `or` is reached, which for an
      application is the step after the application. FE69 says Absys 1
      shared processing between the copies ("in a way which attempts to
      minimise unnecessary duplication"). That changes cost, not results.

14. *Source.* **The search is fair.** E90 p. 10: "the implementation of
    Absys used a 'fair' search rule, in that all derivations continue to be
    incrementally elaborated until found unsatisfiable". The derivations
    take a step each, in turn.
    - *Experiment.* `--search depth-first` elaborates the first derivation
      until it fails or hibernates, as a backtracking Prolog would.
    - Kowalski (1988) credits Absys with "the central role of
      backtracking", and E82 p. 122 says: "each conjunction notionally
      constituting a separate (parallel) computation branch ... In practice,
      of course, the non-determinism was handled by appropriate differential
      record keeping and backtracking in a similar spirit to Prolog
      implementations." So the designer's own accounts disagree: in 1982
      the implementation backtracked; in 1990 it "used a 'fair' search
      rule". Backtracking and fairness are compatible only if the
      backtracking interleaved the branches, which E82 does not say.
    - *Decision, on conflicting evidence.* The derivations here are
      elaborated fairly, as E90 says, which is also the "notional" meaning
      E82 gives. `--search depth-first` is the backtracking reading. Nothing
      published gives a result that tells the two apart.

15. *Decision, on conflicting evidence.* **Within a derivation, the body of
    an application takes the place of the literal applied.** So its
    literals are elaborated before the literals that came after it.
    - E90 p. 9 says the goal literal is "replaced by" the body.
    - E90 p. 3 also says "Absys used a fair computation rule", and note 9
      says "Absys does not guarantee a particular order of processing of
      conjuncts".
    - A fair rule, in the sense E90 cites (Lassez and Maher), is one that
      selects every literal eventually. Taking literals strictly in turn
      (first in, first out) is one such rule, not the only one.
    - E90 prints `setdiff` (p. 15) as a definition only, with no query and
      no result, and does not define `mem`. Run on inputs chosen here,
      `[1,2,3,4]` and `[2,4,6]`, with `mem` defined here, strictly in turn
      it computes `[1]`, not `[1,3]`. The reason is that `not`, which is not
      safe (rule 18), is then elaborated before the recursion that
      instantiates its variable.
    - Strictly in turn but with `not` delayed until its variables are
      instantiated, it computes `[1,3]` too. E90 p. 3 mentions "primitive
      and user declared delay constructs" it does not describe.
    - So the example favours the body in the literal's place without
      proving it. The default follows it, as the simplest rule under which
      E90's examples work without an undescribed delay.
    - *Experiment.* `--select fifo` gives the strictly fair literal rule
      (docs/research.md, E02).

16. *Source.* **A derivation with nothing left to do hibernates**, and a
    later assertion can wake it. E90 note 13: "'Hibernate' is a better
    word, since the computational process associated with the derivation
    can be reactivated". It counts as a refutation (p. 11).

17. *Source.* **The system is incremental.**
    - FE69: "as assertions are accepted by the system, whatever processing
      can be done on the basis of data already present in the system is
      done".
    - E90 p. 13 says an assertion typed later is added to the text "and
      augment[s] the goal".
    - *Decision.* Each assertion of a file is accepted in turn. It is added
      to every derivation alive, and the computation runs until each
      derivation hibernates or fails, or the budget runs out (100000 steps
      for each assertion by default).

## Negation and aggregation (1990 notation)

18. *Source.* **`not` starts an independent computation with the criteria
    reversed** (E90 p. 14: it "acts like a degenerate or in that during
    goal evaluation it initiates an independent computational branch, but
    one in which the satisfiability criteria for termination are
    reversed"). If any derivation of the negated assertions reaches a
    refutation, the `not` fails. If every one fails, it holds.
    - *Source.* It is not safe. E90 note 18: "both Absys and PROLOG have
      the shortcoming that they do not use a safe computation rule".
    - *Decision.* The computation starts from the bindings as they stand,
      and does not wait for its variables.
    - *Experiment.* `--negation delayed` makes `not` wait until every
      variable of the negated assertions is instantiated.
    - *Decision.* A derivation of the independent computation that
      hibernates counts as a refutation only if what it leaves suspended is
      equations between variables, or output.
      - E90 counts a derivation that ends with suspended equations as a
        refutation because "the constraint is always satisfiable" (p. 12).
        A process waiting for something else has no such guarantee. An
        example is an application of a procedure not yet defined.
      - If the computation can go no further and has neither a refutation
        nor a failure in every derivation, the `not` is set aside. It waits
        on whatever its computation waited for outside itself, and starts
        again from the bindings as they then stand. That can include an
        identifier of the text it met first, such as a procedure defined
        later.
      - Without this, a `not` or `set` applied before a procedure it uses
        was defined gives a wrong answer (docs/research.md, E02).

19. *Source.* **`{set(X), {…}, S}`** is "the Absys primitive aggregation
    operator set" of Absys-4 (E90 p. 15). The form is transcribed as
    printed.
    - *Decision.* It starts an independent computation, as `not` does.
      When every derivation of it has hibernated or failed, S is equated
      with the list of the distinct values of X in the derivations that
      hibernated, in the order those derivations stand.
    - *Decision.* An empty set gives `[]`, which `setdiff` needs.
    - *Decision.* It counts refutations as `not` does, and waits in the
      same way when it cannot yet be decided.
    - E90 says only that set was "implemented in an analogous way to the
      PROLOG operator". Prolog's `setof` sorts its answers and fails when
      there are none. Neither of those is done here.

20. *Decision.* **`tout("U", U)` writes `U = [2]` when U is complete.**
    E90 note 11 says only that the example refutation "would cause the
    output 'assertion' tout("U", U) to produce 'U = [2]'". It is not said
    when. U is complete only at the end of that refutation, so here `tout`
    waits, like any other process, until its term has no uninstantiated
    variable in it.

## The 1969 notation

21. *Source.* **`key`** (FE69): "The key statement indicates the data that
    must be present before evaluation of the function may take place". An
    application waits until its key variables are instantiated.
    - E90 p. 3 mentions "both primitive and user declared delay
      constructs". `key` is the only user-declared one with a known
      syntax.
    - FE69 says that without `key`, "processing of map might give rise to
      further processing of map ... before processing the assertion p =
      [r'&s']", so that it would not terminate.
      - When the list is already present, that cannot happen under rule
        12, so here `map` without `key` terminates. This is a place where
        the reconstruction is more orderly than Absys 1 promised to be.
      - When the list is not yet present, `map` without `key` goes on for
        ever here, making lists of every length. With `key` it waits.

22. *Source.* **`=> z`**: a function with a result. FE69 says `f(m,n) = b`
    "is equivalent to" the body with `b` for `z`. So `i = item(q)` is the
    application `item(q, i)`. An application inside a term gets a new
    variable for its result.

23. *Source.* **Arithmetic is an assertion, usable in every direction.**
    - FE69: "the assertions a = b + 1, a = 1 + b, 1 + b = a, b + 1 = a are
      equivalent".
    - F68 gives the "pass sets" of `plus(p,q,r)`: any two of the three, or
      all three.
    - `a + b = c` is therefore elaborated when two of a, b and c are
      numbers, and otherwise waits. `-` and `/` are the same relations read
      the other way.
    - *Decision.* Numbers are integers. A division must come out whole.
    - *Decision.* Arithmetic is in the 1969 notation only. E90 shows none,
      and allows `-` in names such as `math-course`.

24. *Decision.* **`null(p)` is `p = []`.**

## What is not reconstructed

- **The higher-order features** of the full Absys-4 language (E90 p. 3,
  described in a report not seen). This includes partial application
  (FE69: "Functions may be introduced by partial application of other
  functions"). A lambda expression inside a lambda expression is refused.
- **The "simple control mechanisms"** of Absys-4 (E90 p. 3), and any
  primitive delay other than the ones above. Their form is unknown.
- **The cost and ordering hints of F68** ("a number may be associated with
  an assertion to indicate the cost of attempting it"). F68 is a design
  paper; it describes the system as then "partly implemented on an Elliott
  4120", and nothing later shows these hints in Absys.
- **The Elliott 4120 implementation itself.** It was written in machine
  code (Reid 2011 quotes Gray on its "tie-in to a manufacturers' machine
  code"), and no listing of it is known to us. This is a
  reconstruction of the language as described, not an emulation of the
  program.
