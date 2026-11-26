% ABSYS-E07, the session: can a modern Prolog keep one computation open and
% feed it assertions one at a time, as an Absys session does?
%
%   swipl -q research/e07/e07-session.pl
%
% A SWI-Prolog engine (engine_create/3) is a computation that can be
% suspended and handed more input (engine_post/3, engine_fetch/1). The
% session below runs inside one engine. Each assertion is read as text, its
% variables are joined by name to the session's variables, and it is called
% in conjunction with everything before it. Bindings, choice points and
% coroutines (when/2) all live on in the engine between assertions. After
% each assertion it reports the bindings and the goals still waiting
% (copy_term/3).
%
% One thing is not built in, and is written here by hand: an application of
% a procedure that has no definition yet is an existence error in Prolog.
% `waits/1` below makes it wait instead, and a definition (`def(Clauses)`)
% wakes it. That is the layer Absys had in its evaluator.

:- use_module(library(lists)).
:- use_module(library(apply)).

:- dynamic defined/1.        % name/arity of every procedure defined so far

%% the session's loop, inside the engine
session(Env0, Pending0) :-
    engine_fetch(Text),
    term_string(T, Text, [variable_names(Names)]),
    (   T = def(Clauses)          % a definition's variables are its own
    ->  Env = Env0, define(Clauses), Pending1 = [], wake(Pending0)
    ;   foldl(join, Names, Env0, Env),
        assertion(T, Pending0, Pending1)
    ),
    report(Env, Pending1, R),
    engine_yield(R),
    session(Env, Pending1).

join(N=V, Env, Env) :- memberchk(N=W, Env), !, V = W.
join(N=V, Env, [N=V|Env]).

% an assertion is a conjunction of equations and applications
assertion((A, B), P0, P) :- !, assertion(A, P0, P1), assertion(B, P1, P).
assertion(G, P, P) :- G = (_ = _), !, call(G).
assertion(G, P, P) :- G = (_ \= _), !, call(G).
assertion(G, P, P) :- predicate_property(G, built_in), !, call(G).
assertion(G, P0, P) :- waits(G, P0, P).

% an application of a procedure not yet defined waits for its definition
waits(G, P, P) :- functor(G, F, N), ( library_relation(F/N) ; defined(F/N) ), !, call(G).
waits(G, P, [G|P]).

library_relation(add/3).
library_relation(mul/3).

define(Clauses) :-
    forall(member(C, Clauses),
           ( ( C = (H :- _) -> true ; H = C ),
             functor(H, F, N),
             ( defined(F/N) -> true ; dynamic(F/N), assertz(defined(F/N)) ),
             assertz(C) )).

wake(Pending) :- reverse(Pending, Gs), maplist([G]>>assertion(G, [], _), Gs).

report(Env, Pending, state(Bs, Waiting, Undefined)) :-
    reverse(Env, Vs),
    copy_term(Vs-Pending, Bs-Undefined, Waiting).

%% run a session: each assertion in turn, printing the state after it
run_session(Title, Assertions) :-
    format("~n~w~n", [Title]),
    engine_create(_, session([], []), E),
    feed(E, Assertions),
    engine_destroy(E),
    retract_all_defined.

feed(_, []).
feed(E, [A|As]) :-
    (   catch(engine_post(E, A, R), Err, (R = error(Err)))
    ->  show(A, R),
        ( R = error(_) -> true ; feed(E, As) )
    ;   format("  » ~w~n      no derivation: the session has failed~n", [A])
    ).

show(A, error(error(E, _))) :- !, format("  » ~w~n      error: ~q~n", [A, E]).
show(A, error(E)) :- !, format("  » ~w~n      error: ~q~n", [A, E]).
show(A, state(Bs, W, U)) :-
    format("  » ~w~n", [A]),
    maplist([N=V]>>(var(V) -> V = '$VAR'(N) ; true), Bs),   % unknowns by name
    Opts = [numbervars(true), quoted(false), portray(true)],
    exclude([N=V]>>(V == '$VAR'(N)), Bs, Known),
    ( Known == [] -> true ; format("      ~W~n", [Known, Opts]) ),
    forall(member(G, W), format("      waiting: ~W~n", [G, Opts])),
    forall(member(G, U), format("      waiting for a definition: ~W~n", [G, Opts])).

retract_all_defined :-
    forall(retract(defined(F/N)), abolish(F/N)).

%% the waiting relations of 1982, as coroutines on Foster's pass sets
add(A, B, C) :- when(((nonvar(A),nonvar(B));(nonvar(A),nonvar(C));(nonvar(B),nonvar(C))), add_(A, B, C)).
add_(A, B, C) :- ( var(C) -> C is A+B ; var(B) -> B is C-A ; var(A) -> A is C-B ; C =:= A+B ).
mul(A, B, C) :- when(((nonvar(A),nonvar(B));(nonvar(A),nonvar(C));(nonvar(B),nonvar(C))), mul_(A, B, C)).
mul_(A, B, C) :- ( var(C) -> C is A*B ; var(B) -> 0 =:= C mod A, B is C//A ; var(A) -> 0 =:= C mod B, A is C//B ; C =:= A*B ).

rev_defs("def([(rev(R, S) :- rev1(R, [], S)), rev1([], S, S), (rev1([H|L], S, T) :- rev1(L, [H|S], T))])").

main :-
    rev_defs(Rev),
    run_session("E03 (research/e03-incremental.abs): the goal before its procedure",
        [ "A = [X, Y, Z]", "rev(A, B)", "X = Y", Rev, "Y = Z", "B = [42|C]" ]),
    run_session("README: rev([1,2,U], [V|L]), then rev([5], [V])",
        [ Rev, "rev([1,2,U], [V|L])", "rev([5], [V])" ]),
    run_session("E06 (Elcock 1982), one relation at a time: u+v=16, w*u=v, u+10=12",
        [ "add(U, V, 16)", "mul(W, U, V)", "add(U, 10, 12)" ]),
    run_session("A later assertion that refutes the first alternative: Prolog backtracks into it",
        [ "def([c(1), c(2), c(3)])", "c(X)", "X > 1", "X \\= 2" ]),
    run_session("The same, with the refuting assertion before the choice",
        [ "def([c(1), c(2), c(3)])", "X > 1", "c(X)" ]),
    nl.

:- initialization((main, halt)).
