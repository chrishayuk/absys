// Talks to the language server the way an editor does: over standard input
// and output. It tests the WebAssembly build the extension ships and, when
// it has been built, the native program other editors would use.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import test from "node:test";
import { fileURLToPath } from "node:url";

const here = fileURLToPath(new URL("..", import.meta.url));
const native = fileURLToPath(new URL("../../../target/release/absys", import.meta.url));
const servers = [["WebAssembly", process.execPath, [`${here}server/lsp.js`, "--stdio"]]];
if (existsSync(native)) servers.push(["native", native, ["lsp"]]);

function connect(command, args) {
  const child = spawn(command, args, { stdio: ["pipe", "pipe", "inherit"] });
  let buffer = Buffer.alloc(0);
  const waiting = [];
  const arrived = [];
  child.stdout.on("data", (chunk) => {
    buffer = Buffer.concat([buffer, chunk]);
    for (;;) {
      const header = buffer.indexOf("\r\n\r\n");
      if (header === -1) return;
      const length = Number(/Content-Length: (\d+)/i.exec(buffer.subarray(0, header).toString())[1]);
      if (buffer.length < header + 4 + length) return;
      const message = JSON.parse(buffer.subarray(header + 4, header + 4 + length).toString());
      buffer = buffer.subarray(header + 4 + length);
      const next = waiting.shift();
      if (next) next(message);
      else arrived.push(message);
    }
  });
  const send = (message) => {
    const body = JSON.stringify({ jsonrpc: "2.0", ...message });
    child.stdin.write(`Content-Length: ${Buffer.byteLength(body)}\r\n\r\n${body}`);
  };
  const receive = () => new Promise((resolve) => (arrived.length ? resolve(arrived.shift()) : waiting.push(resolve)));
  let id = 0;
  const request = async (method, params) => {
    send({ id: ++id, method, params });
    const reply = await receive();
    assert.equal(reply.id, id);
    return reply.result;
  };
  const exited = new Promise((resolve) => child.on("exit", resolve));
  return { send, receive, request, exited, kill: () => child.kill() };
}


const APP = `app = lambda R, S, T
    {R = [], S = T} or
    {R = [X|L], T = [X|N], app(L, S, N)}
app([1], U, [1,2])
U = [3]
`;

/** Semantic tokens as [line, column, length, type name]. */
function decode(data, legend) {
  const out = [];
  let line = 0;
  let column = 0;
  for (let i = 0; i < data.length; i += 5) {
    line += data[i];
    column = data[i] ? data[i + 1] : column + data[i + 1];
    out.push([line, column, data[i + 2], legend[data[i + 3]]]);
  }
  return out;
}

for (const [kind, command, args] of servers) {
  test(`the ${kind} language server`, async () => {
    const s = connect(command, args);
    try {
    const init = await s.request("initialize", { capabilities: {} });
    const legend = init.capabilities.semanticTokensProvider.legend.tokenTypes;
    assert.equal(init.serverInfo.name, "absys");
    s.send({ method: "initialized", params: {} });

    const uri = "file:///append.abs";
    s.send({ method: "textDocument/didOpen", params: { textDocument: { uri, languageId: "absys", version: 1, text: APP } } });
    const published = await s.receive();
    assert.equal(published.method, "textDocument/publishDiagnostics");
    const problems = published.params.diagnostics;
    assert.equal(problems.length, 1, JSON.stringify(problems));
    assert.equal(problems[0].range.start.line, 4, "the contradiction is on the line of U = [3]");
    assert.match(problems[0].message, /no derivation survives/);

    const hover = await s.request("textDocument/hover", { textDocument: { uri }, position: { line: 3, character: 9 } });
    assert.match(hover.contents.value, /U = \[2\]/, "hover on the goal shows the state after it");
    const tokens = decode((await s.request("textDocument/semanticTokens/full", { textDocument: { uri } })).data, legend);
    assert.deepEqual(tokens[0], [0, 0, 3, "function"], "app, defined, is a function");
    assert.ok(tokens.some((t) => t[3] === "keyword"));
    const symbols = await s.request("textDocument/documentSymbol", { textDocument: { uri } });
    assert.deepEqual(symbols.map((x) => x.name), ["app"]);
    const items = await s.request("textDocument/completion", { textDocument: { uri }, position: { line: 4, character: 0 } });
    assert.ok(items.some((i) => i.label === "app") && items.some((i) => i.label === "lambda"));

    // the 1969 notation, chosen by the file name
    const one = "file:///x.ab1";
    s.send({ method: "textDocument/didOpen", params: { textDocument: { uri: one, languageId: "absys", version: 1, text: "x=y   y = 2\n3 = x\n" } } });
    const p = (await s.receive()).params.diagnostics;
    assert.equal(p.length, 1);
    assert.equal(p[0].range.start.line, 1);

    await s.request("shutdown");
    s.send({ method: "exit" });
    assert.equal(await s.exited, 0);
    } catch (e) {
      s.kill();
      throw e;
    }
  });
}
