# Absys for Visual Studio Code

Colours, problems as you type, hover and a run button for texts in
[Absys](https://github.com/chrishayuk/absys), the assertional language of
J. M. Foster and E. W. Elcock (University of Aberdeen, 1967-68), as
reconstructed from what was published about it.

- `.abs` files are in the notation of Elcock's 1990 retrospective (`app = lambda R, S, T …`, `{…} or {…}`, `[X|L]`).
- `.ab1` files are in the notation of Foster and Elcock's 1969 paper on Absys 1 (`map'=lambda p, f key p begin … end`, `[a;b]`, `[h&t]`).

## Features

- **Problems as you type.** Anything not in the notation is an error. An assertion after which no derivation survives
  is a warning: the text has become unsatisfiable there. A text whose derivations are still elaborating when the
  server's budget runs out is noted.
- **Hover** on an assertion: the computation after it was accepted, with every derivation, its answers, what is
  suspended and what it has output. On an identifier: its value in each derivation at the end of the text.
- **Colours** from a TextMate grammar and from the server's semantic tokens.
- **Outline**: the procedures the text defines.
- **Completion** of the keywords and the procedures defined.
- **Run This Text** (⌘↵ / Ctrl+Enter, or the ▶ in the editor's title bar): every derivation left at the end, with its
  whole binding environment. Also *Run This Text, Showing the State After Each Assertion* and *… Showing Every Step*.

Everything runs on `absys` compiled to WebAssembly, inside the extension: nothing else needs installing. To use a
native build instead, set `absys.path`. `absys.search` chooses the search the run commands use (`fair`, as Absys, or
`depth-first`, for comparison).
