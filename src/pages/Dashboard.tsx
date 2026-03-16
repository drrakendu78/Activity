import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { listen } from "@tauri-apps/api/event";
import {
  startPoller,
  stopPoller,
  isPollerRunning,
  getRpcStatus,
  type RpcStatusEvent,
} from "../lib/commands";
import IconPicker from "../components/IconPicker";

const CATEGORY_EMOJI: Record<string, string> = {
  gaming: "\u{1F3AE}", dev: "\u{1F4BB}", browser: "\u{1F310}", social: "\u{1F4AC}",
  media: "\u{1F3B5}", creative: "\u{1F3A8}", productivity: "\u{1F4C4}", system: "\u{2699}\u{FE0F}",
  discovered: "\u{1F50D}", other: "\u{1F4E6}",
};

function formatElapsed(startTimestamp: number | null): string {
  if (!startTimestamp) return "";
  const now = Math.floor(Date.now() / 1000);
  const elapsed = now - startTimestamp;
  if (elapsed < 0) return "";
  const h = Math.floor(elapsed / 3600);
  const m = Math.floor((elapsed % 3600) / 60);
  const s = elapsed % 60;
  if (h > 0) return `${h}h ${m.toString().padStart(2, "0")}m`;
  return `${m}m ${s.toString().padStart(2, "0")}s`;
}

