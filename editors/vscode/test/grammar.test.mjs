// Tokenises the examples with the same TextMate engine VS Code uses and
// checks the scopes the grammar assigns.
import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { createRequire } from "node:module";
import test from "node:test";
import { fileURLToPath } from "node:url";
import oniguruma from "vscode-oniguruma";
import textmate from "vscode-textmate";

const require = createRequire(import.meta.url);
const grammarPath = fileURLToPath(new URL("../syntaxes/absys.tmLanguage.json", import.meta.url));
const examples = fileURLToPath(new URL("../../../examples/", import.meta.url));

const wasm = readFileSync(require.resolve("vscode-oniguruma/release/onig.wasm"));
await oniguruma.loadWASM(wasm.buffer.slice(wasm.byteOffset, wasm.byteOffset + wasm.byteLength));
const registry = new textmate.Registry({
  onigLib: Promise.resolve({
    createOnigScanner: (patterns) => new oniguruma.OnigScanner(patterns),
    createOnigString: (text) => new oniguruma.OnigString(text),
  }),
  loadGrammar: async () => textmate.parseRawGrammar(readFileSync(grammarPath, "utf8"), grammarPath),
});
const grammar = await registry.loadGrammar("source.absys");

/** Each non-blank piece of text with its most specific scope. */
function scopes(source) {
  const out = [];
  let state = textmate.INITIAL;
  for (const line of source.split("\n")) {
    const r = grammar.tokenizeLine(line, state);
    for (const t of r.tokens) {
      const text = line.slice(t.startIndex, t.endIndex);
      if (text.trim()) out.push([text.trim(), t.scopes[t.scopes.length - 1]]);
    }
    state = r.ruleStack;
  }
  return out;
}
const scopeOf = (source, piece) => scopes(source).find(([t]) => t === piece)?.[1];

test("the 1990 notation", () => {
  const src = "app = lambda R, S, T\n {R = [], S = T} or {R = [X|L], T = [X|N], app(L, S, N)}  % append";
  assert.equal(scopeOf(src, "app"), "entity.name.function.definition.absys");
  assert.equal(scopeOf(src, "lambda"), "keyword.control.lambda.absys");
  assert.equal(scopeOf(src, "or"), "keyword.control.absys");
  assert.equal(scopeOf(src, "R"), "variable.other.absys");
  assert.equal(scopeOf(src, "|"), "keyword.operator.list.absys");
  assert.equal(scopeOf(src, "% append"), "comment.line.percentage.absys");
  const s = "non-maths-major = lambda X\n {math-course(C), not(takes(X, C))}\nstudent('BROWN') tout(\"U\", U)";
  assert.equal(scopeOf(s, "non-maths-major"), "entity.name.function.definition.absys");
  assert.equal(scopeOf(s, "math-course"), "entity.name.function.call.absys");
  assert.equal(scopeOf(s, "not"), "keyword.control.absys");
  assert.equal(scopeOf(s, "'BROWN'"), "string.quoted.single.absys");
  assert.equal(scopeOf(s, "tout"), "support.function.builtin.absys");
});

test("the 1969 notation: primes are not quotes", () => {
  const src = "map'=lambda p, f key p\n begin {null(p) or p = [r'&s'] f(r) map(s,f)} end\nx = '='";
  assert.equal(scopeOf(src, "map"), "entity.name.function.definition.absys");
  assert.equal(scopeOf(src, "key"), "keyword.control.absys");
  assert.equal(scopeOf(src, "null"), "support.function.builtin.absys");
  const pieces = scopes(src);
  assert.ok(pieces.some(([t, s]) => t === "r" && s === "variable.other.name.absys"));
  assert.ok(pieces.some(([t, s]) => t === "'" && s === "storage.modifier.prime.absys"));
  assert.ok(!pieces.some(([t, s]) => t.startsWith("'&") && s.startsWith("string")), "r'&s' is not a string");
  assert.equal(scopeOf(src, "'='"), "string.quoted.single.absys");
  assert.equal(scopeOf("item'=lambda x=>z key x begin end", "=>"), "keyword.operator.result.absys");
});

test("every example is coloured, and nothing is left unscoped", () => {
  for (const f of readdirSync(examples).filter((n) => n.endsWith(".abs") || n.endsWith(".ab1"))) {
    for (const [text, scope] of scopes(readFileSync(examples + f, "utf8"))) {
      assert.notEqual(scope, "source.absys", `${f}: ${text} has no scope`);
    }
  }
});
