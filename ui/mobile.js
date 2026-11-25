/* Lumo mobile client — touchpad, files, clipboard, power. No dependencies. */
(() => {
  "use strict";

  const $ = (sel, root = document) => root.querySelector(sel);
  const $$ = (sel, root = document) => Array.from(root.querySelectorAll(sel));

  const ICONS = {
    settings:
      '<path d="M4 6h10"/><path d="M18 6h2"/><circle cx="16" cy="6" r="2"/><path d="M4 12h2"/><path d="M10 12h10"/><circle cx="8" cy="12" r="2"/><path d="M4 18h10"/><path d="M18 18h2"/><circle cx="16" cy="18" r="2"/>',
    hand:
      '<path d="M18 11V6a2 2 0 0 0-4 0v5"/><path d="M14 10V4a2 2 0 0 0-4 0v6"/><path d="M10 10.5V6a2 2 0 0 0-4 0v8"/><path d="M18 8a2 2 0 1 1 4 0v6a8 8 0 0 1-8 8h-2c-2.8 0-4.5-.86-5.99-2.34l-3.6-3.6a2 2 0 0 1 2.83-2.82L7 15"/>',
    mouse:
      '<rect x="6" y="2" width="12" height="20" rx="6"/><path d="M12 2v6"/>',
    "mouse-pointer": '<path d="m3 3 7.07 16.97 2.51-7.39 7.39-2.51L3 3Z"/>',
    "mouse-right":
      '<rect x="6" y="2" width="12" height="20" rx="6"/><path d="M12 2v6"/><path d="M15 6h.01"/>',
    folder:
      '<path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z"/>',
    file:
      '<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z"/><path d="M14 2v6h6"/>',
    image:
      '<rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="9" cy="9" r="2"/><path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"/>',
    film:
      '<rect x="2" y="3" width="20" height="18" rx="2"/><path d="M7 3v18"/><path d="M17 3v18"/><path d="M2 9h5"/><path d="M2 15h5"/><path d="M17 9h5"/><path d="M17 15h5"/>',
    music:
      '<path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/>',
    archive:
      '<rect x="2" y="4" width="20" height="5" rx="1"/><path d="M4 9v10a1 1 0 0 0 1 1h14a1 1 0 0 0 1-1V9"/><path d="M10 13h4"/>',
    text:
      '<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z"/><path d="M14 2v6h6"/><path d="M8 13h8"/><path d="M8 17h6"/>',
    download:
      '<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><path d="M7 10l5 5 5-5"/><path d="M12 15V3"/>',
    upload:
      '<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><path d="M7 8l5-5 5 5"/><path d="M12 3v12"/>',
    copy:
      '<rect x="9" y="9" width="12" height="12" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>',
    clipboard:
      '<rect x="8" y="2" width="8" height="4" rx="1"/><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/>',
    lock:
      '<rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/>',
    moon: '<path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"/>',
    rotate:
      '<path d="M21 12a9 9 0 1 1-3-6.7L21 8"/><path d="M21 3v5h-5"/>',
    power:
      '<path d="M18.36 6.64a9 9 0 1 1-12.73 0"/><path d="M12 2v10"/>',
    x: '<path d="M18 6 6 18"/><path d="m6 6 12 12"/>',
    check: '<path d="M20 6 9 17l-5-5"/>',
    refresh:
      '<path d="M21 12a9 9 0 0 1-9 9 9 9 0 0 1-8.2-5.3"/><path d="M3 12a9 9 0 0 1 9-9 9 9 0 0 1 8.2 5.3"/><path d="M21 4v4h-4"/><path d="M3 20v-4h4"/>',
    send:
      '<path d="m22 2-7 20-4-9-9-4Z"/><path d="M22 2 11 13"/>',
    trash:
      '<path d="M3 6h18"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/>',
    keyboard:
      '<rect x="2" y="6" width="20" height="12" rx="2"/><path d="M6 10h.01"/><path d="M10 10h.01"/><path d="M14 10h.01"/><path d="M18 10h.01"/><path d="M7 14h10"/>',
    play: '<path d="m9 18 12-7-12-7v14Z" fill="currentColor" stroke="none"/>',
    pause: '<path d="M9 4v16" stroke-width="2.4"/><path d="M15 4v16" stroke-width="2.4"/>',
    prev: '<path d="M19 20 9 12l10-8v16Z" fill="currentColor" stroke="none"/><path d="M5 19V5"/>',
    next: '<path d="M5 4l10 8-10 8V4Z" fill="currentColor" stroke="none"/><path d="M19 5v14"/>',
    shield:
      '<path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10Z"/><path d="m9 12 2 2 4-4"/>',
    "link-off":
      '<path d="M9 17H7A5 5 0 0 1 7 7h2"/><path d="M15 7h2a5 5 0 0 1 4 8"/><path d="M8 12h4"/><path d="m2 2 20 20"/>',
    "chevron-right": '<path d="m9 18 6-6-6-6"/>',
    external:
      '<path d="M15 3h6v6"/><path d="M10 14 21 3"/><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>',
    plus: '<path d="M12 5v14"/><path d="M5 12h14"/>',
    minus: '<path d="M5 12h14"/>',
    wifi:
      '<path d="M5 12.55a11 11 0 0 1 14.08 0"/><path d="M1.42 9a16 16 0 0 1 21.16 0"/><path d="M8.53 16.11a6 6 0 0 1 6.95 0"/><path d="M12 20h.01"/>',
    smartphone:
      '<rect x="5" y="2" width="14" height="20" rx="2"/><path d="M12 18h.01"/>',
  };

  function icon(name, size = 20) {
    const body = ICONS[name] || ICONS.file;
    return `<svg viewBox="0 0 24 24" width="${size}" height="${size}" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false">${body}</svg>`;
  }

  function hydrateIcons(root = document) {
    $$("[data-icon]", root).forEach((el) => {
      if (el.dataset.iconReady) return;
      el.innerHTML = icon(el.dataset.icon, Number(el.dataset.size) || 20);
      el.dataset.iconReady = "1";
    });
  }

  function esc(value) {
    return String(value ?? "")
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#39;");
  }

  function bytes(n) {
    const value = Number(n) || 0;
    if (value <= 0) return "0 B";
    const units = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.min(units.length - 1, Math.floor(Math.log(value) / Math.log(1024)));
    const scaled = value / Math.pow(1024, i);
    return `${scaled >= 10 || i === 0 ? Math.round(scaled) : scaled.toFixed(1)} ${units[i]}`;
  }

  function timeAgo(epochSeconds) {
    if (!epochSeconds) return "just now";
    const diff = Math.max(0, Math.floor(Date.now() / 1000 - epochSeconds));
    if (diff < 10) return "just now";
    if (diff < 60) return `${diff}s ago`;
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    return `${Math.floor(diff / 86400)}d ago`;
  }

  const KIND_LABEL = {
    folder: "Folder",
    image: "Image",
    video: "Video",
    audio: "Audio",
    archive: "Archive",
    text: "Text",
    pdf: "PDF",
    document: "Document",
    sheet: "Spreadsheet",
    slides: "Slides",
    file: "File",
  };

  function kindIcon(kind) {
    switch (kind) {
      case "folder":
        return icon("folder", 19);
      case "image":
        return icon("image", 19);
      case "video":
        return icon("film", 19);
      case "audio":
        return icon("music", 19);
      case "archive":
        return icon("archive", 19);
      case "text":
        return icon("text", 19);
      default:
        return icon("file", 19);
    }
  }

  function readAuthToken() {
    try {
      const fromUrl = new URLSearchParams(location.search).get("t") || "";
      if (fromUrl.length >= 16) {
        localStorage.setItem("lumo.token", fromUrl);
        return fromUrl;
      }
      return localStorage.getItem("lumo.token") || "";
    } catch (_) {
      return "";
    }
  }

  let AUTH_TOKEN = readAuthToken();

  function dlUrl(id, inline = false) {
    const base = `/api/dl/${encodeURIComponent(id)}${inline ? "?inline=1" : ""}`;
    if (!AUTH_TOKEN) return base;
    return `${base}${inline ? "&" : "?"}t=${encodeURIComponent(AUTH_TOKEN)}`;
  }

  const state = {
    boot: null,
    ws: null,
    wsReady: false,
    reconnectAttempt: 0,
    reconnectTimer: null,
    pingTimer: null,
    pingId: 0,
    latency: null,
    view: "pad",
    shares: [],
    received: [],
    clipboard: { kind: "empty", files: [], at: 0 },
    history: [],
    pending: null,
    clockSkew: 0,
    uploads: 0,
    pad: { speed: 1.2, natural: true, inertia: true },
    media: {
      active: false,
      title: "",
      artist: "",
      album: "",
      app: "",
      playing: false,
      position: 0,
      duration: 0,
      art: null,
    },
    mediaBase: { position: 0, at: 0 },
    mediaOpen: false,
  };

  /* ---------------- Theme ---------------- */

  const themeMedia = window.matchMedia("(prefers-color-scheme: dark)");

  function resolveTheme(choice) {
    return choice === "system" ? (themeMedia.matches ? "dark" : "light") : choice;
  }

  function applyTheme(choice) {
    const resolved = resolveTheme(choice);
    document.documentElement.setAttribute("data-theme", resolved);
    const meta = $('meta[name="theme-color"]');
    if (meta) meta.content = resolved === "dark" ? "#000000" : "#FFFFFF";
    $$("[data-theme-choice]").forEach((btn) => {
      btn.setAttribute("aria-selected", btn.dataset.themeChoice === choice ? "true" : "false");
    });
  }

  function currentThemeChoice() {
    return localStorage.getItem("lumo.theme") || "dark";
  }

  themeMedia.addEventListener("change", () => {
    if (currentThemeChoice() === "system") applyTheme("system");
  });

  /* ---------------- API ---------------- */

  class ApiError extends Error {
    constructor(status, message) {
      super(message);
      this.status = status;
    }
  }

  async function api(path, options = {}) {
    const headers = new Headers(options.headers || {});
    if (AUTH_TOKEN) headers.set("X-Lumo-Token", AUTH_TOKEN);
    let res;
    try {
      res = await fetch(path, { ...options, credentials: "same-origin", headers });
    } catch (err) {
      throw new ApiError(0, "Network unavailable");
    }
    if (res.status === 401) {
      showPairing();
      throw new ApiError(401, "Pairing required");
    }
    if (!res.ok) {
      let message = res.statusText || `HTTP ${res.status}`;
      try {
        const body = await res.json();
        if (body && body.error) message = body.error;
      } catch (_) {}
      throw new ApiError(res.status, message);
    }
    const type = res.headers.get("content-type") || "";
    return type.includes("application/json") ? res.json() : res.text();
  }

  function showPairing() {
    AUTH_TOKEN = "";
    try {
      localStorage.removeItem("lumo.token");
      if (location.search) history.replaceState({}, "", location.pathname);
    } catch (_) {}
    $("#app").hidden = true;
    $("#pairing").hidden = false;
    closeSheet();
  }

  function hidePairing() {
    $("#pairing").hidden = true;
    $("#app").hidden = false;
  }

  /* ---------------- Toasts ---------------- */

  function toast(message, tone = "neutral", timeout = 3200) {
    const stack = $("#toasts");
    const el = document.createElement("div");
    el.className = `toast ${tone}`;
    el.innerHTML = `<span class="dot" aria-hidden="true"></span><span>${esc(message)}</span>`;
    stack.appendChild(el);
    const remove = () => {
      el.classList.add("hide");
      setTimeout(() => el.remove(), 200);
    };
    setTimeout(remove, timeout);
    el.addEventListener("click", remove);
  }

  /* ---------------- Modal ---------------- */

  let modalResolver = null;

  function confirmModal({ title, body, confirmLabel = "Confirm", tone = "danger" }) {
    $("#modalTitle").textContent = title;
    $("#modalBody").textContent = body;
    const confirm = $("#modalConfirm");
    confirm.textContent = confirmLabel;
    confirm.className = `btn ${tone === "danger" ? "danger" : "primary"}`;
    $("#modalBackdrop").hidden = false;
    confirm.focus({ preventScroll: true });
    return new Promise((resolve) => {
      modalResolver = resolve;
    });
  }

  function settleModal(value) {
    $("#modalBackdrop").hidden = true;
    if (modalResolver) {
      const resolve = modalResolver;
      modalResolver = null;
      resolve(value);
    }
  }

  /* ---------------- Connection ---------------- */

  function phoneName() {
    const ua = navigator.userAgent || "";
    if (/iPad/.test(ua)) return "iPad";
    if (/iPhone/.test(ua)) return "iPhone";
    if (/Android/.test(ua)) return "Android";
    if (/Macintosh/.test(ua)) return "Mac";
    return "Phone";
  }

  function browserName() {
    const ua = navigator.userAgent || "";
    if (/EdgiOS|Edg\//.test(ua)) return "Edge";
    if (/FxiOS|Firefox/.test(ua)) return "Firefox";
    if (/CriOS|Chrome/.test(ua)) return "Chrome";
    if (/Safari/.test(ua)) return "Safari";
    return "Browser";
  }

  function setConnection(status, label) {
    const pill = $("#connPill");
    pill.dataset.state = status;
    $("#connLabel").textContent = label;
    const badge = $("#connBadge");
    if (badge) {
      badge.textContent = label;
      badge.className = `badge ${status === "online" ? "success" : status === "offline" ? "error" : ""}`;
    }
  }

  function wsSend(payload) {
    if (state.ws && state.ws.readyState === WebSocket.OPEN) {
      state.ws.send(JSON.stringify(payload));
      return true;
    }
    return false;
  }

  function connect() {
    clearTimeout(state.reconnectTimer);
    if (state.ws && (state.ws.readyState === WebSocket.OPEN || state.ws.readyState === WebSocket.CONNECTING)) {
      return;
    }
    const protocol = location.protocol === "https:" ? "wss:" : "ws:";
    const url = `${protocol}//${location.host}/ws?name=${encodeURIComponent(phoneName())}${
      AUTH_TOKEN ? `&t=${encodeURIComponent(AUTH_TOKEN)}` : ""
    }`;
    setConnection("connecting", "Connecting");
    let ws;
    try {
      ws = new WebSocket(url);
    } catch (err) {
      scheduleReconnect();
      return;
    }
    state.ws = ws;
    ws.onopen = () => {
      state.reconnectAttempt = 0;
      state.wsReady = true;
      setConnection("online", "Online");
      startPing();
    };
    ws.onmessage = (event) => {
      let msg;
      try {
        msg = JSON.parse(event.data);
      } catch (_) {
        return;
      }
      handleMessage(msg);
    };
    ws.onclose = () => {
      state.wsReady = false;
      state.ws = null;
      stopPing();
      setConnection("offline", "Offline");
      scheduleReconnect();
    };
    ws.onerror = () => ws.close();
  }

  function scheduleReconnect() {
    clearTimeout(state.reconnectTimer);
    const delay = Math.min(10000, 700 * Math.pow(1.6, Math.min(6, state.reconnectAttempt++)));
    state.reconnectTimer = setTimeout(() => {
      if (!document.hidden) connect();
    }, delay);
  }

  function startPing() {
    stopPing();
    state.pingTimer = setInterval(() => {
      state.pingId = Date.now();
      wsSend({ t: "ping", id: state.pingId });
    }, 3000);
  }

  function stopPing() {
    clearInterval(state.pingTimer);
    state.pingTimer = null;
  }

  /* ---------------- Media player ---------------- */

  function formatPlayback(value) {
    const seconds = Math.max(0, Math.floor(Number(value) || 0));
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = seconds % 60;
    return h > 0 ? `${h}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}` : `${m}:${String(s).padStart(2, "0")}`;
  }

  function friendlyApp(value) {
    const app = String(value || "");
    if (!app || /^[0-9a-f]{16}$/i.test(app)) return "";
    const tail = app.split("!").pop().split("\\").pop();
    return tail.replace(/\.exe$/i, "");
  }

  function updateMediaProgress() {
    const media = state.media;
    if (!media.active) return;
    let position = Number(media.position) || 0;
    if (media.playing) position += (performance.now() - state.mediaBase.at) / 1000;
    if (media.duration > 0) position = Math.min(position, media.duration);
    const percent = media.duration > 0 ? Math.max(0, Math.min(100, (position / media.duration) * 100)) : 0;
    $("#mediaBar").style.width = `${percent}%`;
    $("#mediaPos").textContent = formatPlayback(position);
    $("#mediaDur").textContent = media.duration > 0 ? formatPlayback(media.duration) : "—";
  }

  function setMedia(media) {
    if (!media) return;
    const previous = state.media;
    const incoming = Number(media.position) || 0;
    const trackChanged =
      !previous.active ||
      previous.title !== media.title ||
      previous.artist !== media.artist ||
      Math.abs((previous.duration || 0) - (media.duration || 0)) > 0.5;
    let position = incoming;
    // SMTC/POSITION is often only refreshed every few seconds while playing.
    // Keep our locally extrapolated clock instead of snapping backwards on
    // every server update.
    if (!trackChanged && previous.playing && media.playing) {
      const expected =
        state.mediaBase.position + (performance.now() - state.mediaBase.at) / 1000;
      // Ignore small backwards jitter from poll timing, but accept a real seek
      // (arrow key/scrub) that moves the player several seconds back.
      if (position < expected - 0.25 && expected - position < 3) position = expected;
    }
    state.media = { ...previous, ...media, position };
    state.mediaBase = { position, at: performance.now() };
    if (!state.media.active) state.mediaOpen = false;

    renderMediaVisibility();
    if (!state.media.active) return;

    $("#mediaTitle").textContent = state.media.title || "Now playing";
    $("#mediaArtist").textContent = state.media.artist || state.media.album || "Unknown artist";
    $("#mediaApp").textContent = friendlyApp(state.media.app);
    $("#mediaToggleIcon").innerHTML = icon(state.media.playing ? "pause" : "play", 18);
    $("#mediaToggle").setAttribute("aria-label", state.media.playing ? "Pause" : "Play");

    const art = $("#mediaArt");
    const fallback = $(".media-art-fallback");
    if (state.media.art) {
      if (art.getAttribute("src") !== state.media.art) art.src = state.media.art;
      art.hidden = false;
      fallback.hidden = true;
    } else {
      art.hidden = true;
      art.removeAttribute("src");
      fallback.hidden = false;
    }
    updateMediaProgress();
  }

  function renderMediaVisibility() {
    const player = $("#mediaPlayer");
    const trigger = $("#padMedia");
    const visible = state.mediaOpen && !!state.media.active;
    player.classList.toggle("is-open", visible);
    player.setAttribute("aria-hidden", visible ? "false" : "true");
    player.inert = !visible;
    trigger.classList.toggle("is-open", visible);
    trigger.setAttribute("aria-pressed", visible ? "true" : "false");
  }

  function toggleMediaPlayer() {
    if (!state.media.active) {
      toast("No media is playing", "neutral");
      return;
    }
    state.mediaOpen = !state.mediaOpen;
    renderMediaVisibility();
    if (state.mediaOpen) updateMediaProgress();
  }

  function mediaAction(action, delta) {
    const payload = { t: "media", action };
    if (Number.isFinite(delta)) payload.delta = delta;
    if (!wsSend(payload)) {
      toast("Connect to your PC to control media", "error");
    }
  }

  function bindMedia() {
    $("#padMedia").addEventListener("click", toggleMediaPlayer);
    $("#mediaPlayer").addEventListener("click", (event) => {
      const button = event.target.closest("[data-media]");
      if (!button) return;
      const action = button.dataset.media;
      if (action === "seek-back") mediaAction("seek", -10);
      else if (action === "seek-forward") mediaAction("seek", 10);
      else mediaAction(action);
    });
  }

  function handleMessage(msg) {
    switch (msg.t) {
      case "welcome": {
        state.clockSkew = Date.now() - (Number(msg.serverTime) || 0) * 1000;
        if (state.boot) state.boot.deviceName = msg.deviceName;
        $("#pcName").textContent = msg.deviceName || "Your PC";
        $("#connDevice").textContent = msg.deviceName || "Your PC";
        if (msg.clipboard) setClipboard(msg.clipboard, false);
        if (msg.media) setMedia(msg.media);
        state.pending = normalizePending(msg.pendingPower);
        renderPending();
        refreshShares();
        break;
      }
      case "clipboard":
        if (msg.clipboard) setClipboard(msg.clipboard, true);
        break;
      case "media":
        if (msg.media) setMedia(msg.media);
        break;
      case "shared-text": {
        const clip = {
          kind: "text",
          text: msg.text || "",
          files: [],
          at: Math.floor(Date.now() / 1000),
        };
        setClipboard(clip, false);
        pushHistory(clip);
        toast("Text sent from your PC", "success", 4000);
        break;
      }
      case "pong": {
        if (msg.id === state.pingId) state.latency = Math.max(0, Date.now() - msg.id);
        break;
      }
      case "refresh":
        if (msg.scope === "shares") refreshShares();
        if (msg.scope === "received") refreshReceived();
        break;
      case "toast":
        if (msg.message) toast(msg.message, msg.tone || "neutral");
        break;
      case "session-expired":
        showPairing();
        if (state.ws) state.ws.close();
        toast("Pairing was reset on the PC", "error", 5000);
        break;
      default:
        break;
    }
  }

  /* ---------------- Bootstrap ---------------- */

  async function loadBootstrap() {
    try {
      const boot = await api("/api/bootstrap");
      state.boot = boot;
      state.clipboard = boot.clipboard || { kind: "empty", files: [], at: 0 };
      state.shares = boot.shares || [];
      if (boot.media) setMedia(boot.media);
      state.pending = normalizePending(boot.pendingPower);
      $("#pcName").textContent = boot.deviceName || "Your PC";
      $("#connDevice").textContent = boot.deviceName || "Your PC";
      $("#connAddress").textContent = location.host;
      $("#appVersion").textContent = boot.version || "0.1.9";
      $("#thisPhone").textContent = `${phoneName()} · ${browserName()}`;
      hidePairing();
      renderShares();
      renderClipboard();
      renderPending();
      if (!state.ws) connect();
    } catch (err) {
      if (err.status === 401) {
        showPairing();
        return;
      }
      setConnection("offline", "Offline");
      setTimeout(loadBootstrap, 3500);
    }
  }

  function normalizePending(pending) {
    if (!pending) return null;
    const until = Number(pending.executeAt) * 1000 + state.clockSkew;
    if (!Number.isFinite(until) || until <= Date.now()) return null;
    return { action: pending.action, until };
  }

  /* ---------------- Navigation ---------------- */

  function switchView(view) {
    state.view = view;
    document.body.dataset.view = view;
    $$(".view").forEach((el) => {
      el.hidden = el.dataset.view !== view;
    });
    $$(".tab").forEach((tab) => {
      if (tab.dataset.tab === view) tab.setAttribute("aria-current", "page");
      else tab.removeAttribute("aria-current");
    });
    if (view === "files") {
      refreshShares();
      refreshReceived();
    }
    if (view === "clipboard") refreshClipboard();
  }

  /* ---------------- Touchpad ---------------- */

  const padState = {
    pointers: new Map(),
    gesture: null,
    lastTwo: null,
    pending: { x: 0, y: 0 },
    frame: 0,
    inertiaFrame: 0,
    velocity: { x: 0, y: 0 },
    wheelAccum: { x: 0, y: 0 },
    lastTap: 0,
  };

  const trail = {
    canvas: null,
    ctx: null,
    points: [],
    frame: 0,
    width: 0,
    height: 0,
    maxAge: 520,
  };

  function initTrail() {
    const canvas = $("#padTrail");
    if (!canvas) return;
    trail.canvas = canvas;
    const resize = () => {
      const rect = canvas.getBoundingClientRect();
      const dpr = Math.min(2, window.devicePixelRatio || 1);
      trail.width = Math.max(1, rect.width);
      trail.height = Math.max(1, rect.height);
      canvas.width = Math.round(trail.width * dpr);
      canvas.height = Math.round(trail.height * dpr);
      const ctx = canvas.getContext("2d");
      if (ctx) {
        ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
        trail.ctx = ctx;
      }
    };
    if (typeof ResizeObserver === "function") {
      new ResizeObserver(resize).observe(canvas.parentElement || canvas);
    }
    window.addEventListener("resize", resize);
    resize();
  }

  function addTrailPoint(x, y) {
    if (!trail.ctx) return;
    const now = performance.now();
    trail.points.push({ x, y, t: now });
    if (trail.points.length > 64) trail.points.shift();
    if (!trail.frame) trail.frame = requestAnimationFrame(drawTrail);
  }

  function drawTrail(now) {
    trail.frame = 0;
    if (!trail.ctx) return;
    const ctx = trail.ctx;
    ctx.clearRect(0, 0, trail.width, trail.height);
    trail.points = trail.points.filter((point) => now - point.t < trail.maxAge);
    if (!trail.points.length) {
      ctx.clearRect(0, 0, trail.width, trail.height);
      return;
    }
    ctx.save();
    ctx.globalCompositeOperation = "source-over";
    ctx.lineCap = "round";
    ctx.lineJoin = "round";
    ctx.shadowColor = "#4CC9F0";
    ctx.shadowBlur = 12;
    for (let i = 1; i < trail.points.length; i++) {
      const from = trail.points[i - 1];
      const to = trail.points[i];
      const age = Math.max(0, 1 - (now - to.t) / trail.maxAge);
      ctx.strokeStyle = `rgba(76, 201, 240, ${age * 0.72})`;
      ctx.lineWidth = Math.max(1, 7 * age);
      ctx.beginPath();
      ctx.moveTo(from.x, from.y);
      ctx.lineTo(to.x, to.y);
      ctx.stroke();
    }
    const last = trail.points[trail.points.length - 1];
    const headAge = Math.max(0, 1 - (now - last.t) / trail.maxAge);
    ctx.fillStyle = `rgba(76, 201, 240, ${headAge * 0.95})`;
    ctx.beginPath();
    ctx.arc(last.x, last.y, Math.max(2, 6 * headAge), 0, Math.PI * 2);
    ctx.fill();
    ctx.restore();
    trail.frame = requestAnimationFrame(drawTrail);
  }

  function clearTrail() {
    trail.points = [];
    if (trail.ctx) trail.ctx.clearRect(0, 0, trail.width, trail.height);
  }

  function queueMove(dx, dy) {
    if (!Number.isFinite(dx) || !Number.isFinite(dy)) return;
    padState.pending.x += dx * state.pad.speed;
    padState.pending.y += dy * state.pad.speed;
    if (!padState.frame) {
      padState.frame = requestAnimationFrame(flushMove);
    }
  }

  function flushMove() {
    padState.frame = 0;
    const x = Math.trunc(padState.pending.x);
    const y = Math.trunc(padState.pending.y);
    padState.pending.x -= x;
    padState.pending.y -= y;
    if (!x && !y) return;
    if (!wsSend({ t: "move", dx: clamp(x, -800, 800), dy: clamp(y, -800, 800) })) {
      // Dropped input while disconnected; don't pile up deltas.
      padState.pending.x = 0;
      padState.pending.y = 0;
    }
  }

  function clamp(value, min, max) {
    return Math.max(min, Math.min(max, value));
  }

  function sendButton(button, action) {
    wsSend({ t: "button", button, action });
  }

  function sendWheel(x, y) {
    if (!x && !y) return;
    wsSend({ t: "wheel", x: clamp(x, -20, 20), y: clamp(y, -20, 20) });
  }

  function padRelative(event, target) {
    const rect = target.getBoundingClientRect();
    return { x: event.clientX - rect.left, y: event.clientY - rect.top };
  }

  function showRipple(point) {
    const ripple = $("#padRipple");
    ripple.style.left = `${point.x}px`;
    ripple.style.top = `${point.y}px`;
    ripple.classList.remove("go");
    void ripple.offsetWidth;
    ripple.classList.add("go");
  }

  function hideHint() {
    const hint = $("#padHint");
    if (hint && !hint.classList.contains("fade")) hint.classList.add("fade");
  }

  function stopInertia() {
    if (padState.inertiaFrame) {
      cancelAnimationFrame(padState.inertiaFrame);
      padState.inertiaFrame = 0;
    }
  }

  function startInertia() {
    if (!state.pad.inertia) return;
    const v = padState.velocity;
    const speed = Math.hypot(v.x, v.y);
    if (speed < 0.12) return;
    let vx = v.x;
    let vy = v.y;
    let last = performance.now();
    const step = (now) => {
      const dt = Math.min(48, now - last);
      last = now;
      const decay = Math.pow(0.93, dt / 16);
      vx *= decay;
      vy *= decay;
      if (Math.hypot(vx, vy) < 0.02) {
        padState.inertiaFrame = 0;
        return;
      }
      queueMove(vx * dt, vy * dt);
      padState.inertiaFrame = requestAnimationFrame(step);
    };
    padState.inertiaFrame = requestAnimationFrame(step);
  }

  function trackVelocity(point) {
    const now = performance.now();
    const samples = padState.gesture.samples;
    samples.push({ t: now, x: point.x, y: point.y });
    while (samples.length > 2 && now - samples[0].t > 90) samples.shift();
  }

  function computeVelocity() {
    const samples = padState.gesture ? padState.gesture.samples : [];
    if (samples.length < 2) return { x: 0, y: 0 };
    const last = samples[samples.length - 1];
    const first = samples[0];
    const dt = Math.max(1, last.t - first.t);
    return { x: (last.x - first.x) / dt, y: (last.y - first.y) / dt };
  }

  function bindPad() {
    const pad = $("#pad");
    if (!pad) return;

    const active = (on) => pad.classList.toggle("is-active", on);

    pad.addEventListener("pointerdown", (event) => {
      stopInertia();
      if (event.pointerType === "mouse") return;
      pad.setPointerCapture(event.pointerId);
      const point = padRelative(event, pad);
      padState.pointers.set(event.pointerId, { x: event.clientX, y: event.clientY });
      hideHint();
      active(true);
      if (padState.pointers.size === 1) {
        clearTrail();
        addTrailPoint(point.x, point.y);
        padState.gesture = {
          start: performance.now(),
          moved: 0,
          two: false,
          lastX: event.clientX,
          lastY: event.clientY,
          samples: [{ t: performance.now(), x: point.x, y: point.y }],
          startPoint: point,
        };
      } else if (padState.pointers.size === 2 && padState.gesture) {
        padState.gesture.two = true;
        padState.lastTwo = centroid();
      }
      event.preventDefault();
    });

    pad.addEventListener("pointermove", (event) => {
      const stored = padState.pointers.get(event.pointerId);
      if (!stored) return;
      stored.x = event.clientX;
      stored.y = event.clientY;
      const gesture = padState.gesture;
      if (!gesture) return;

      if (padState.pointers.size === 1 && !gesture.two) {
        const dx = event.clientX - gesture.lastX;
        const dy = event.clientY - gesture.lastY;
        gesture.lastX = event.clientX;
        gesture.lastY = event.clientY;
        gesture.moved += Math.hypot(dx, dy);
        const point = padRelative(event, pad);
        addTrailPoint(point.x, point.y);
        trackVelocity(point);
        queueMove(dx, dy);
      } else if (padState.pointers.size >= 2) {
        const center = centroid();
        if (!padState.lastTwo) {
          padState.lastTwo = center;
          return;
        }
        const dx = center.x - padState.lastTwo.x;
        const dy = center.y - padState.lastTwo.y;
        padState.lastTwo = center;
        gesture.moved += Math.hypot(dx, dy) * 0.6;
        padState.wheelAccum.x += dx;
        padState.wheelAccum.y += dy;
        const direction = state.pad.natural ? 1 : -1;
        const ticksY = Math.trunc(padState.wheelAccum.y / 26) * direction;
        const ticksX = Math.trunc(padState.wheelAccum.x / 26) * -direction;
        if (ticksY || ticksX) {
          padState.wheelAccum.x -= (ticksX / -direction) * 26;
          padState.wheelAccum.y -= (ticksY / direction) * 26;
          sendWheel(ticksX, ticksY);
          nudgeWheelThumb(ticksY * 6);
        }
      }
      event.preventDefault();
    });

    const finish = (event) => {
      if (!padState.pointers.has(event.pointerId)) return;
      padState.pointers.delete(event.pointerId);
      const gesture = padState.gesture;
      if (padState.pointers.size > 0) {
        if (padState.pointers.size === 1) {
          const remaining = padState.pointers.values().next().value;
          if (gesture) {
            gesture.lastX = remaining.x;
            gesture.lastY = remaining.y;
          }
          padState.lastTwo = null;
        }
        return;
      }
      active(false);
      if (!gesture) return;
      const duration = performance.now() - gesture.start;
      const isTap = duration < 280 && gesture.moved < 12;
      if (isTap) {
        const point = padRelative(event, pad);
        showRipple(point);
        if (gesture.two) {
          sendButton("right", "click");
        } else {
          sendButton("left", "click");
        }
      } else if (!gesture.two) {
        padState.velocity = computeVelocity();
        startInertia();
      }
      padState.gesture = null;
      padState.lastTwo = null;
      padState.wheelAccum.x = 0;
      padState.wheelAccum.y = 0;
    };

    pad.addEventListener("pointerup", finish);
    pad.addEventListener("pointercancel", (event) => {
      padState.pointers.delete(event.pointerId);
      if (padState.pointers.size === 0) {
        active(false);
        padState.gesture = null;
        padState.lastTwo = null;
      }
    });
    pad.addEventListener("contextmenu", (event) => event.preventDefault());

    function centroid() {
      const points = Array.from(padState.pointers.values());
      const slice = points.slice(0, 2);
      const x = slice.reduce((sum, p) => sum + p.x, 0) / slice.length;
      const y = slice.reduce((sum, p) => sum + p.y, 0) / slice.length;
      return { x, y };
    }
  }

  function nudgeWheelThumb(dy) {
    const thumb = $(".wheel-thumb");
    if (!thumb) return;
    const current = Number(thumb.dataset.offset || 0);
    const next = clamp(current + dy, -34, 34);
    thumb.dataset.offset = String(next);
    thumb.style.transform = `translateY(${next}px)`;
    clearTimeout(nudgeWheelThumb.timer);
    nudgeWheelThumb.timer = setTimeout(() => {
      thumb.style.transform = "";
      thumb.dataset.offset = "0";
    }, 260);
  }

  function bindWheel() {
    const wheel = $("#wheel");
    if (!wheel) return;
    let drag = null;
    let lastTap = 0;
    wheel.addEventListener("pointerdown", (event) => {
      if (event.pointerType === "mouse") return;
      wheel.setPointerCapture(event.pointerId);
      drag = {
        y: event.clientY,
        accum: 0,
        moved: 0,
        start: performance.now(),
      };
      wheel.classList.add("is-active");
      event.preventDefault();
    });
    wheel.addEventListener("pointermove", (event) => {
      if (!drag) return;
      const dy = event.clientY - drag.y;
      drag.y = event.clientY;
      drag.accum += dy;
      drag.moved += Math.abs(dy);
      const step = 18;
      const ticks = Math.trunc(drag.accum / step);
      if (ticks) {
        drag.accum -= ticks * step;
        sendWheel(0, -ticks);
        nudgeWheelThumb(-ticks * 6);
      }
      event.preventDefault();
    });
    const end = (event, cancelled) => {
      if (!drag) return;
      const now = performance.now();
      const isTap = !cancelled && drag.moved < 6 && now - drag.start < 260;
      drag = null;
      wheel.classList.remove("is-active");
      $(".wheel-thumb").style.transform = "";
      $(".wheel-thumb").dataset.offset = "0";
      if (!isTap) return;
      if (now - lastTap < 340) {
        lastTap = 0;
        sendButton("middle", "click");
        wheel.classList.add("middle-flash");
        clearTimeout(bindWheel.flashTimer);
        bindWheel.flashTimer = setTimeout(() => wheel.classList.remove("middle-flash"), 220);
      } else {
        lastTap = now;
      }
    };
    wheel.addEventListener("pointerup", (event) => end(event, false));
    wheel.addEventListener("pointercancel", (event) => end(event, true));
    wheel.addEventListener("keydown", (event) => {
      if (event.key === "ArrowUp" || event.key === "ArrowDown") {
        const dir = event.key === "ArrowUp" ? 1 : -1;
        sendWheel(0, dir);
        nudgeWheelThumb(-dir * 6);
        event.preventDefault();
      }
    });
    wheel.addEventListener("contextmenu", (event) => event.preventDefault());
  }

  function bindPadButtons() {
    $$("[data-mouse]").forEach((button) => {
      const name = button.dataset.mouse;
      const press = (event) => {
        event.preventDefault();
        try {
          button.setPointerCapture(event.pointerId);
        } catch (_) {}
        button.classList.add("pressed");
        sendButton(name, "down");
      };
      const release = () => {
        if (!button.classList.contains("pressed")) return;
        button.classList.remove("pressed");
        sendButton(name, "up");
      };
      button.addEventListener("pointerdown", press);
      button.addEventListener("pointerup", release);
      button.addEventListener("pointercancel", release);
      button.addEventListener("lostpointercapture", release);
      button.addEventListener("contextmenu", (event) => event.preventDefault());
    });

  }

  function bindPadSettings() {
    let saved = {};
    try {
      saved = JSON.parse(localStorage.getItem("lumo.pad") || "{}");
    } catch (_) {}
    state.pad.speed = Number(saved.speed) || 1.2;
    state.pad.natural = saved.natural !== false;
    state.pad.inertia = saved.inertia !== false;

    const speed = $("#pointerSpeed");
    speed.value = state.pad.speed;
    $("#speedValue").textContent = `${state.pad.speed.toFixed(1)}×`;
    speed.addEventListener("input", () => {
      state.pad.speed = Number(speed.value) || 1.2;
      $("#speedValue").textContent = `${state.pad.speed.toFixed(1)}×`;
      savePad();
    });
    const natural = $("#naturalScroll");
    natural.checked = state.pad.natural;
    natural.addEventListener("change", () => {
      state.pad.natural = natural.checked;
      savePad();
    });
    const inertia = $("#inertia");
    inertia.checked = state.pad.inertia;
    inertia.addEventListener("change", () => {
      state.pad.inertia = inertia.checked;
      savePad();
    });
  }

  function savePad() {
    localStorage.setItem("lumo.pad", JSON.stringify(state.pad));
  }

  /* ---------------- Clipboard ---------------- */

  function setClipboard(clipboard, notify) {
    const previous = state.clipboard;
    state.clipboard = clipboard || { kind: "empty", files: [], at: 0 };
    renderClipboard();
    const changed =
      (previous.kind !== state.clipboard.kind) ||
      (previous.text !== state.clipboard.text) ||
      (previous.files || []).map((f) => f.id).join() !==
        (state.clipboard.files || []).map((f) => f.id).join();
    if (notify && changed && state.clipboard.kind !== "empty") {
      pushHistory(state.clipboard);
      const label =
        state.clipboard.kind === "text"
          ? "Text received from PC"
          : state.clipboard.files?.[0]?.name
          ? `Received ${state.clipboard.files[0].name}`
          : "Clipboard updated";
      toast(label);
    }
  }

  function pushHistory(clipboard) {
    const key = `${clipboard.kind}:${clipboard.text || ""}:${(clipboard.files || [])
      .map((f) => f.id)
      .join()}`;
    if (state.history[0] && state.history[0].key === key) return;
    state.history.unshift({ key, clipboard, at: Date.now() });
    state.history = state.history.slice(0, 20);
    renderClipHistory();
  }

  function renderClipboard() {
    const clipboard = state.clipboard;
    const kindBadge = $("#clipKind");
    const preview = $("#clipPreview");
    const copyBtn = $("#copyClip");
    if (!clipboard || clipboard.kind === "empty") {
      kindBadge.textContent = "Empty";
      kindBadge.className = "badge";
      preview.innerHTML = '<p class="muted">Copy something on your PC and it appears here.</p>';
      copyBtn.disabled = true;
      $("#clipUpdated").textContent = "Not synced yet";
      return;
    }
    $("#clipUpdated").textContent = `Updated ${timeAgo(clipboard.at)}`;
    if (clipboard.kind === "text") {
      kindBadge.textContent = "Text";
      kindBadge.className = "badge accent";
      preview.textContent = clipboard.text || "";
      copyBtn.disabled = false;
    } else if (clipboard.kind === "image") {
      kindBadge.textContent = "Image";
      kindBadge.className = "badge accent";
      const file = (clipboard.files || [])[0];
      preview.innerHTML = file
        ? `<img src="${dlUrl(file.id, true)}" alt="Clipboard image from your PC" />`
        : '<p class="muted">Image unavailable.</p>';
      copyBtn.disabled = true;
    } else {
      kindBadge.textContent = "Files";
      kindBadge.className = "badge accent";
      const files = clipboard.files || [];
      preview.innerHTML = `<div class="mini">${files
        .map(
          (file) =>
            `<a class="mini-row" href="${dlUrl(file.id)}" download>${icon(
              file.isDir ? "folder" : "download",
              16
            )}<span>${esc(file.name)}</span><span class="faint">${bytes(file.size)}</span></a>`
        )
        .join("")}</div>`;
      copyBtn.disabled = true;
    }
  }

  function renderClipHistory() {
    const list = $("#clipHistory");
    const empty = $("#clipHistoryEmpty");
    if (!state.history.length) {
      list.innerHTML = "";
      empty.hidden = false;
      return;
    }
    empty.hidden = true;
    list.innerHTML = state.history
      .map((entry) => {
        const clip = entry.clipboard;
        const title =
          clip.kind === "text"
            ? (clip.text || "").slice(0, 140)
            : clip.kind === "image"
            ? "Image from PC"
            : `${(clip.files || []).length} file(s) from PC`;
        const actions =
          clip.kind === "text"
            ? `<button class="icon-btn small" data-copy-history="${encodeURIComponent(
                clip.text || ""
              )}" aria-label="Copy text">${icon("copy", 17)}</button>`
            : clip.files?.[0]
            ? `<a class="icon-btn small" href="${dlUrl(clip.files[0].id)}" download aria-label="Download">${icon(
                "download",
                17
              )}</a>`
            : "";
        return `<div class="item"><span class="item-icon ${clip.kind === "text" ? "text" : "file"}">${kindIcon(
          clip.kind
        )}</span><div class="item-main"><div class="item-name clip-text">${esc(
          title
        )}</div><div class="item-meta">${timeAgo(entry.at / 1000)}</div></div><div class="item-actions">${actions}</div></div>`;
      })
      .join("");
    $$("[data-copy-history]", list).forEach((button) => {
      button.addEventListener("click", () => {
        copyText(decodeURIComponent(button.dataset.copyHistory)).then((ok) =>
          toast(ok ? "Copied on phone" : "Couldn't copy — select the text manually", ok ? "success" : "error")
        );
      });
    });
  }

  async function copyText(text) {
    if (navigator.clipboard && window.isSecureContext) {
      try {
        await navigator.clipboard.writeText(text);
        return true;
      } catch (_) {}
    }
    const area = document.createElement("textarea");
    area.value = text;
    area.setAttribute("readonly", "");
    area.style.position = "fixed";
    area.style.top = "0";
    area.style.left = "0";
    area.style.width = "1px";
    area.style.height = "1px";
    area.style.padding = "0";
    area.style.border = "0";
    area.style.opacity = "0.01";
    area.style.fontSize = "16px";
    document.body.appendChild(area);
    area.focus();
    area.select();
    area.setSelectionRange(0, area.value.length);
    let ok = false;
    try {
      ok = document.execCommand("copy");
    } catch (_) {}
    area.remove();
    return ok;
  }

  async function refreshClipboard() {
    try {
      const res = await api("/api/clipboard");
      setClipboard(res.clipboard, false);
    } catch (err) {
      if (err.status !== 401) toast("Couldn't refresh the clipboard", "error");
    }
  }

  async function sendClipboardText() {
    const input = $("#clipText");
    const text = input.value;
    if (!text.trim()) {
      toast("Type something first");
      input.focus();
      return;
    }
    try {
      await api("/api/clipboard", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ text }),
      });
      input.value = "";
      toast("Copied to your PC clipboard", "success");
      refreshClipboard();
    } catch (err) {
      if (err.status !== 401) toast(err.message || "Couldn't send text", "error");
    }
  }

  async function uploadClipboardFiles(files) {
    if (!files.length) return;
    toast(`Sending ${files.length} file${files.length === 1 ? "" : "s"} to the PC clipboard…`);
    for (const file of files) {
      try {
        // Windows decodes PNG/JPEG for the clipboard; convert HEIC/WebP/etc. first.
        const prepared = await normalizeClipboardImage(file);
        await putFile(prepared, { clipboard: true });
      } catch (err) {
        toast(`${file.name}: ${err.message}`, "error", 5000);
      }
    }
    toast("Ready to paste on your PC", "success");
    refreshClipboard();
  }

  async function normalizeClipboardImage(file) {
    if (!file.type || !file.type.startsWith("image/")) return file;
    if (file.type === "image/png" || file.type === "image/jpeg") return file;
    if (typeof createImageBitmap !== "function" || !HTMLCanvasElement.prototype.toBlob) {
      return file;
    }
    try {
      const bitmap = await createImageBitmap(file);
      const canvas = document.createElement("canvas");
      canvas.width = bitmap.width;
      canvas.height = bitmap.height;
      canvas.getContext("2d").drawImage(bitmap, 0, 0);
      bitmap.close?.();
      const blob = await new Promise((resolve) => canvas.toBlob(resolve, "image/png"));
      if (!blob) return file;
      const name = file.name.replace(/\.[^.]+$/, "") + ".png";
      return new File([blob], name, { type: "image/png" });
    } catch (_) {
      return file;
    }
  }

  /* ---------------- Files ---------------- */

  function renderShares() {
    const list = $("#shareList");
    const empty = $("#shareEmpty");
    const shares = state.shares || [];
    if (!shares.length) {
      list.innerHTML = "";
      empty.hidden = false;
      return;
    }
    empty.hidden = true;
    list.innerHTML = shares
      .map(
        (item) =>
          `<div class="item"><span class="item-icon ${item.kind}">${kindIcon(
            item.kind
          )}</span><div class="item-main"><div class="item-name">${esc(item.name)}</div><div class="item-meta">${
            item.isDir ? "Folder · downloads as .zip" : `${bytes(item.size)} · ${KIND_LABEL[item.kind] || "File"}`
          }</div></div><div class="item-actions"><a class="icon-btn small" href="${dlUrl(
            item.id
          )}" download aria-label="Download ${esc(item.name)}">${icon("download", 18)}</a></div></div>`
      )
      .join("");
  }

  async function refreshShares() {
    try {
      const res = await api("/api/shares");
      state.shares = res.items || [];
      renderShares();
    } catch (err) {
      if (err.status !== 401) toast("Couldn't load shared files", "error");
    }
  }

  async function refreshReceived() {
    try {
      const res = await api("/api/received");
      state.received = res.items || [];
      renderReceived();
    } catch (err) {
      if (err.status !== 401 && state.view === "files") toast("Couldn't load received files", "error");
    }
  }

  function renderReceived() {
    const list = $("#recentUploads");
    if (!list) return;
    const items = (state.received || []).slice(0, 8);
    if (!items.length) {
      list.innerHTML = `<div class="empty"><span class="empty-icon">${icon(
        "upload",
        24
      )}</span><h3>Nothing received yet</h3><p>Files you send land in the PC's Lumo inbox.</p></div>`;
      return;
    }
    list.innerHTML = items
      .map(
        (item) =>
          `<div class="item"><span class="item-icon ${item.kind}">${kindIcon(
            item.kind
          )}</span><div class="item-main"><div class="item-name">${esc(item.name)}</div><div class="item-meta">${
            item.isDir ? "Folder" : bytes(item.size)
          } · ${timeAgo(item.addedAt)}</div></div><span class="badge success">Saved</span></div>`
      )
      .join("");
  }

  function queueUploads(fileList) {
    const files = Array.from(fileList || []);
    if (!files.length) return;
    uploadSequentially(files);
  }

  async function uploadSequentially(files) {
    const list = $("#uploadList");
    for (const file of files) {
      const row = document.createElement("div");
      row.className = "upload";
      row.innerHTML = `<div class="upload-top"><span class="upload-name">${esc(
        file.name
      )}</span><span class="upload-status">0%</span></div><div class="progress"><span></span></div>`;
      list.prepend(row);
      const bar = $(".progress > span", row);
      const status = $(".upload-status", row);
      try {
        const extract = /\.zip$/i.test(file.name);
        const result = await putFile(file, {
          extract,
          onProgress: (fraction) => {
            const pct = Math.round(fraction * 100);
            bar.style.width = `${pct}%`;
            status.textContent = pct >= 100 ? "Processing…" : `${pct}%`;
          },
        });
        bar.style.width = "100%";
        status.textContent = "Saved";
        row.classList.add("done");
        toast(`Sent ${file.name}`, "success");
        if (result && result.item) {
          state.received.unshift(result.item);
          renderReceived();
        }
      } catch (err) {
        row.classList.add("failed");
        status.textContent = "Failed";
        if (err.status !== 401) toast(`${file.name}: ${err.message}`, "error", 5000);
      }
    }
    refreshReceived();
  }

  function putFile(file, { clipboard = false, extract = false, onProgress } = {}) {
    return new Promise((resolve, reject) => {
      const params = new URLSearchParams();
      if (clipboard) params.set("clipboard", "1");
      if (extract) params.set("extract", "1");
      const query = params.toString();
      const xhr = new XMLHttpRequest();
      xhr.open("PUT", `/api/upload${query ? `?${query}` : ""}`);
      xhr.setRequestHeader("X-Lumo-Name", encodeURIComponent(file.name));
      xhr.setRequestHeader("Content-Type", "application/octet-stream");
      if (xhr.upload && onProgress) {
        xhr.upload.onprogress = (event) => {
          if (event.lengthComputable) onProgress(event.loaded / event.total);
        };
      }
      xhr.onload = () => {
        if (xhr.status === 401) {
          showPairing();
          reject(new ApiError(401, "Pairing required"));
          return;
        }
        if (xhr.status >= 200 && xhr.status < 300) {
          try {
            resolve(JSON.parse(xhr.responseText));
          } catch (_) {
            resolve({});
          }
        } else {
          let message = `Upload failed (${xhr.status})`;
          try {
            const body = JSON.parse(xhr.responseText);
            if (body.error) message = body.error;
          } catch (_) {}
          reject(new ApiError(xhr.status, message));
        }
      };
      xhr.onerror = () => reject(new ApiError(0, "Network error"));
      xhr.send(file);
    });
  }

  /* ---------------- Power ---------------- */

  const POWER_COPY = {
    lock: { title: "Lock your PC?", body: "The screen locks and apps keep running.", label: "Lock" },
    sleep: { title: "Put your PC to sleep?", body: "The PC suspends until you wake it.", label: "Sleep", tone: "danger" },
    restart: { title: "Restart your PC?", body: "Apps close and the PC reboots. You get 8 seconds to cancel.", label: "Restart", tone: "danger" },
    shutdown: { title: "Shut down your PC?", body: "Apps close and the PC powers off. You get 8 seconds to cancel.", label: "Shut down", tone: "danger" },
  };

  let armedPower = null;
  let armedTimer = 0;

  function resetArmedPower() {
    clearTimeout(armedTimer);
    armedTimer = 0;
    if (!armedPower) return;
    const button = document.querySelector(`.power-card[data-power="${armedPower}"]`);
    if (button) {
      button.classList.remove("armed");
      const sub = button.querySelector(".power-sub");
      if (sub && button.dataset.originalSub) sub.textContent = button.dataset.originalSub;
    }
    armedPower = null;
  }

  function armPower(button, action) {
    if (!button) return;
    resetArmedPower();
    armedPower = action;
    button.dataset.originalSub = button.querySelector(".power-sub")?.textContent || "";
    button.classList.add("armed");
    const sub = button.querySelector(".power-sub");
    if (sub) sub.textContent = "Tap again to confirm";
    armedTimer = setTimeout(resetArmedPower, 4500);
  }

  async function requestPower(action) {
    const copy = POWER_COPY[action];
    if (!copy) return;
    // A two-tap inline confirmation is more reliable on phones than a modal,
    // and it keeps destructive actions explicit.
    if (action !== "lock") {
      const button = document.querySelector(`.power-card[data-power="${action}"]`);
      if (armedPower !== action) {
        armPower(button, action);
        return;
      }
      resetArmedPower();
    } else {
      resetArmedPower();
    }
    try {
      const res = await api("/api/power", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ action }),
      });
      if (res.scheduled) {
        state.pending = { action, until: Date.now() + Number(res.seconds || 8) * 1000 };
        renderPending();
        toast(`${copy.label} scheduled — cancel within ${res.seconds || 8}s`, "success", 4000);
      } else {
        toast(`${copy.label} request sent`, "success");
      }
    } catch (err) {
      if (err.status !== 401) toast(err.message || "Power action failed", "error");
    }
  }

  function renderPending() {
    const card = $("#pendingCard");
    clearInterval(renderPending.timer);
    if (!state.pending) {
      card.hidden = true;
      return;
    }
    card.hidden = false;
    const title = state.pending.action === "restart" ? "Restarting" : "Shutting down";
    $("#pendingTitle").textContent = title;
    const tick = () => {
      if (!state.pending) {
        card.hidden = true;
        clearInterval(renderPending.timer);
        return;
      }
      const remaining = Math.max(0, Math.ceil((state.pending.until - Date.now()) / 1000));
      $("#pendingCountdown").textContent = remaining > 0 ? `in ${remaining}s · tap Cancel to stop` : "now";
      const total = 8;
      $("#pendingProgress").style.width = `${Math.max(0, Math.min(100, (remaining / total) * 100))}%`;
      if (remaining <= 0) {
        card.hidden = true;
        state.pending = null;
        clearInterval(renderPending.timer);
      }
    };
    tick();
    renderPending.timer = setInterval(tick, 250);
  }

  async function cancelPower() {
    try {
      await api("/api/power", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ action: "cancel" }),
      });
      state.pending = null;
      renderPending();
      toast("Cancelled", "success");
    } catch (err) {
      if (err.status !== 401) toast(err.message || "Nothing to cancel", "error");
    }
  }

  /* ---------------- Settings sheet ---------------- */

  function openSheet() {
    $("#sheetBackdrop").hidden = false;
    const sheet = $("#settingsSheet");
    sheet.classList.add("open");
    sheet.setAttribute("aria-hidden", "false");
    $("#closeSheet").focus({ preventScroll: true });
  }

  function closeSheet() {
    const sheet = $("#settingsSheet");
    if (!sheet) return;
    sheet.classList.remove("open");
    sheet.setAttribute("aria-hidden", "true");
    $("#sheetBackdrop").hidden = true;
  }

  function bindSettings() {
    $("#btnSettings").addEventListener("click", openSheet);
    $("#closeSheet").addEventListener("click", closeSheet);
    $("#sheetBackdrop").addEventListener("click", closeSheet);

    $$("[data-theme-choice]").forEach((button) => {
      button.addEventListener("click", () => {
        const choice = button.dataset.themeChoice;
        localStorage.setItem("lumo.theme", choice);
        applyTheme(choice);
      });
    });

    $("#sendTypedText").addEventListener("click", async () => {
      const input = $("#typeText");
      const text = input.value;
      if (!text) {
        toast("Type something first");
        return;
      }
      if (wsSend({ t: "text", text })) {
        input.value = "";
        toast("Keystrokes sent", "success");
        return;
      }
      try {
        await api("/api/text", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ text }),
        });
        input.value = "";
        toast("Keystrokes sent", "success");
      } catch (err) {
        if (err.status !== 401) toast("Couldn't send text", "error");
      }
    });

    $("#unpair").addEventListener("click", async () => {
      const ok = await confirmModal({
        title: "Unpair this phone?",
        body: "You'll need to scan the QR code on your PC again to reconnect.",
        confirmLabel: "Unpair",
        tone: "danger",
      });
      if (!ok) return;
      try {
        await api("/api/unpair", { method: "POST" });
      } catch (_) {}
      closeSheet();
      showPairing();
    });
  }

  /* ---------------- Bindings ---------------- */

  function bindChrome() {
    $$(".tab").forEach((tab) => {
      tab.addEventListener("click", () => switchView(tab.dataset.tab));
    });

    $("#retryPair").addEventListener("click", () => location.reload());

    $("#modalCancel").addEventListener("click", () => settleModal(false));
    $("#modalConfirm").addEventListener("click", () => settleModal(true));
    $("#modalBackdrop").addEventListener("click", (event) => {
      if (event.target === $("#modalBackdrop")) settleModal(false);
    });
    document.addEventListener("keydown", (event) => {
      if (event.key === "Escape") {
        if (!$("#modalBackdrop").hidden) settleModal(false);
        else closeSheet();
      }
    });

    document.addEventListener("visibilitychange", () => {
      if (!document.hidden) {
        if (!state.ws && $("#pairing").hidden) connect();
        loadBootstrap();
      }
    });
  }

  function bindFiles() {
    $("#tabFromPc").addEventListener("click", () => selectFilesTab("from"));
    $("#tabToPc").addEventListener("click", () => selectFilesTab("to"));
    $("#refreshShares").addEventListener("click", refreshShares);
    $("#refreshUploads").addEventListener("click", refreshReceived);
    $("#pickFiles").addEventListener("click", () => $("#fileInput").click());
    $("#fileInput").addEventListener("change", (event) => {
      queueUploads(event.target.files);
      event.target.value = "";
    });
  }

  function selectFilesTab(tab) {
    const from = tab === "from";
    $("#tabFromPc").setAttribute("aria-selected", from ? "true" : "false");
    $("#tabToPc").setAttribute("aria-selected", from ? "false" : "true");
    $("#panelFromPc").hidden = !from;
    $("#panelToPc").hidden = from;
    if (from) refreshShares();
    else refreshReceived();
  }

  function bindClipboard() {
    $("#copyClip").addEventListener("click", async () => {
      if (state.clipboard.kind !== "text" || !state.clipboard.text) return;
      const ok = await copyText(state.clipboard.text);
      toast(ok ? "Copied on phone" : "Couldn't copy", ok ? "success" : "error");
    });
    $("#refreshClip").addEventListener("click", refreshClipboard);
    $("#sendClip").addEventListener("click", sendClipboardText);
    $("#clipText").addEventListener("keydown", (event) => {
      if (event.key === "Enter" && (event.metaKey || event.ctrlKey)) sendClipboardText();
    });
    $("#clipFiles").addEventListener("click", () => $("#clipFileInput").click());
    $("#clipFileInput").addEventListener("change", (event) => {
      uploadClipboardFiles(Array.from(event.target.files || []));
      event.target.value = "";
    });
    $("#clearClipHistory").addEventListener("click", () => {
      state.history = [];
      renderClipHistory();
    });
  }

  function bindPower() {
    $$("[data-power]").forEach((button) => {
      button.addEventListener("click", () => requestPower(button.dataset.power));
    });
    $("#cancelPower").addEventListener("click", cancelPower);
  }

  function bindQuickKeyboard() {
    const overlay = $("#keyboardOverlay");
    const input = $("#quickType");
    let sent = "";
    let warned = false;

    const fitViewport = () => {
      if (overlay.hidden) return;
      const viewport = window.visualViewport;
      if (!viewport) return;
      overlay.style.top = `${viewport.offsetTop}px`;
      overlay.style.height = `${viewport.height}px`;
      overlay.style.bottom = "auto";
    };
    const reset = () => {
      input.value = "";
      sent = "";
      warned = false;
    };
    const open = () => {
      overlay.hidden = false;
      reset();
      fitViewport();
      input.focus();
    };
    const close = () => {
      overlay.hidden = true;
      overlay.style.removeProperty("top");
      overlay.style.removeProperty("height");
      overlay.style.removeProperty("bottom");
      reset();
      input.blur();
    };
    if (window.visualViewport) {
      window.visualViewport.addEventListener("resize", fitViewport);
      window.visualViewport.addEventListener("scroll", fitViewport);
    }

    $("#padKeyboard").addEventListener("click", open);
    overlay.addEventListener("click", close);
    document.addEventListener("keydown", (event) => {
      if (event.key === "Escape" && !overlay.hidden) close();
    });

    // Send only the delta since the previous successful update. Backspace is
    // sent as \b, which the PC input layer maps to VK_BACK. If autocorrect or
    // a selection rewrite makes the new value unrelated, clear what we sent
    // and retype the current value from scratch.
    function sendLive() {
      const value = input.value;
      if (value === sent) return;
      let payload;
      if (value.startsWith(sent)) {
        payload = value.slice(sent.length);
      } else if (sent.startsWith(value)) {
        const removed = [...sent].length - [...value].length;
        payload = "\b".repeat(Math.max(0, removed));
      } else {
        payload = "\b".repeat([...sent].length) + value;
      }
      if (!payload) {
        sent = value;
        return;
      }
      if (wsSend({ t: "text", text: payload })) {
        sent = value;
        warned = false;
      } else if (!warned) {
        toast("Not connected to your PC", "error");
        warned = true;
      }
    }

    input.addEventListener("input", sendLive);
    input.addEventListener("keydown", (event) => {
      // Native textarea backspace does nothing at an empty value, but the PC
      // may still have existing text to delete.
      if (event.key === "Backspace" && input.value.length === 0) {
        event.preventDefault();
        if (wsSend({ t: "text", text: "\b" })) {
          warned = false;
        } else if (!warned) {
          toast("Not connected to your PC", "error");
          warned = true;
        }
        return;
      }
      if (event.key === "Enter" && (event.metaKey || event.ctrlKey)) {
        event.preventDefault();
        close();
      }
    });
  }

  /* ---------------- Init ---------------- */

  function init() {
    hydrateIcons();
    applyTheme(currentThemeChoice());
    bindChrome();
    initTrail();
    bindPad();
    bindPadButtons();
    bindPadSettings();
    bindWheel();
    bindMedia();
    bindFiles();
    bindClipboard();
    bindPower();
    bindQuickKeyboard();
    bindSettings();
    setConnection("connecting", "Connecting");
    renderClipHistory();
    loadBootstrap();
    setInterval(() => {
      if (state.view === "files" && !document.hidden) refreshShares();
    }, 6000);
    setInterval(updateMediaProgress, 500);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
