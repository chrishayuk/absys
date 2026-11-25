// Drives the playground in headless Chrome over the DevTools protocol:
// loads examples, steps through their assertions, and reads the page.
//   node web/browser-test.mjs [screenshot.png]   (needs Chrome and python3)
import { spawn } from "node:child_process";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";

const root = new URL("..", import.meta.url).pathname;
const chromePath = process.env.CHROME || "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const [port, debug] = [8781, 9343];
const server = spawn("python3", ["-m", "http.server", "-d", root + "web", "--bind", "127.0.0.1", String(port)], { stdio: "ignore" });
const chrome = spawn(chromePath, ["--headless=new", "--disable-gpu", `--remote-debugging-port=${debug}`,
  `--user-data-dir=${mkdtempSync(tmpdir() + "/absys-chrome-")}`, "--window-size=1440,1400", "about:blank"], { stdio: "ignore" });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const quit = (code) => (server.kill(), chrome.kill(), process.exit(code));

let target;
for (let i = 0; i < 50 && !target; i++) {
  await sleep(200);
  try {
    target = (await (await fetch(`http://127.0.0.1:${debug}/json`)).json()).find((t) => t.type === "page");
  } catch {}
}
if (!target) (console.log("Chrome did not start"), quit(2));
const socket = new WebSocket(target.webSocketDebuggerUrl);
await new Promise((r) => (socket.onopen = r));
let id = 0;
const waiting = new Map();
const errors = [];
socket.onmessage = (event) => {
  const m = JSON.parse(event.data);
  if (m.id && waiting.has(m.id)) waiting.get(m.id)(m.result ?? m);
  if (m.method === "Runtime.exceptionThrown") errors.push(m.params.exceptionDetails.exception?.description ?? m.params.exceptionDetails.text);
};
const send = (method, params = {}) => new Promise((r) => (waiting.set(++id, r), socket.send(JSON.stringify({ id, method, params }))));
const js = async (expression) => {
  const r = await send("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true });
  if (r.exceptionDetails) throw new Error(r.exceptionDetails.exception?.description ?? r.exceptionDetails.text);
  return r.result.value;
};
const until = async (expression, what) => {
  for (let i = 0; i < 100; i++) {
    if (await js(expression)) return;
    await sleep(100);
  }
  throw new Error(`timed out: ${what}`);
};
const open = async (example) => {
  await send("Page.navigate", { url: `http://127.0.0.1:${port}/index.html?example=${example}` });
  await until(`document.querySelectorAll(".ab-timeline li").length > 0 && !document.querySelector(".ab-run").disabled`, `${example} runs`);
};
const step = (i) => js(`document.querySelectorAll(".ab-timeline li")[${i}].click()`);
const cards = () => js(`[...document.querySelectorAll(".ab-card")].map((c) => c.textContent)`);
const chips = () => js(`[...document.querySelectorAll(".ab-card .chip")].map((c) => c.textContent)`);

let checked = 0, failed = 0;
const check = (ok, what) => (checked++, ok || (failed++, console.log(`FAIL ${what}`)));

try {
  await send("Runtime.enable");
  await send("Page.enable");

  await open("reverse");
  check((await js(`document.querySelectorAll(".ab-timeline li").length`)) === 4, "reverse: four assertions");
  await step(2);
  check((await chips()).sort().join(", ") === "U = X₃, X₃ = V", "reverse: U = X₃ and X₃ = V suspended after the goal");
  check((await cards()).join(" ").includes("L= [2,1]") || (await cards()).join(" ").includes("L[2,1]"), "reverse: L = [2,1]");
  check(await js(`document.querySelectorAll(".ab-treelist .t-failed").length === 4`), "reverse: four derivations failed in the tree");
  await step(3);
  check((await chips()).length === 0, "reverse: nothing suspended after rev([5], [V])");
  check(await js(`[...document.querySelectorAll(".ab-ans tr.changed th")].map((t) => t.textContent).includes("U")`), "reverse: U shown as changed");
  check((await js(`document.querySelector(".ab-paint .ab-band") !== null`)), "the assertion shown is marked in the text");

  await open("assertions-not-assignments");
  await step(4);
  check(await js(`document.querySelector(".ab-summary").classList.contains("dead")`), "X = 17: no derivation survives");

  await open("triples");
  const last = await js(`document.querySelectorAll(".ab-timeline li").length - 1`);
  await step(last);
  check((await cards()).length === 3, "triples: three conjunctions");

  // options: fifo breaks setdiff
  await open("setdiff");
  const toEnd = `document.querySelectorAll(".ab-timeline li")[document.querySelectorAll(".ab-timeline li").length - 1].click()`;
  await js(toEnd);
  check((await cards()).join(" ").includes("[1,3]"), "setdiff: D = [1,3]");
  await js(`(() => { const s = document.querySelectorAll(".ab-opts select")[1]; s.value = "fifo"; s.dispatchEvent(new Event("change")); })()`);
  await sleep(300);
  await until(`!document.querySelector(".ab-run").disabled`, "rerun");
  await js(toEnd);
  check((await cards()).join(" ").includes("[1]") && !(await cards()).join(" ").includes("[1,3]"), "setdiff, fifo: D = [1]");

  await open("reverse");
  await step(2);
  if (process.argv[2]) {
    const shot = await send("Page.captureScreenshot", { format: "png", captureBeyondViewport: true });
    writeFileSync(process.argv[2], Buffer.from(shot.data, "base64"));
  }
  check(errors.length === 0, `no errors in the page: ${errors.join("; ")}`);
} catch (e) {
  failed++;
  console.log(`FAIL ${e.message}`);
}
console.log(`${checked} browser checks, ${failed ? failed + " failed" : "all passed"}`);
quit(failed ? 1 : 0);
