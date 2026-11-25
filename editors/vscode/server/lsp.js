// Starts the language server: absys compiled for WASI, run as `absys lsp`,
// talking the Language Server Protocol on standard input and output.
"use strict";

const path = require("path");
const { run } = require("./wasi.js");

process.exitCode = run(path.join(__dirname, "absys.wasm"), { args: ["lsp"] });