export default function Dashboard() {
  const { t } = useTranslation();
  const [running, setRunning] = useState(false);
  const [connected, setConnected] = useState(false);
  const [currentApp, setCurrentApp] = useState<string | null>(null);
  const [exeName, setExeName] = useState<string | null>(null);
  const [windowTitle, setWindowTitle] = useState<string | null>(null);
  const [category, setCategory] = useState<string | null>(null);
  const [details, setDetails] = useState<string | null>(null);
  const [rpcState, setRpcState] = useState<string | null>(null);
  const [largeImageKey, setLargeImageKey] = useState<string | null>(null);
  const [startTimestamp, setStartTimestamp] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [elapsed, setElapsed] = useState("");
  const [showIconPicker, setShowIconPicker] = useState(false);

  useEffect(() => {
    isPollerRunning().then(setRunning);
    getRpcStatus().then((s) => {
      setConnected(s.connected);
      setCurrentApp(s.current_app);
    });
    const unlisten = listen<RpcStatusEvent>("rpc-status-changed", (event) => {
      setConnected(event.payload.connected);
      setCurrentApp(event.payload.current_app);
      setExeName(event.payload.exe_name);
      setWindowTitle(event.payload.window_title);
      setCategory(event.payload.category);
      setDetails(event.payload.details);
      setRpcState(event.payload.state);
      setLargeImageKey(event.payload.large_image_key);
      setStartTimestamp(event.payload.start_timestamp);
    });
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  useEffect(() => {
    if (!startTimestamp || !running) { setElapsed(""); return; }
    setElapsed(formatElapsed(startTimestamp));
    const interval = setInterval(() => setElapsed(formatElapsed(startTimestamp)), 1000);
    return () => clearInterval(interval);
  }, [startTimestamp, running]);

  const handleToggle = async () => {
    setError(null);
    setLoading(true);
    try {
      if (running) {
        await stopPoller();
        setRunning(false); setConnected(false); setCurrentApp(null);
        setExeName(null); setWindowTitle(null); setCategory(null);
        setDetails(null); setRpcState(null); setLargeImageKey(null);
        setStartTimestamp(null);
      } else {
        await startPoller();
        setRunning(true);
      }
    } catch (e) { setError(String(e)); }
    finally { setLoading(false); }
  };

  const emoji = CATEGORY_EMOJI[category || "other"] || "\u{1F4E6}";
  const hasIcon = largeImageKey && largeImageKey.startsWith("http");

  return (
    <div style={{ padding: 24, display: "flex", flexDirection: "column", gap: 16 }}>
      {/* Header */}
      <div>
        <h1 style={{ fontSize: 22, fontWeight: 700, letterSpacing: "-0.02em", color: "var(--text-1)" }}>
          {t("dashboard.title")}
        </h1>
        <p style={{ fontSize: 13, color: "var(--text-2)", marginTop: 2 }}>
          {t("dashboard.subtitle")}
        </p>
      </div>

      {/* Status Card */}
      <div className="card" style={{ padding: 16 }}>
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
          <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
            <div style={{
              width: 40, height: 40, borderRadius: "50%", display: "flex",
              alignItems: "center", justifyContent: "center",
              background: running ? "var(--green-bg)" : "var(--red-bg)",
            }}>
              <div
                className={running ? "status-dot-active" : "status-dot-stopped"}
                style={{
                  width: 12, height: 12, borderRadius: "50%",
                  background: running ? "var(--green)" : "var(--red)",
                }}
              />
            </div>
            <div>
              <div style={{ fontSize: 14, fontWeight: 600, color: "var(--text-1)" }}>
                {running ? t("dashboard.serviceRunning") : t("dashboard.serviceStopped")}
              </div>
              <div style={{ fontSize: 12, color: "var(--text-2)", marginTop: 1 }}>
                {running
                  ? connected
                    ? t("dashboard.connectedToDiscord")
                    : t("dashboard.notConnected")
                  : t("dashboard.startServicePrompt")}
              </div>
            </div>
          </div>
          <button
            onClick={handleToggle}
            disabled={loading}
            className={running ? "btn-danger" : "btn-primary"}
            style={{ opacity: loading ? 0.5 : 1, minWidth: 72 }}
          >
            {loading ? "..." : running ? t("dashboard.stop") : t("dashboard.start")}
          </button>
        </div>
        {error && (
          <div style={{
            marginTop: 12, padding: "8px 12px", borderRadius: 8,
            background: "var(--red-bg)", color: "var(--red)", fontSize: 12,
          }}>
            {error}
          </div>
        )}
      </div>

      {/* Current Activity */}
      <div className="card" style={{ padding: 16 }}>
        <div className="section-label" style={{ marginBottom: 12 }}>
          {t("dashboard.currentActivity")}
        </div>

        {running && currentApp ? (
          <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
            <div style={{ display: "flex", alignItems: "flex-start", gap: 14 }}>
              {/* Icon — click to change */}
              <div
                className={`icon-editable${connected ? "" : " breathe-red"}`}
                onClick={() => setShowIconPicker(true)}
                style={{
                  width: 48, height: 48, borderRadius: 12,
                  display: "flex", alignItems: "center", justifyContent: "center",
                  background: hasIcon ? "var(--bg-input)" : "var(--accent-bg)",
                  flexShrink: 0, border: "1px solid var(--border-light)",
                  cursor: "pointer", position: "relative",
                }}
              >
                {hasIcon ? (
                  <img
                    src={largeImageKey!}
                    alt={currentApp}
                    style={{ width: 34, height: 34, objectFit: "contain" }}
                    onError={(e) => {
                      (e.target as HTMLImageElement).style.display = "none";
                      (e.target as HTMLImageElement).parentElement!.innerHTML = `<span style="font-size:24px">${emoji}</span>`;
                    }}
                  />
                ) : (
                  <span style={{ fontSize: 24 }}>{emoji}</span>
                )}
                {/* Always-visible pencil badge */}
                <div className="icon-edit-badge">
                  <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="white" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M17 3a2.83 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z" />
                  </svg>
                </div>
                {/* Hover overlay with pencil */}
                <div className="icon-edit-overlay">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="white" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
                    <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
                  </svg>
                </div>
              </div>

              {/* Info */}
              <div style={{ minWidth: 0, flex: 1, paddingTop: 2 }}>
                <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                  <span style={{ fontSize: 15, fontWeight: 600, color: "var(--text-1)" }}>
                    {currentApp}
                  </span>
                  {category && (
                    <span className="badge badge-gray" style={{ textTransform: "capitalize" }}>
                      {t(`categories.${category}`, category)}
                    </span>
                  )}
                </div>
                {details && (
                  <div style={{ fontSize: 13, color: "var(--text-2)", marginTop: 2 }}>{details}</div>
                )}
                {rpcState && (
                  <div style={{ fontSize: 12, color: "var(--text-3)" }}>{rpcState}</div>
                )}
                {elapsed && (
                  <div style={{ display: "flex", alignItems: "center", gap: 6, marginTop: 6, fontSize: 12, color: "var(--text-2)" }}>
                    <span className="pulse-dot" style={{
                      display: "inline-block", width: 6, height: 6, borderRadius: "50%",
                      background: "var(--green)",
                    }} />
                    {elapsed} {t("dashboard.elapsed")}
                  </div>
                )}
              </div>
            </div>

            {/* Meta */}
            {(exeName || windowTitle) && (
              <div style={{ display: "flex", alignItems: "center", gap: 8, marginLeft: 62 }}>
                {exeName && (
                  <span style={{
                    fontSize: 10, fontFamily: "monospace", padding: "2px 6px",
                    borderRadius: 4, background: "var(--bg-input)", color: "var(--text-3)",
                  }}>
                    {exeName}
                  </span>
                )}
                {windowTitle && (
                  <span style={{
                    fontSize: 10, color: "var(--text-3)",
                    overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", maxWidth: 200,
                  }} title={windowTitle}>
                    {windowTitle}
                  </span>
                )}
              </div>
            )}
          </div>
        ) : (
          <div style={{ textAlign: "center", padding: "24px 0", color: "var(--text-3)", fontSize: 13 }}>
            {running ? t("dashboard.waitingForFocus") : t("dashboard.startServicePrompt")}
          </div>
        )}
      </div>

      {/* Footer */}
      <div style={{ textAlign: "center", fontSize: 11, color: "var(--text-3)", lineHeight: 1.5 }}>
        {t("dashboard.footer1")}<br />
        {t("dashboard.footer2")}
      </div>

      {/* Icon Picker Popup */}
      {showIconPicker && exeName && currentApp && (
        <IconPicker
          exeName={exeName}
          appName={currentApp}
          onClose={() => setShowIconPicker(false)}
          onPicked={(url) => setLargeImageKey(url)}
        />
      )}
    </div>
  );
}
