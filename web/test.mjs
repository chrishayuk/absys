// Runs every example through the WebAssembly build and compares it with the
// native build:  node web/test.mjs   (after node web/build.mjs and cargo build --release)
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { absys } from "./worker.js";

const root = fileURLToPath(new URL("..", import.meta.url));
const module = await WebAssembly.compile(readFileSync(`${root}web/absys.wasm`));
const native = `${root}target/release/absys`;
let failed = 0;
const fail = (m) => (failed++, console.log("FAIL " + m));
const examples = JSON.parse(readFileSync(`${root}web/examples.json`, "utf8"));
for (const ex of examples) {
  const args = ["json", "--notation", ex.notation, "--steps", "20000"];
  const wasm = absys(module, args, ex.text);
  const want = execFileSync(native, args, { input: ex.text, maxBuffer: 64 << 20 }).toString();
  if (wasm.out !== want) fail(`${ex.name}: the wasm build and the native build differ`);
  const run = JSON.parse(wasm.out);
  if (run.error) fail(`${ex.name}: ${run.error.message}`);
  if (!run.steps.length) fail(`${ex.name}: no steps`);
}
// the reverse example hibernates with U = X₃ and X₃ = V suspended, then U = 5
const rev = examples.find((e) => e.name === "reverse");
const r = JSON.parse(absys(module, ["json"], rev.text).out);
const mid = r.steps[2].derivations[0];
if (mid.suspended.map((s) => s.text).sort().join(", ") !== "U = X₃, X₃ = V") fail("reverse: the suspended equations");
const last = r.steps[3].derivations[0];
if (!last.answers.some((a) => a.name === "U" && a.value === "5")) fail("reverse: U = 5 after rev([5], [V])");
// the language server answers in the page as it does in an editor
import { answer } from "./lsp-worker.js";
const app = examples.find((e) => e.name === "append");
const line = app.text.split("\n").findIndex((l) => l.startsWith("app([1]"));
const lsp = answer(module, app.text, [
  { method: "textDocument/semanticTokens/full" },
  { method: "textDocument/hover", params: { position: { line, character: 9 } } },
]);
if (!lsp.results[0]?.data?.length) fail("lsp: no semantic tokens");
if (!/U = \[2\]/.test(lsp.results[1]?.contents?.value ?? "")) fail("lsp: hover on the goal does not give U = [2]");
console.log(`${examples.length} texts through the wasm build, ${failed ? failed + " failed" : "all agree with the native build"}`);
process.exit(failed ? 1 : 0);
