# Absys in your editor

Two pieces give an editor an understanding of an Absys text:

- **`absys lsp`**, a language server. It is the `absys` program itself,
  given the argument `lsp`, and it works with any editor that speaks the
  Language Server Protocol.
- **A Visual Studio Code extension**, in [`vscode`](vscode). It bundles
  `absys` compiled to WebAssembly, and adds a grammar and run commands.

Both run the reconstruction itself. The server accepts the text an assertion
at a time, exactly as `absys FILE` does (with a smaller budget, 20000 steps
for each assertion, since it runs on every keystroke). So the editor and
`absys` cannot disagree. `.abs` is the 1990 notation; `.ab1` the 1969 one.

## The language server

`absys lsp` talks on standard input and output.

- **Diagnostics.**
  - Anything not in the notation is an error.
  - An assertion after which no derivation survives is a warning: the text
    became unsatisfiable there. The warning gives the last failure.
  - An assertion on which the budget ran out with derivations still
    elaborating is noted.
- **Hover** on an assertion shows the computation after it was accepted:
  each derivation, its answers, what is suspended and its output. On an
  identifier, it shows the identifier's value in each derivation at the
  end. On a keyword, it shows what the keyword does, with the source.
- **Semantic tokens.** The legend is:
  - `comment` (notes);
  - `string` (constants);
  - `keyword` (`lambda`, `or`, `not`, `begin`, `end`, `key`);
  - `number`;
  - `operator`;
  - `variable`;
  - `function` (identifiers applied or defined);
  - `macro` (`tout`, `set`, `null`).
- **Document symbols**: the procedures the text defines.
- **Completion** of the keywords and the procedures defined.

It also builds for WASI: `cargo build --release --target wasm32-wasip1`
writes `target/wasm32-wasip1/release/absys.wasm`. That one file is the
interpreter, the playground's engine and the server.

### Neovim (0.11 or later)

```lua
vim.filetype.add({ extension = { abs = "absys", ab1 = "absys" } })
vim.lsp.config("absys", { cmd = { "/path/to/absys", "lsp" }, filetypes = { "absys" } })
vim.lsp.enable("absys")
```

### Helix

```toml
[language-server.absys]
command = "/path/to/absys"
args = ["lsp"]

[[language]]
name = "absys"
scope = "source.absys"
file-types = ["abs", "ab1"]
comment-token = "%"
language-servers = ["absys"]
```

### Emacs (Eglot)

```elisp
(define-derived-mode absys-mode prog-mode "Absys" (setq-local comment-start "% "))
(add-to-list 'auto-mode-alist '("\\.ab[s1]\\'" . absys-mode))
(with-eval-after-load 'eglot
  (add-to-list 'eglot-server-programs '(absys-mode "/path/to/absys" "lsp")))
```

These configurations follow each editor's documented format, but have not
been run here. The server itself is tested:

- over standard input and output, against both the WebAssembly build and
  the native program (`vscode/test/lsp.test.mjs`);
- inside a real Visual Studio Code (`npm run test:vscode`).

## The Visual Studio Code extension

```sh
cd editors/vscode
npm install
node build.mjs          # server/absys.wasm and dist/extension.js
npm test                # the grammar, the runner and the server
npm run test:vscode     # the extension, inside VS Code
npm run package         # absys-language-0.1.0.vsix
```

The extension's features are listed in its [README](vscode/README.md).
