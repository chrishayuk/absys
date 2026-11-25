// Runs inside VS Code. Exercises the extension the way a person would: open
// an example, hover, see its problems, outline and colours, and
// run it.
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const vscode = require("vscode");

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

async function until(what, probe, seconds = 20) {
  for (let i = 0; i < seconds * 10; i++) {
    const value = await probe();
    if (value) return value;
    await sleep(100);
  }
  throw new Error(`timed out waiting for ${what}`);
}

const hoverText = (hovers) =>
  hovers.flatMap((h) => h.contents).map((c) => (typeof c === "string" ? c : c.value)).join("\n");

/** Runs a command that starts an absys task and returns the task's exit code. */
async function taskExit(command) {
  const finished = new Promise((resolve) => {
    const listener = vscode.tasks.onDidEndTaskProcess((event) => {
      if (event.execution.task.definition.type === "absys") {
        listener.dispose();
        resolve(event.exitCode);
      }
    });
  });
  await vscode.commands.executeCommand(command);
  return Promise.race([finished, sleep(20000).then(() => "timed out")]);
}

exports.run = async function run() {
  const folder = vscode.workspace.workspaceFolders[0].uri.fsPath;
  const document = await vscode.workspace.openTextDocument(path.join(folder, "reverse.abs"));
  await vscode.window.showTextDocument(document);
  assert.equal(document.languageId, "absys", "the .abs extension selects the language");

  const line = (text) => {
    for (let i = 0; i < document.lineCount; i++) if (document.lineAt(i).text.startsWith(text)) return i;
    throw new Error(`no line ${text}`);
  };

  // The server is ready when it answers a hover. On the goal: the state after it.
  const goal = new vscode.Position(line("rev([1,2,U]"), 2);
  const hovers = await until("a hover from the language server", async () => {
    const found = await vscode.commands.executeCommand("vscode.executeHoverProvider", document.uri, goal);
    return found.length ? found : undefined;
  });
  assert.match(hoverText(hovers), /U = X₃/);
  const later = await vscode.commands.executeCommand("vscode.executeHoverProvider", document.uri, new vscode.Position(line("rev([5]"), 2));
  assert.match(hoverText(later), /U = 5/);
  console.log("hover ok");

  const legend = await vscode.commands.executeCommand("vscode.provideDocumentSemanticTokensLegend", document.uri);
  assert.deepEqual(legend.tokenTypes, ["comment", "string", "keyword", "number", "operator", "variable", "function", "macro"]);
  const tokens = await vscode.commands.executeCommand("vscode.provideDocumentSemanticTokens", document.uri);
  assert.ok(tokens.data.length > 0);
  const symbols = await vscode.commands.executeCommand("vscode.executeDocumentSymbolProvider", document.uri);
  assert.deepEqual(symbols.map((s) => s.name), ["rev1", "rev"]);
  console.log("symbols and semantic tokens ok");

  // A text that becomes unsatisfiable, and a line not in the notation.
  const broken = await vscode.workspace.openTextDocument({ language: "absys", content: "X = Y\nY = 2\nX = 3\nZ = Z +\n" });
  await vscode.window.showTextDocument(broken);
  const problems = await until("diagnostics", () => {
    const found = vscode.languages.getDiagnostics(broken.uri);
    return found.length >= 2 ? found : undefined;
  });
  const warning = problems.find((p) => p.severity === vscode.DiagnosticSeverity.Warning);
  assert.match(warning.message, /no derivation survives/);
  assert.equal(warning.range.start.line, 2);
  const error = problems.find((p) => p.severity === vscode.DiagnosticSeverity.Error);
  assert.equal(error.source, "absys");
  assert.equal(error.range.start.line, 3);
  console.log("diagnostics ok");

  // Run This Text, each assertion, and every step: tasks that end with exit code 0.
  await vscode.window.showTextDocument(document);
  assert.equal(await taskExit("absys.run"), 0);
  console.log("run ok");
  assert.equal(await taskExit("absys.each"), 0);
  console.log("each ok");
  assert.equal(await taskExit("absys.trace"), 0);
  console.log("trace ok");
};
