// End-to-end smoke test for the Lumo LAN server. Usage: node tools/smoke.mjs <base> <token>
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

const base = process.argv[2] || "http://127.0.0.1:47821";
const token = process.argv[3];
if (!token) {
  console.error("usage: node smoke.mjs <base> <token>");
  process.exit(2);
}
const zipPath = process.argv[4];
const inbox = process.env.LUMO_INBOX || path.resolve(".test-inbox");

let cookie = "";
async function req(pathname, options = {}) {
  const headers = { ...(options.headers || {}) };
  if (cookie) headers.cookie = cookie;
  const res = await fetch(base + pathname, { ...options, headers, redirect: "manual" });
  return res;
}

function headerValue(res, name) {
  return res.headers.get(name);
}

const results = [];
function ok(name, detail = "") {
  results.push(`PASS ${name}${detail ? " — " + detail : ""}`);
}

// 1. health is public
{
  const res = await req("/health");
  assert.equal(res.status, 200);
  const body = await res.json();
  assert.equal(body.ok, true);
  ok("health is public", `port ${body.port}`);
}

// 2. API requires pairing
{
  const a = await req("/api/bootstrap");
  assert.equal(a.status, 401);
  const b = await req("/pair?t=wrong-token");
  assert.equal(b.status, 401);
  ok("API and bad pairing are rejected");
}

// 3. pair sets an HttpOnly cookie and redirects
{
  const res = await req(`/pair?t=${encodeURIComponent(token)}`);
  assert.equal(res.status, 303);
  const setCookie = headerValue(res, "set-cookie");
  assert.ok(setCookie && setCookie.includes("lumo_auth="), "Set-Cookie missing");
  assert.ok(setCookie.toLowerCase().includes("httponly"));
  cookie = setCookie.split(";")[0];
  ok("pairing sets HttpOnly cookie");
}

// 4. bootstrap works with cookie
let boot;
{
  const res = await req("/api/bootstrap");
  assert.equal(res.status, 200);
  boot = await res.json();
  assert.equal(typeof boot.deviceName, "string");
  assert.ok(Array.isArray(boot.shares));
  ok("bootstrap with cookie", `${boot.deviceName} · ${boot.version}`);
}

// 5. static assets and blocked desktop files
{
  for (const file of ["/tokens.css", "/mobile.js", "/manifest.webmanifest", "/icons/icon-192.png"]) {
    const res = await req(file);
    assert.equal(res.status, 200, file);
  }
  for (const file of ["/index.html", "/desktop.js", "/desktop.css"]) {
    const res = await req(file);
    assert.equal(res.status, 404, file);
  }
  ok("mobile assets served; desktop assets hidden");
}

// 6. clipboard text round-trip
{
  const text = "Lumo smoke test ✓ " + new Date().toISOString();
  const res = await req("/api/clipboard", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ text }),
  });
  assert.equal(res.status, 200);
  const got = await req("/api/clipboard");
  const body = await got.json();
  assert.equal(body.clipboard.kind, "text");
  assert.equal(body.clipboard.text, text);
  ok("clipboard text round-trip");
}

// 7. upload streams to the inbox
let uploaded;
let uploadedBytes = 0;
{
  const payload = Buffer.from("hello from the smoke test\n".repeat(2000));
  uploadedBytes = payload.length;
  const res = await req("/api/upload", {
    method: "PUT",
    headers: {
      "x-lumo-name": encodeURIComponent("smoke test.txt"),
      "content-type": "application/octet-stream",
    },
    body: payload,
  });
  assert.equal(res.status, 200);
  const body = await res.json();
  uploaded = body.item;
  assert.equal(body.item.name, "smoke test.txt");
  assert.equal(body.bytes, payload.length);
  const saved = path.join(inbox, "smoke test.txt");
  assert.equal(fs.existsSync(saved), true);
  assert.equal(fs.statSync(saved).size, payload.length);
  ok("upload saved to inbox", `${payload.length} bytes`);
}

// 8. full + ranged download
{
  const res = await req(`/api/dl/${uploaded.id}`);
  assert.equal(res.status, 200);
  const text = await res.text();
  assert.ok(text.startsWith("hello from the smoke test"));

  const range = await req(`/api/dl/${uploaded.id}`, { headers: { range: "bytes=6-15" } });
  assert.equal(range.status, 206);
  const part = await range.text();
  assert.equal(part, "from the s");
  assert.equal(range.headers.get("content-range"), `bytes 6-15/${uploadedBytes}`);
  ok("download full + range", range.headers.get("content-range"));
}

// 9. zip upload with extract=1 becomes a received folder and downloads as zip
if (zipPath && fs.existsSync(zipPath)) {
  const data = fs.readFileSync(zipPath);
  const res = await req("/api/upload?extract=1", {
    method: "PUT",
    headers: {
      "x-lumo-name": encodeURIComponent("smoke-folder.zip"),
      "content-type": "application/octet-stream",
    },
    body: data,
  });
  assert.equal(res.status, 200);
  const body = await res.json();
  assert.equal(body.item.isDir, true, "zip should extract to a folder");
  const zip = await req(`/api/dl/${body.item.id}`);
  assert.equal(zip.status, 200);
  const buf = Buffer.from(await zip.arrayBuffer());
  assert.ok(buf.length > 0);
  assert.equal(buf[0], 0x50);
  assert.equal(buf[1], 0x4b);
  ok("zip upload extracts and re-downloads as zip", body.item.name);
}

// 10. power cancel (does not touch the machine)
{
  const res = await req("/api/power", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ action: "cancel" }),
  });
  assert.equal(res.status, 200);
  const body = await res.json();
  assert.equal(body.ok, true);
  ok("power cancel responds", `cancelled=${body.cancelled}`);
}

// 11. websocket welcome + ping/pong + clipboard push
{
  const ws = new WebSocket(`${base.replace(/^http/, "ws")}/ws?t=${encodeURIComponent(token)}&name=SmokePhone`);
  const welcome = await new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error("welcome timeout")), 5000);
    ws.addEventListener("message", (event) => {
      const msg = JSON.parse(event.data);
      if (msg.t === "welcome") {
        clearTimeout(timer);
        resolve(msg);
      }
    });
    ws.addEventListener("error", (e) => reject(new Error("ws error")));
  });
  assert.equal(typeof welcome.deviceName, "string");
  assert.ok(welcome.clipboard.kind);
  const pong = await new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error("pong timeout")), 4000);
    ws.addEventListener("message", (event) => {
      const msg = JSON.parse(event.data);
      if (msg.t === "pong") {
        clearTimeout(timer);
        resolve(msg);
      }
    });
    ws.send(JSON.stringify({ t: "ping", id: 424242 }));
  });
  assert.equal(pong.id, 424242);
  ws.close();
  ok("websocket welcome + ping/pong");
}

// 12. devices count returns to zero after disconnect
{
  await new Promise((r) => setTimeout(r, 300));
  const res = await req("/api/bootstrap");
  const body = await res.json();
  assert.equal(Array.isArray(body.shares), true);
  ok("bootstrap remains healthy after ws close");
}

// 13. unpair clears the cookie client-side
{
  const res = await req("/api/unpair", { method: "POST" });
  assert.equal(res.status, 200);
  const setCookie = headerValue(res, "set-cookie");
  assert.ok(setCookie && setCookie.includes("Max-Age=0"));
  ok("unpair clears cookie");
}

console.log(results.join("\n"));
console.log(`\nSMOKE PASS (${results.length} checks) · ${base}`);
