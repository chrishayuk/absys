# 1. Assertions, not instructions

Absys was designed and built around 1967 by the Computer Research Group at the
University of Aberdeen (J. M. Foster, E. W. Elcock, and colleagues) on an
Elliott 4120. Foster's paper on the ideas behind it has the title
*Assertions: Programs Written without Specifying Unnecessary Order*. Its
introduction says what that means:

> The basic elements in the system are not instructions to do something, as are
> the statements of Algol, but assertions about the data, such as a=b or a+b=c.
> The evaluation of a program of assertions is not the obeying in specified
> sequence of a set of instructions, but an attempt to find data which
> satisfies the assertions. (Foster 1968)

This tutorial teaches the language as it is reconstructed here, which is from
what its designers published (`sources/README.md`). Where the sources are
silent and a decision had to be made, the tutorial says so and points to
`docs/decisions.md`.

## Install

You need [Rust](https://rustup.rs). There are no other dependencies.

```sh
cargo install --path .
absys examples/append.abs
```

Or use the [playground](../web/index.html), which runs the same program in the
browser and lets you step through a text one assertion at a time.

## A first text

An Absys text is a conjunction of assertions. `absys` accepts them one at a
time, the way the Aberdeen system accepted them from a typewriter. As each
assertion is accepted, whatever can be done is done:

```absys
X = Y
Y = 2
```

```output
1 derivation:
  derivation #0 (hibernating)
    X = 2
    Y = 2
```

`X = Y` is not an assignment. Nothing was known about either variable when it
was accepted, so it could only be *kept*. When `Y = 2` arrived, the kept
equation was taken up again, and gave `X` its value. The order of the two
lines makes no difference to the result.

The output is described in later chapters:

- A **derivation** is one way of satisfying the text (chapter 5).
- **Hibernating** means the derivation has nothing more it can do until
  something more is asserted (chapter 6).

## Something that cannot hold

```absys
Y = X
X = 2
3 = Y
```

```output
No derivation survives: the assertions are unsatisfiable.
```

Foster and Elcock (1969) use this example: "the occurrence in a program of
the assertions y=x, x=2 and 3=y would make that program terminate
unsuccessfully".

The next chapter looks at what an equation does, and what happens when it
cannot yet be decided.
