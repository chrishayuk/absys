# Sources

No program text, listing, manual or tape of Absys from Aberdeen is known to
survive. The implementation was in Elliott 4120 machine code, and the machine
was broken up in the mid-1970s (Reid 2011). This reconstruction therefore
rests on what was published about Absys.

Copies of the papers are not kept in the repository. They belong to their
publishers. `sources/downloads/` is ignored by git, and the list below says
where each one can be had. What was read from each, and how, is in
`notes/`.

## Primary sources: by the people who built it

| | source | what was used | access |
|---|---|---|---|
| **E90** | E. W. Elcock, "Absys: the first logic programming language: a retrospective and a commentary", *J. Logic Programming* 9(1) (1990) 1-17. doi:10.1016/0743-1066(90)90030-9 | all of it, read from the scanned pages; [notes](notes/elcock-1990.md) | Elsevier open archive. CORE (https://core.ac.uk/download/pdf/82537271.pdf) refuses automated fetching; the copy used was downloaded by hand |
| **FE69** | J. M. Foster and E. W. Elcock, "Absys 1: an incremental compiler for assertions; an introduction", in *Machine Intelligence 4* (Meltzer and Michie, eds.), Edinburgh U.P. (1969) 423-429 | OCR extracts of its sections "Introduction", "The 'AND' connective", "Data directed control", "Functions", "The 'OR' connective" and "Recursive functions and keys", as quoted on HOPL (exp=357; a Wayback copy, since hopl.info's certificate has lapsed); [notes](notes/foster-elcock-1969.md) | the whole volume is on archive.org (`machineintellige0004unse`), for lending only. **The last pages, roughly 428-429, have not been seen** |
| **F68** | J. M. Foster, "Assertions: programs written without specifying unnecessary order", in *Machine Intelligence 3* (Michie, ed.), Edinburgh U.P. (1968) 387-391 | all of it, from the archive.org OCR | archive.org `mi3_20200519`, open |

Also consulted:

- E. W. Elcock, "Descriptions", *Machine Intelligence 3* (1968) 173-180.
  This is on recognition by assertions; nothing in it is used here.
- The ABSET papers. ABSET was the Aberdeen group's successor language:
  - Elcock, Foster, Gray, McGregor and Murray, "ABSET: a programming
    language based on sets; motivation and examples", *Machine Intelligence
    6* (1971).
  - Elcock, McGregor and Murray, "Data directed control and operating
    systems", *Computer J.* 15(2) (1972) 125-129.

  Both were read in HOPL extracts (exp=356). They describe the "novalue"
  object that holds a list of waiting jobs, which they say is what FE69
  called data directed control. That supports the mechanism of decisions 9
  and 10, but it is ABSET, not Absys.

Not seen:

- E. W. Elcock and P. M. D. Gray, "Absys, equation solving and logic
  programming", TR 213, Dept. of Computer Science, University of Western
  Ontario (1988). This is E90's reference [9], which E90 says describes the
  full Absys-4 language.
- E. W. Elcock, "Problem solving compilers", in Findler and Meltzer (eds.),
  *Artificial Intelligence and Heuristic Programming*, Edinburgh U.P.
  (1971). E90 says "more complex examples can be found" there.
- E. W. Elcock, "Absys: the historical inevitability of logic programming",
  NACLP 1989, and "Absys, the first logic-programming language: a view of
  the inevitability of logic programming", in *Computational Logic: Essays
  in Honor of Alan Robinson* (MIT Press, 1991).
- The Absys 1 "complete description of the language, which will appear
  elsewhere" (FE69). It does not appear to have been published.

## Secondary sources

- R. A. Kowalski, "The early years of logic programming", *Comm. ACM* 31
  (1988) 38-44. It credits Absys with anticipating "'invertability',
  'negation by failure', 'aggregation operators', and the central role of
  backtracking". The last of these is not in E90, which describes a fair
  search instead (decision 14).
- J. Cohen, "Constraint logic programming languages", *Comm. ACM* 33(7)
  (1990). It credits Absys I with "non-determinism, the elimination of the
  occur test, and testing for satisfiability of systems of equations", but
  places it at Edinburgh. It was Aberdeen.
- A. Colmerauer and P. Roussel, "The birth of Prolog" (1992). It mentions
  "the prior existence of the Absys language, a possible competitor with
  Prolog in the opinion of E. Elcock".
- J. S. Reid, "Note on the Aberdeen University Elliott 4120 computer"
  (2011). It covers the machine, the group (Elcock, Murray and Foster from
  1966; Gray from 1968; McGregor from 1970), and the fact that Absys was
  written in machine code.

## Which Absys?

| | when | what is known of it | here |
|---|---|---|---|
| F68's proposal | written 1967 | ideas, not a language: assertions joined by `and`, `new` variables, cost and pass-set ordering | pass sets for arithmetic (decision 23) |
| **Absys 1** | "a working on-line incremental compiler" by 1968 (FE69) | the 1969 notation: `begin … end`, `{… or …}`, `[a;b]`, `[h&t]`, primes, `lambda … => z key …`, arithmetic | `.ab1` files |
| **Absys-4** | "circa 1968" (E90) | E90 describes "a subset" of it, "in a 'modern' redescription" with "slight liberties" in the syntax, adding `not` and `set` | `.abs` files |

The dates conflict:

- E90 says Absys was "essentially completed in 1967".
- F68, given at a workshop in 1967, says it is "partly implemented".
- Reid (2011) has it developed "in 1968 and beyond".

The machine is the Elliott 4120 in every primary source. The KDF9 and the
Elliott 503 are sometimes named in secondary accounts without support.

No source gives the full syntax of either version. Neither has a grammar or
a manual. Every rule this reconstruction adds where the sources are silent
is in `docs/decisions.md`, marked as a decision.
