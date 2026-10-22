import { createServer } from "node:http";
import { createHash } from "node:crypto";
import { spawn } from "node:child_process";
import { mkdir, readFile } from "node:fs/promises";
import path from "node:path";
import assert from "node:assert/strict";
import { root, id, destination } from "./build-plugin.mjs";

const profile = process.argv[2] ?? "debug";
assert.ok(["debug", "release"].includes(profile));
const runtime = path.join(root, "target", "smoke-runtime");
await mkdir(runtime, { recursive: true });
const server = createServer();
let socket, child, registered = false;
let resolveMessage, rejectMessage;
const completed = new Promise((resolve, reject) => { resolveMessage = resolve; rejectMessage = reject; });
function send(payload) {
  const bytes = Buffer.from(JSON.stringify(payload));
  const header = Buffer.alloc(bytes.length < 126 ? 2 : 4);
  header[0] = 0x81;
  if (bytes.length < 126) header[1] = bytes.length;
  else { header[1] = 126; header.writeUInt16BE(bytes.length, 2); }
  socket.write(Buffer.concat([header, bytes]));
}
server.on("upgrade", (request, connection, head) => {
  socket = connection;
  const accept = createHash("sha1").update(request.headers["sec-websocket-key"] + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11").digest("base64");
  connection.write(`HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: ${accept}\r\n\r\n`);
  let buffer = head;
  connection.on("error", rejectMessage);
  connection.on("data", chunk => {
    try {
      buffer = Buffer.concat([buffer, chunk]);
      while (buffer.length >= 2) {
        const opcode = buffer[0] & 15;
        let length = buffer[1] & 127, offset = 2;
        if (length === 126) { if (buffer.length < 4) return; length = buffer.readUInt16BE(2); offset = 4; }
        if (length === 127) { if (buffer.length < 10) return; length = Number(buffer.readBigUInt64BE(2)); offset = 10; }
        assert.ok(length < 4 * 1024 * 1024);
        const masked = !!(buffer[1] & 128);
        const total = offset + (masked ? 4 : 0) + length;
        if (buffer.length < total) return;
        const mask = masked ? buffer.subarray(offset, offset + 4) : null;
        const payload = Buffer.from(buffer.subarray(offset + (masked ? 4 : 0), total));
        if (mask) for (let i = 0; i < payload.length; i++) payload[i] ^= mask[i % 4];
        buffer = buffer.subarray(total);
        if (opcode === 8) continue;
        assert.equal(opcode, 1);
        const message = JSON.parse(payload.toString());
        if (!registered) {
          assert.deepEqual(message, { event: "registerPlugin", uuid: "smoke-instance" });
          registered = true;
          send({ event: "willAppear", action: `${id}.gain-control`, context: "smoke-action", device: "test", payload: { controller: "Encoder", settings: { channels: "invalid" } } });
          send({ event: "propertyInspectorDidAppear", action: `${id}.gain-control`, context: "smoke-action", device: "test", payload: {} });
        } else if (message.event === "sendToPropertyInspector" && message.payload?.error?.code === "invalidSettings") resolveMessage();
      }
    } catch (error) { rejectMessage(error); }
  });
});
await new Promise(resolve => server.listen(0, "127.0.0.1", resolve));
const timer = setTimeout(() => rejectMessage(new Error("Native registration timed out")), 10000);
try {
  child = spawn(path.join(root, "target", profile, "streamdeck-voicemeeter.exe"), ["-port", String(server.address().port), "-pluginUUID", "smoke-instance", "-registerEvent", "registerPlugin", "-info", "{}"], {
    cwd: profile === "release" ? destination : root, windowsHide: true, env: { ...process.env, APPDATA: runtime, LOCALAPPDATA: runtime, RUST_LOG: "info" }, stdio: ["ignore", "pipe", "pipe"],
  });
  let stderr = "";
  child.stderr.on("data", data => { stderr += data; });
  child.on("error", rejectMessage);
  const exited = new Promise(resolve => child.on("exit", (code, signal) => {
    if (!registered) rejectMessage(new Error(`Native process exited before registration: ${code}/${signal}: ${stderr}`));
    resolve(code);
  }));
  await completed;
  socket.end(Buffer.from([0x88, 0]));
  const code = await Promise.race([exited, new Promise((_, reject) => setTimeout(() => reject(new Error("Native shutdown timed out")), 5000).unref())]);
  assert.equal(code, 0, stderr);
  const log = await readFile(path.join(runtime, id, "logs/plugin.log"), "utf8");
  assert.match(log, /Plugin starting/);
  assert.match(log, /Plugin stopped/);
  console.log("Native EXE registered over WebSocket, rejected malformed settings and shut down cleanly with logs.");
} finally {
  clearTimeout(timer);
  if (child && child.exitCode === null) child.kill();
  socket?.destroy();
  server.close();
}
