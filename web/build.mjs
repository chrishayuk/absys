// Builds the files the playground needs:  node web/build.mjs
//   web/absys.wasm      absys, compiled for WASI: the interpreter and the language server (`absys lsp`)
//   web/examples.json   the examples and the research texts, with their notes
import { execSync } from "node:child_process";
import { copyFileSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));
execSync("cargo build --release --target wasm32-wasip1", { cwd: root, stdio: "inherit" });
copyFileSync(`${root}target/wasm32-wasip1/release/absys.wasm`, `${root}web/absys.wasm`);

// The order the playground lists them in: the sources' examples first.
const first = ["assertions-not-assignments", "append", "reverse", "students", "setdiff", "assertions", "alternatives", "triples", "map", "relation-list", "mem-1982"];
const read = (dir) =>
  readdirSync(`${root}${dir}`)
    .filter((f) => /\.ab[s12]$/.test(f))
    .map((f) => {
      const text = readFileSync(`${root}${dir}/${f}`, "utf8");
      const name = f.replace(/\.ab[s12]$/, "");
      const title = ((/^%\s*(.*)$/m.exec(text) || [, name])[1]).replace(/[:.]$/, "");
      return { name, file: `${dir}/${f}`, notation: f.endsWith(".ab1") ? "1969" : f.endsWith(".ab2") ? "1982" : "1990", title, text, group: dir };
    });
const examples = read("examples").sort((a, b) => {
  const i = (x) => (first.indexOf(x.name) + 1 || 99);
  return i(a) - i(b) || a.name.localeCompare(b.name);
});
const all = [...examples, ...read("research").sort((a, b) => a.name.localeCompare(b.name))];
writeFileSync(`${root}web/examples.json`, JSON.stringify(all));
console.log(`web/absys.wasm, web/examples.json (${all.length} texts) written`);
