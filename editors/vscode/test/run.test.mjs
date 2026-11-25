// Runs every example through the runner the "Run This Text" command uses,
// and compares what it prints with what the native absys prints, when that
// has been built.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const runner = fileURLToPath(new URL("../server/run.js", import.meta.url));
const examples = fileURLToPath(new URL("../../../examples/", import.meta.url));
const native = fileURLToPath(new URL("../../../target/release/absys", import.meta.url));

function run(text, options = [], name = "text.abs") {
  const file = join(mkdtempSync(join(tmpdir(), "absys-")), name);
  writeFileSync(file, text);
  return { file, ...spawnSync(process.execPath, [runner, ...options, file]) };
}

test("the append refutation", () => {
  const r = run("app = lambda R, S, T\n {R = [], S = T} or\n {R = [X|L], T = [X|N], app(L, S, N)}\napp([1], U, [1,2])\n", ["--env"]);
  assert.equal(r.status, 0, r.stderr.toString());
  assert.match(r.stdout.toString(), /U = \[2\]/);
  assert.match(r.stdout.toString(), /X₁\/1, L₁\/\[\], N₁\/\[2\]/);
});

test("the 1969 notation, from the file name", () => {
  const r = run("x=y   y = 2", [], "text.ab1");
  assert.match(r.stdout.toString(), /x = 2/);
});

test("a text not in the notation stops with its place", () => {
  const r = run("X = Y +\n");
  assert.equal(r.status, 2);
  assert.match(r.stderr.toString(), /text\.abs:1:7: `\+` is not in the notation/);
});

test("every example agrees with the native absys", { skip: !existsSync(native) && "no native build" }, () => {
  for (const f of readdirSync(examples).filter((n) => n.endsWith(".abs") || n.endsWith(".ab1"))) {
    const text = readFileSync(examples + f, "utf8");
    const mine = run(text, ["--env"], f);
    const theirs = spawnSync(native, ["--env", examples + f]);
    assert.equal(mine.stdout.toString(), theirs.stdout.toString(), f);
  }
});
