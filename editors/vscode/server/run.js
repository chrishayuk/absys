// Runs a text:  node run.js [options] text.abs
// The text is read here and given to absys as the one file it can open,
// under its own name, so absys needs no access to the disk. The options are
// absys's own (--trace, --each, --env, --search …, --steps N).
"use strict";

const fs = require("fs");
const path = require("path");
const { run } = require("./wasi.js");

const wasm = path.join(__dirname, "absys.wasm");
const args = process.argv.slice(2);
const file = args.pop();
if (!file || file.startsWith("--")) {
  console.error("usage: run.js [options] text.abs");
  process.exit(2);
}
let text;
try {
  text = fs.readFileSync(file);
} catch (error) {
  console.error(`absys: cannot read ${file}: ${error.message}`);
  process.exit(2);
}
const base = path.basename(file);
// A name that starts with a dash would be read as an option.
const name = base.startsWith("-") ? `./${base}` : base;
process.exitCode = run(wasm, { args: [...args, name], files: { [base]: text }, keyboard: false });
