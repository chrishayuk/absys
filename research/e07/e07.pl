% ABSYS-E07: the experiments of docs/research.md, run on SWI-Prolog, beside
% the Absys texts they mirror. Each query runs under an inference limit, so
% one that would go on for ever is reported as such instead of hanging.
%
%   swipl -q research/e07/e07.pl
%
% Prints one line per experiment: what it is, and what Prolog does.

:- use_module(library(clpfd)).
:- use_module(library(clpq)).

limit(1000000).

%% run(+Label, :Goal): the first answer, or how the goal ended.
run(Label, Goal0, Show0) :-
    copy_term(Goal0-Show0, Goal-Show),   % each experiment on its own variables
    limit(L),
    catch(( call_with_inference_limit(Goal, L, R)
          -> ( R == inference_limit_exceeded -> Out = 'no answer: the inference limit ran out'
             ; format(atom(Out), 'first answer: ~w', [Show]) )
          ;  Out = 'fails'
          ),
          E,
          ( E = error(Err, _) -> format(atom(Out), 'error: ~q', [Err]) ; format(atom(Out), 'error: ~q', [E]) )),
    format("~w~t~58|~w~n", [Label, Out]).

%% all(+Label, +Template, :Goal): every answer, or how the search ended.
all(Label, T0, Goal0) :-
    copy_term(T0-Goal0, T-Goal),
    limit(L),
    catch(( call_with_inference_limit(findall(T, Goal, Ts), L, R),
            ( R == inference_limit_exceeded -> format(atom(Out), 'answers before the limit ran out: none collected (findall never finished)', [])
            ; length(Ts, N), format(atom(Out), '~d answers: ~w', [N, Ts]) ) ),
          error(Err, _), format(atom(Out), 'error: ~q', [Err])),
    format("~w~t~58|~w~n", [Label, Out]).

%% count(+Label, :Goal): answers found before the search went on for ever.
count(Label, T0, Goal0) :-
    copy_term(T0-Goal0, T-Goal),
    limit(L),
    nb_setval(found, []),
    catch(call_with_inference_limit((Goal, nb_getval(found, F0), nb_setval(found, [T|F0]), fail ; true), L, R), _, R = error),
    nb_getval(found, F), reverse(F, Fs), length(Fs, N),
    ( R == inference_limit_exceeded -> Tail = ', then no more: the inference limit ran out' ; Tail = ', and the search ended' ),
    format("~w~t~58|~d answers ~w~w~n", [Label, N, Fs, Tail]).

% ---- the programs ----

app([], S, S).
app([X|L], S, [X|N]) :- app(L, S, N).

rev1([], S, S).
rev1([X|L], S, T) :- rev1(L, [X|S], T).
rev(R, S) :- rev1(R, [], S).

loop(X) :- loop(X).
p(A) :- loop(A).
p(1).

% left recursion, as research/e04-left-recursion.abs
as(L) :- as(M), L = [a|M].
as([]).

mem(X, [X|_]).
mem(X, [_|T]) :- mem(X, T).
setdiff(S1, S2, S) :- findall(X, (mem(X, S1), \+ mem(X, S2)), S).

% Foster's pass sets (1968) as coroutines: a+b=c is taken up when two of
% its three numbers are known, and waits otherwise, like Absys's arithmetic.
two_known(A, B, C) :- ( nonvar(A), nonvar(B) ; nonvar(A), nonvar(C) ; nonvar(B), nonvar(C) ), !.
plus(A, B, C)  :- when(((nonvar(A),nonvar(B));(nonvar(A),nonvar(C));(nonvar(B),nonvar(C))), plus_(A, B, C)).
plus_(A, B, C) :- ( var(C) -> C is A+B ; var(B) -> B is C-A ; var(A) -> A is C-B ; C =:= A+B ).
times(A, B, C) :- when(((nonvar(A),nonvar(B));(nonvar(A),nonvar(C));(nonvar(B),nonvar(C))), times_(A, B, C)).
times_(A, B, C) :- ( var(C) -> C is A*B ; var(B) -> 0 =:= C mod A, B is C//A ; var(A) -> 0 =:= C mod B, A is C//B ; C =:= A*B ).

student(brown). student(smith).
math_course(c101). math_course(c301).
takes(brown, c101). takes(smith, c101). takes(smith, c301).
non_maths_major(X) :- math_course(C), \+ takes(X, C).

main :-
    format("~nE01: one procedure, several directions~n"),
    all('app(R, S, [1,2])', R+S, app(R, S, [1,2])),
    run('app([1,2], S, T)', app([1,2], S, T), T-S),
    run('rev(R, [1,2,3]), first answer', rev(R, [1,2,3]), R),
    count('rev(R, [1,2,3]), every answer', R, rev(R, [1,2,3])),

    format("~nE02: order~n"),
    run('\\+ X = 1, X = 2', (\+ X1 = 1, X1 = 2), X1),
    run('X = 2, \\+ X = 1', (X2 = 2, \+ X2 = 1), X2),
    run('setdiff([1,2,3,4], [2,4,6], D)', setdiff([1,2,3,4], [2,4,6], D), D),
    run('non_maths_major(brown)', non_maths_major(brown), yes),
    run('non_maths_major(smith)', non_maths_major(smith), yes),

    format("~nE03: a goal before its procedure~n"),
    run('later(A, B), later not yet defined', call(later, _, _), defined),

    format("~nE04: a first alternative that never ends~n"),
    run('p(A): loop(A) first, then A = 1', p(A4), A4),
    run('as([a,a]): left recursion, list known', as([a,a]), yes),

    format("~nE06: Elcock 1982, u+v=16 and w*u=v and u+10=12~n"),
    run('is/2, in that order', (16 is U+V, V is W*U, 12 is U+10), [U,V,W]),
    run('is/2, reordered by hand', (U2 is 12-10, V2 is 16-U2, W2 is V2/U2), [U2,V2,W2]),
    run('clpfd, in that order', (U3+V3 #= 16, W3*U3 #= V3, U3+10 #= 12), [U3,V3,W3]),
    run('when/2 pass sets, in that order', (plus(U4, V4, 16), times(W4, U4, V4), plus(U4, 10, 12)), [U4,V4,W4]),

    format("~nE09: x+y=10 and x-y=4~n"),
    run('when/2 pass sets', (plus(X9, Y9, 10), plus(Y9, 4, X9)), [X9,Y9]),
    run('clpfd', (X5+Y5 #= 10, X5-Y5 #= 4), [X5,Y5]),
    run('clpfd, then label', (X6+Y6 #= 10, X6-Y6 #= 4, X6 in -100..100, label([X6,Y6])), [X6,Y6]),
    run('clpq', ({X7+Y7 = 10, X7-Y7 = 4}), [X7,Y7]),
    nl.

:- initialization((main, halt)).
