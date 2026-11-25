// The Absys extension: starts the language server (`absys lsp`) and adds
// commands that run the text in the editor: its derivations at the end, the
// state after each assertion, or every step.
"use strict";

const vscode = require("vscode");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { LanguageClient, TransportKind } = require("vscode-languageclient/node");

/** @type {LanguageClient | undefined} */
let client;

function setting(name) {
  const value = vscode.workspace.getConfiguration("absys").get(name);
  return typeof value === "string" ? value.trim() : value;
}

async function startServer(context) {
  const custom = setting("path");
  // Built in: absys compiled to WebAssembly, run by the editor's own Node,
  // so nothing has to be installed.
  const builtIn = { module: context.asAbsolutePath(path.join("server", "lsp.js")), transport: TransportKind.stdio };
  const serverOptions = custom ? { command: custom, args: ["lsp"] } : { run: builtIn, debug: builtIn };
  client = new LanguageClient("absys", "Absys", serverOptions, {
    documentSelector: [{ language: "absys" }],
  });
  await client.start();
}

async function restartServer(context) {
  if (client) await client.stop().catch(() => {});
  client = undefined;
  await startServer(context);
}

/** Runs absys as a task in a terminal of its own. */
async function runTask(context, args, title, file) {
  const custom = setting("path");
  const all = [...args, "--search", setting("search") || "fair", file];
  const execution = custom
    ? new vscode.ProcessExecution(custom, all)
    : new vscode.ProcessExecution(process.execPath, [context.asAbsolutePath(path.join("server", "run.js")), ...all], {
        env: { ELECTRON_RUN_AS_NODE: "1" },
      });
  const task = new vscode.Task({ type: "absys" }, vscode.TaskScope.Workspace, `${title} ${path.basename(file)}`, "absys", execution, []);
  task.presentationOptions = {
    reveal: vscode.TaskRevealKind.Always,
    panel: vscode.TaskPanelKind.Dedicated,
    focus: true,
    clear: true,
    echo: false,
    showReuseMessage: false,
  };
  await vscode.tasks.executeTask(task);
}

async function runText(context, args, title) {
  const editor = vscode.window.activeTextEditor;
  if (!editor || editor.document.languageId !== "absys") {
    vscode.window.showInformationMessage("Open an Absys text (.abs, or .ab1 for the 1969 notation) to run it.");
    return;
  }
  const document = editor.document;
  let file = document.uri.fsPath;
  if (document.isUntitled || document.uri.scheme !== "file") {
    // Not on disk yet: run a copy.
    file = path.join(os.tmpdir(), "untitled.abs");
    fs.writeFileSync(file, document.getText());
  } else if (document.isDirty) {
    await document.save();
  }
  await runTask(context, args, title, file);
}

async function activate(context) {
  context.subscriptions.push(
    vscode.commands.registerCommand("absys.run", () => runText(context, ["--env"], "Run")),
    vscode.commands.registerCommand("absys.each", () => runText(context, ["--each"], "Each assertion")),
    vscode.commands.registerCommand("absys.trace", () => runText(context, ["--trace"], "Trace")),
    vscode.commands.registerCommand("absys.restartServer", () => restartServer(context)),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration("absys.path")) restartServer(context);
    })
  );
  await startServer(context);
}

function deactivate() {
  return client ? client.stop() : undefined;
}

module.exports = { activate, deactivate };
