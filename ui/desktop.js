/* Lumo desktop client — talks to the Rust core through Tauri IPC. */
(() => {
  "use strict";

  const $ = (sel, root = document) => root.querySelector(sel);
  const $$ = (sel, root = document) => Array.from(root.querySelectorAll(sel));

  const ICONS = {
    overview:
      '<rect x="3" y="3" width="7" height="9" rx="1"/><rect x="14" y="3" width="7" height="5" rx="1"/><rect x="14" y="12" width="7" height="9" rx="1"/><rect x="3" y="16" width="7" height="5" rx="1"/>',
    folder:
      '<path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z"/>',
    "folder-plus":
      '<path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z"/><path d="M12 10v6"/><path d="M9 13h6"/>',
    clipboard:
      '<rect x="8" y="2" width="8" height="4" rx="1"/><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/>',
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
    power:
      '<path d="M18.36 6.64a9 9 0 1 1-12.73 0"/><path d="M12 2v10"/>',
    settings:
      '<path d="M4 6h10"/><path d="M18 6h2"/><circle cx="16" cy="6" r="2"/><path d="M4 12h2"/><path d="M10 12h10"/><circle cx="8" cy="12" r="2"/><path d="M4 18h10"/><path d="M18 18h2"/><circle cx="16" cy="18" r="2"/>',
    moon: '<path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"/>',
    sun:
      '<circle cx="12" cy="12" r="4"/><path d="M12 2v2"/><path d="M12 20v2"/><path d="m4.93 4.93 1.41 1.41"/><path d="m17.66 17.66 1.41 1.41"/><path d="M2 12h2"/><path d="M20 12h2"/><path d="m6.34 17.66-1.41 1.41"/><path d="m19.07 4.93-1.41 1.41"/>',
    plus: '<path d="M12 5v14"/><path d="M5 12h14"/>',
    inbox:
      '<path d="M22 12h-6l-2 3h-4l-2-3H2"/><path d="M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11Z"/>',
    copy:
      '<rect x="9" y="9" width="12" height="12" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>',
    link:
      '<path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/>',
    refresh:
      '<path d="M21 12a9 9 0 0 1-9 9 9 9 0 0 1-8.2-5.3"/><path d="M3 12a9 9 0 0 1 9-9 9 9 0 0 1 8.2 5.3"/><path d="M21 4v4h-4"/><path d="M3 20v-4h4"/>',
    lock:
      '<rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/>',
    activity: '<path d="M22 12h-4l-3 9L9 3l-3 9H2"/>',
    alert:
      '<path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0Z"/><path d="M12 9v4"/><path d="M12 17h.01"/>',
    upload:
      '<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><path d="M7 8l5-5 5 5"/><path d="M12 3v12"/>',
    send:
      '<path d="m22 2-7 20-4-9-9-4Z"/><path d="M22 2 11 13"/>',
    shield:
      '<path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10Z"/><path d="m9 12 2 2 4-4"/>',
    external:
      '<path d="M15 3h6v6"/><path d="M10 14 21 3"/><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>',
    smartphone:
      '<rect x="5" y="2" width="14" height="20" rx="2"/><path d="M12 18h.01"/>',
    x: '<path d="M18 6 6 18"/><path d="m6 6 12 12"/>',
  };

  function icon(name, size = 18) {
    const body = ICONS[name] || ICONS.overview;
    return `<svg viewBox="0 0 24 24" width="${size}" height="${size}" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false">${body}</svg>`;
  }

  function hydrateIcons(root = document) {
    $$("[data-icon]", root).forEach((el) => {
      if (el.dataset.iconReady) return;
      el.innerHTML = icon(el.dataset.icon, Number(el.dataset.size) || 18);
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

  function formatUptime(seconds) {
    const s = Math.max(0, Math.floor(seconds || 0));
    if (s < 60) return `${s}s`;
    if (s < 3600) return `${Math.floor(s / 60)}m ${s % 60}s`;
    if (s < 86400) return `${Math.floor(s / 3600)}h ${Math.floor((s % 3600) / 60)}m`;
    return `${Math.floor(s / 86400)}d ${Math.floor((s % 86400) / 3600)}h`;
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
      case "image":
        return icon("image", 18);
      case "video":
        return icon("film", 18);
      case "audio":
        return icon("music", 18);
      case "archive":
        return icon("archive", 18);
      case "text":
        return icon("text", 18);
      case "folder":
        return icon("folder", 18);
      default:
        return icon("file", 18);
    }
  }

  const tauri = window.__TAURI__;
  if (!tauri || !tauri.core || !tauri.event) {
    $("#fatal").hidden = false;
    return;
  }
  const invoke = tauri.core.invoke;
  const listen = tauri.event.listen;

  const state = {
    boot: null,
    view: "overview",
    skew: 0,
    refreshing: false,
    refreshTimer: null,
    modalResolver: null,
    restartRequired: false,
    previewCache: new Map(),
    themeChoice: "system",
  };

  const PAGE_META = {
    overview: { title: "Overview", sub: "“The design is very human.”©" },
    files: { title: "Files", sub: "Share from the PC, receive from the phone — folders become zips." },
    clipboard: { title: "Clipboard", sub: "Move text, images and files between Windows and your phone." },
    settings: { title: "Settings", sub: "Connection, startup, security and appearance." },
  };

  /* ---------------- Data ---------------- */

  async function load() {
    try {
      const boot = await invoke("bootstrap");
      applyBoot(boot);
      $("#shell").hidden = false;
      $("#fatal").hidden = true;
    } catch (err) {
      $("#fatal").hidden = false;
      $("#shell").hidden = true;
      console.error(err);
    }
  }

  function applyBoot(boot) {
    state.boot = boot;
    state.themeChoice = boot.theme || "dark";
    state.skew = Date.now() - (Number(boot.serverTime) || 0) * 1000;
    applyTheme(state.themeChoice, false);
    renderAll();
  }

  async function refresh(silent = true) {
    if (state.refreshing) return;
    state.refreshing = true;
    try {
      const boot = await invoke("bootstrap");
      applyBoot(boot);
    } catch (err) {
      if (!silent) toast(String(err), "error");
    } finally {
      state.refreshing = false;
    }
  }

  function scheduleRefresh() {
    clearTimeout(state.refreshTimer);
    state.refreshTimer = setTimeout(() => refresh(true), 220);
  }

  /* ---------------- Rendering ---------------- */

  function serverNow() {
    return (Date.now() - state.skew) / 1000;
  }

  function timeAgo(epochSeconds) {
    if (!epochSeconds) return "just now";
    const diff = Math.max(0, Math.floor(serverNow() - epochSeconds));
    if (diff < 10) return "just now";
    if (diff < 60) return `${diff}s ago`;
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    return `${Math.floor(diff / 86400)}d ago`;
  }

  function renderAll() {
    if (!state.boot) return;
    const boot = state.boot;
    renderHeaderStatus();
    renderPairing();
    renderStats();
    renderDevices();
    renderShares();
    renderReceived();
    renderClipboard();
    renderActivity();
    renderSettings();
    $("#navShareCount").textContent = boot.shares.length;
    $("#navShareCount").hidden = boot.shares.length === 0;
    $("#versionText").textContent = boot.version;
    $("#sideVersion").textContent = boot.version;
  }

  function renderHeaderStatus() {
    const boot = state.boot;
    const running = !boot.serverError;
    const badge = $("#serverBadge");
    badge.className = `badge ${running ? "success" : "error"}`;
    $("#serverBadgeText").textContent = running ? "Server running" : "Server error";
    $("#serverPill").className = `badge ${running ? "success" : "error"}`;
    $("#serverPill").innerHTML = `<span class="dot"></span>${running ? "Running" : "Error"}`;
    $("#sideDot").dataset.state = running ? "online" : "offline";
    $("#sideStatus").textContent = running ? "Serving on your network" : "Server stopped";
    $("#sideAddress").textContent = boot.baseUrl;
  }

  function renderPairing() {
    const boot = state.boot;
    $("#pairUrl").textContent = boot.pairUrl;
    const frame = $("#qrFrame");
    const skeleton = $("#qrSkeleton");
    if (boot.qrSvg && (!frame.dataset.url || frame.dataset.url !== boot.pairUrl)) {
      frame.dataset.url = boot.pairUrl;
      if (skeleton) skeleton.remove();
      const old = $("svg", frame);
      if (old) old.remove();
      frame.insertAdjacentHTML("beforeend", boot.qrSvg);
    }
    $("#pairLead").textContent = `Open the camera on your phone and scan this code. Or open ${boot.baseUrl} on your phone and use the link from this window.`;
  }

  function renderStats() {
    const boot = state.boot;
    $("#statAddress").textContent = boot.lanIp || "—";
    $("#statPort").textContent = String(boot.port);
    $("#statPhones").textContent = String(boot.connected.length);
    $("#statUptime").textContent = formatUptime(boot.uptimeSecs);
    $("#inboxPath").textContent = boot.inboxDir;
    $("#inboxDir").textContent = boot.inboxDir;
    $("#dataDir").textContent = boot.dataDir;
  }

  function renderDevices() {
    const list = $("#deviceList");
    const devices = state.boot.connected || [];
    if (!devices.length) {
      list.innerHTML = `<div class="device-empty">No phone connected right now.</div>`;
      $("#securityDevices").textContent = "No phones connected.";
      $("#securityCount").textContent = "0";
      return;
    }
    list.innerHTML = devices
      .map(
        (device) => `<div class="device-chip"><span class="ico">${icon(
          "smartphone",
          16
        )}</span><span>${esc(device.name)}</span><span class="device-meta">${esc(
          device.ip
        )} · ${timeAgo(device.connectedAt)}</span></div>`
      )
      .join("");
    $("#securityDevices").textContent = devices
      .map((d) => `${d.name} (${d.ip})`)
      .join(", ");
    $("#securityCount").textContent = String(devices.length);
  }

  function renderShares() {
    const items = state.boot.shares || [];
    $("#shareList").innerHTML = items.map((item) => itemRow(item, "share")).join("");
    $("#shareEmpty").hidden = items.length > 0;
  }

  function renderReceived() {
    const items = state.boot.received || [];
    $("#receivedList").innerHTML = items.map((item) => itemRow(item, "received")).join("");
    $("#receivedEmpty").hidden = items.length > 0;
  }

  function itemRow(item, mode) {
    const meta = item.isDir
      ? "Folder · downloads as .zip"
      : `${bytes(item.size)} · ${KIND_LABEL[item.kind] || "File"}`;
    const actions =
      mode === "share"
        ? `<button class="icon-btn small" data-action="reveal" data-id="${item.id}" aria-label="Show in Explorer" title="Show in Explorer">${icon(
            "external",
            15
          )}</button><button class="icon-btn small" data-action="remove-share" data-id="${
            item.id
          }" aria-label="Stop sharing" title="Stop sharing">${icon("x", 15)}</button>`
        : `<button class="btn ghost small" data-action="open" data-id="${item.id}">Open</button><button class="icon-btn small" data-action="reveal" data-id="${item.id}" aria-label="Show in Explorer" title="Show in Explorer">${icon(
            "external",
            15
          )}</button>`;
    return `<div class="item"><span class="item-icon ${item.kind}">${kindIcon(
      item.kind
    )}</span><div class="item-main"><div class="item-name" title="${esc(item.name)}">${esc(
      item.name
    )}</div><div class="item-meta">${meta}</div></div><div class="item-actions">${actions}</div></div>`;
  }

  function renderClipboard() {
    const clip = state.boot.clipboard || { kind: "empty", files: [], at: 0 };
    const badge = $("#clipKind");
    const preview = $("#clipPreview");
    const updated = $("#clipUpdated");
    badge.className = "badge";
    if (clip.kind === "empty") {
      badge.textContent = "Empty";
      preview.innerHTML = `<p class="muted">Copy something on your PC, then send it to your phone.</p>`;
      updated.textContent = "Reading clipboard…";
      return;
    }
    updated.textContent = `Updated ${timeAgo(clip.at)}`;
    if (clip.kind === "text") {
      badge.textContent = "Text";
      badge.classList.add("accent");
      preview.textContent = clip.text || "";
    } else if (clip.kind === "image") {
      badge.textContent = "Image";
      badge.classList.add("accent");
      const file = (clip.files || [])[0];
      if (!file) {
        preview.innerHTML = `<p class="muted">Image unavailable.</p>`;
        return;
      }
      const cached = state.previewCache.get(file.id);
      if (cached) {
        preview.innerHTML = `<img src="${cached}" alt="Clipboard image" />`;
      } else {
        preview.innerHTML = `<p class="muted">Loading preview…</p>`;
        invoke("item_preview", { id: file.id })
          .then((dataUrl) => {
            state.previewCache.set(file.id, dataUrl);
            if (state.boot.clipboard.files?.[0]?.id === file.id) {
              $("#clipPreview").innerHTML = `<img src="${dataUrl}" alt="Clipboard image" />`;
            }
          })
          .catch(() => {
            preview.innerHTML = `<p class="muted">${esc(file.name)} — preview unavailable.</p>`;
          });
      }
    } else {
      badge.textContent = "Files";
      badge.classList.add("accent");
      preview.innerHTML = `<div class="mini">${(clip.files || [])
        .map(
          (file) =>
            `<div class="row"><span class="ico">${kindIcon(
              file.kind
            )}</span><span class="grow">${esc(file.name)}</span><span class="faint">${bytes(
              file.size
            )}</span></div>`
        )
        .join("")}</div>`;
    }
  }

  function activityIcon(kind) {
    switch (kind) {
      case "share":
        return "folder";
      case "receive":
        return "inbox";
      case "power":
        return "power";
      case "connect":
        return "smartphone";
      case "settings":
        return "settings";
      default:
        return "activity";
    }
  }

  function renderActivity() {
    const items = state.boot.activity || [];
    const list = $("#activityList");
    $("#activityEmpty").hidden = items.length > 0;
    if (!items.length) {
      list.innerHTML = "";
      return;
    }
    list.innerHTML = items
      .slice(0, 9)
      .map((entry) => {
        const tone = entry.kind === "power" ? "power" : entry.ok ? "success" : "error";
        return `<div class="activity-row"><span class="activity-icon ${tone}">${icon(
          activityIcon(entry.kind),
          15
        )}</span><div class="activity-main"><div class="activity-title">${esc(
          entry.title
        )}</div>${entry.detail ? `<div class="activity-detail">${esc(entry.detail)}</div>` : ""}</div><span class="activity-time">${timeAgo(
          entry.at
        )}</span></div>`;
      })
      .join("");
  }

  function renderSettings() {
    const boot = state.boot;
    setInput("#deviceName", boot.deviceName);
    setInput("#portInput", String(boot.configuredPort ?? boot.port));
    $("#autostartToggle").checked = !!boot.autostart;
    $("#startMinimizedToggle").checked = !!boot.startMinimized;
    $("#clipboardSyncToggle").checked = !!boot.clipboardSync;
    $("#clipSyncToggle").checked = !!boot.clipboardSync;
    $("#syncBadge").textContent = boot.clipboardSync ? "Auto-sync on" : "Auto-sync off";
    $("#syncBadge").className = `badge ${boot.clipboardSync ? "success" : ""}`;
    $("#restartBanner").hidden = !state.restartRequired;
    $$("[data-theme-choice]").forEach((button) => {
      button.setAttribute("aria-selected", button.dataset.themeChoice === state.themeChoice ? "true" : "false");
    });
    const labels = { system: "System theme", light: "Light theme", dark: "Dark theme" };
    $("#sideThemeLabel").textContent = labels[state.themeChoice] || "System theme";
    $("#sideTheme .ico").innerHTML = icon(
      state.themeChoice === "dark" ? "sun" : "moon",
      16
    );
  }

  function setInput(selector, value) {
    const input = $(selector);
    if (!input) return;
    if (document.activeElement === input) return;
    if (input.value !== value) input.value = value;
  }

  /* ---------------- Theme ---------------- */

  const themeMedia = window.matchMedia("(prefers-color-scheme: dark)");

  function applyTheme(choice, persist = true) {
    state.themeChoice = choice;
    const resolved = choice === "system" ? (themeMedia.matches ? "dark" : "light") : choice;
    document.documentElement.setAttribute("data-theme", resolved);
    try {
      localStorage.setItem("lumo.theme", choice);
    } catch (_) {}
    if (persist && state.boot) {
      invoke("update_settings", { patch: { theme: choice } }).catch((err) =>
        toast(String(err), "error")
      );
    }
    renderSettings();
  }

  themeMedia.addEventListener("change", () => {
    if (state.themeChoice === "system") {
      const resolved = themeMedia.matches ? "dark" : "light";
      document.documentElement.setAttribute("data-theme", resolved);
    }
  });

  /* ---------------- Toasts / modal ---------------- */

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

  function confirmModal({ title, body, confirmLabel = "Confirm", tone = "danger" }) {
    $("#modalTitle").textContent = title;
    $("#modalBody").textContent = body;
    const button = $("#modalConfirm");
    button.textContent = confirmLabel;
    button.className = `btn ${tone === "danger" ? "danger" : "primary"}`;
    $("#modalBackdrop").hidden = false;
    button.focus();
    return new Promise((resolve) => {
      state.modalResolver = resolve;
    });
  }

  function settleModal(value) {
    $("#modalBackdrop").hidden = true;
    if (state.modalResolver) {
      const resolve = state.modalResolver;
      state.modalResolver = null;
      resolve(value);
    }
  }

  /* ---------------- Actions ---------------- */

  function switchView(view) {
    state.view = view;
    $$(".page").forEach((page) => {
      page.hidden = page.dataset.page !== view;
    });
    $$(".nav-item").forEach((item) => {
      if (item.dataset.nav === view) item.setAttribute("aria-current", "page");
      else item.removeAttribute("aria-current");
    });
    const meta = PAGE_META[view];
    $("#pageTitle").textContent = meta.title;
    $("#pageSub").textContent = meta.sub;
    $("#content").scrollTop = 0;
    if (view === "files") {
      renderShares();
      renderReceived();
    }
  }

  async function shareFiles(kind) {
    try {
      const added = await invoke("pick_and_share", { kind });
      await refresh();
      if (Array.isArray(added) && added.length) {
        toast(`Shared ${added.length} item${added.length === 1 ? "" : "s"}`, "success");
      }
    } catch (err) {
      toast(String(err), "error");
    }
  }

  async function saveSettings(patch, successMessage = "Saved") {
    try {
      const result = await invoke("update_settings", { patch });
      if (result && result.restartRequired) state.restartRequired = true;
      await refresh();
      if (successMessage) toast(successMessage, "success");
      return true;
    } catch (err) {
      toast(String(err), "error");
      return false;
    }
  }

  async function sendClipboardToPhone() {
    try {
      const clip = await invoke("send_clipboard_to_phone");
      state.boot.clipboard = clip;
      renderClipboard();
      toast("Clipboard sent to your phone", "success");
    } catch (err) {
      toast(String(err), "error");
    }
  }

  async function copyCurrentClipboard() {
    const clip = state.boot.clipboard;
    try {
      if (clip.kind === "text" && clip.text) {
        await invoke("copy_text_to_clipboard", { text: clip.text });
        toast("Copied", "success");
      } else if (clip.files?.[0]) {
        await invoke("copy_item_to_clipboard", { id: clip.files[0].id });
        toast("Copied file to clipboard", "success");
      } else {
        toast("Nothing in the clipboard to copy");
      }
    } catch (err) {
      toast(String(err), "error");
    }
  }

  /* ---------------- Bindings ---------------- */

  function bind() {
    hydrateIcons();

    $$(".nav-item").forEach((item) => {
      item.addEventListener("click", () => switchView(item.dataset.nav));
    });

    $("#btnShareFiles").addEventListener("click", () => shareFiles("files"));
    $("#btnOpenInbox").addEventListener("click", () => invoke("open_path", { path: state.boot.inboxDir }));
    $("#addFiles").addEventListener("click", () => shareFiles("files"));
    $("#addFolder").addEventListener("click", () => shareFiles("folder"));
    $("#overviewDrop").addEventListener("click", () => shareFiles("files"));
    $("#dropHint").addEventListener("click", () => shareFiles("files"));
    $("#copyPairUrl").addEventListener("click", () => copyText(state.boot.pairUrl, "Pairing link copied"));
    $("#copyBaseUrl").addEventListener("click", () => copyText(state.boot.baseUrl, "Address copied"));
    $("#regenToken").addEventListener("click", resetPairing);
    $("#resetPair").addEventListener("click", resetPairing);

    $("#shareList").addEventListener("click", async (event) => {
      const button = event.target.closest("[data-action]");
      if (!button) return;
      const { action, id } = button.dataset;
      try {
        if (action === "remove-share") await invoke("remove_share", { id });
        if (action === "reveal") await invoke("reveal_item", { id });
        await refresh();
      } catch (err) {
        toast(String(err), "error");
      }
    });

    $("#receivedList").addEventListener("click", async (event) => {
      const button = event.target.closest("[data-action]");
      if (!button) return;
      try {
        if (button.dataset.action === "open") await invoke("open_item", { id: button.dataset.id });
        if (button.dataset.action === "reveal") await invoke("reveal_item", { id: button.dataset.id });
      } catch (err) {
        toast(String(err), "error");
      }
    });

    $("#clearReceived").addEventListener("click", async () => {
      await invoke("clear_received").catch((err) => toast(String(err), "error"));
      refresh();
    });

    $("#clearActivity").addEventListener("click", async () => {
      await invoke("clear_activity").catch((err) => toast(String(err), "error"));
      refresh();
    });

    $("#sendClipboard").addEventListener("click", sendClipboardToPhone);
    $("#refreshClipboard").addEventListener("click", async () => {
      try {
        const clip = await invoke("refresh_clipboard");
        state.boot.clipboard = clip;
        renderClipboard();
        toast("Clipboard refreshed", "success");
      } catch (err) {
        toast(String(err), "error");
      }
    });
    $("#copyClipboard").addEventListener("click", copyCurrentClipboard);
    $("#sendTextToPhone").addEventListener("click", async () => {
      const text = $("#phoneText").value;
      if (!text.trim()) {
        toast("Type something to send first");
        return;
      }
      try {
        await invoke("send_text_to_phone", { text });
        $("#phoneText").value = "";
        toast("Text sent to your phone", "success");
      } catch (err) {
        toast(String(err), "error");
      }
    });

    $("#clipSyncToggle").addEventListener("change", (event) => {
      saveSettings({ clipboardSync: event.target.checked }, event.target.checked ? "Auto-sync on" : "Auto-sync off");
    });
    $("#clipboardSyncToggle").addEventListener("change", (event) => {
      saveSettings({ clipboardSync: event.target.checked }, event.target.checked ? "Auto-sync on" : "Auto-sync off");
    });

    $("#autostartToggle").addEventListener("change", (event) => {
      saveSettings({ autostart: event.target.checked }, event.target.checked ? "Starts with Windows" : "Autostart off");
    });

    $("#startMinimizedToggle").addEventListener("change", (event) => {
      saveSettings(
        { startMinimized: event.target.checked },
        event.target.checked ? "Starts minimized" : "Window opens at sign-in"
      );
    });

    $("#deviceName").addEventListener("change", (event) => {
      const name = event.target.value.trim();
      if (!name) {
        event.target.value = state.boot.deviceName;
        return;
      }
      saveSettings({ deviceName: name }, "Name updated");
    });

    $("#portInput").addEventListener("change", async (event) => {
      const port = Number(event.target.value);
      if (!Number.isInteger(port) || port < 1024 || port > 65535) {
        toast("Port must be between 1024 and 65535", "error");
        event.target.value = String(state.boot.port);
        return;
      }
      const ok = await saveSettings({ port }, "Port saved");
      if (!ok) event.target.value = String(state.boot.port);
    });

    $("#changeInbox").addEventListener("click", async () => {
      const dir = await invoke("pick_inbox_dir");
      if (dir) await saveSettings({ inboxDir: dir }, "Inbox folder updated");
    });

    $("#openInbox").addEventListener("click", () => invoke("open_path", { path: state.boot.inboxDir }));
    $("#openLog").addEventListener("click", () => invoke("open_path", { path: state.boot.logPath }));
    $("#openData").addEventListener("click", () => invoke("open_path", { path: state.boot.dataDir }));
    $("#relaunch").addEventListener("click", () => invoke("relaunch"));
    $("#quitApp").addEventListener("click", async () => {
      const ok = await confirmModal({
        title: "Quit Lumo?",
        body: "The phone server stops and your phone disconnects. Lumo stays quit until you start it again.",
        confirmLabel: "Quit Lumo",
      });
      if (ok) invoke("quit_app");
    });

    $$("[data-theme-choice]").forEach((button) => {
      button.addEventListener("click", () => applyTheme(button.dataset.themeChoice));
    });

    $("#sideTheme").addEventListener("click", () => {
      const order = ["system", "light", "dark"];
      const next = order[(order.indexOf(state.themeChoice) + 1) % order.length];
      applyTheme(next);
    });

    $("#copyPair").addEventListener("click", () => copyText(state.boot.pairUrl, "Pairing link copied"));

    $("#modalCancel").addEventListener("click", () => settleModal(false));
    $("#modalConfirm").addEventListener("click", () => settleModal(true));
    $("#modalBackdrop").addEventListener("click", (event) => {
      if (event.target === $("#modalBackdrop")) settleModal(false);
    });
    document.addEventListener("keydown", (event) => {
      if (event.key === "Escape" && !$("#modalBackdrop").hidden) settleModal(false);
    });

    document.addEventListener("visibilitychange", () => {
      if (!document.hidden) refresh(true);
    });

    // The Rust side handles dropped paths; just refresh when it reports them.
    listen("lumo-refresh", () => scheduleRefresh());
    listen("lumo-activity", (event) => {
      if (!state.boot) return;
      state.boot.activity.unshift(event.payload);
      state.boot.activity = state.boot.activity.slice(0, 80);
      renderActivity();
    });
    listen("lumo-toast", (event) => {
      if (event.payload?.message) toast(event.payload.message, event.payload.tone || "neutral");
    });

    setInterval(() => {
      if (!document.hidden) refresh(true);
    }, 8000);
  }

  async function copyText(text, message) {
    try {
      await invoke("copy_text_to_clipboard", { text });
      toast(message, "success");
    } catch (err) {
      toast(String(err), "error");
    }
  }

  async function resetPairing() {
    const ok = await confirmModal({
      title: "Reset the pairing link?",
      body: "Every phone using Lumo will have to scan the new QR code. The server keeps running.",
      confirmLabel: "Reset link",
    });
    if (!ok) return;
    try {
      await invoke("regenerate_token");
      await refresh();
      toast("New pairing link generated", "success");
    } catch (err) {
      toast(String(err), "error");
    }
  }

  /* ---------------- Boot ---------------- */

  bind();
  switchView("overview");
  load();

  window.__lumoDesktop = { refresh, toast };
})();
