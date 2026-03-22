import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import Dashboard from "./pages/Dashboard";
import AppConfigs from "./pages/AppConfigs";
import Settings from "./pages/Settings";
import StatusIndicator from "./components/StatusIndicator";
import SetupWizard from "./components/SetupWizard";
import { useTheme } from "./lib/theme";
import { listen } from "@tauri-apps/api/event";
import {
  getRpcStatus, startPoller, stopPoller, isPollerRunning,
  loadConfig, checkForUpdates, startSilentUpdate,
  type RpcStatusEvent, type UpdateInfo,
} from "./lib/commands";

const isTauri = "__TAURI_INTERNALS__" in window;

const tauriMinimize = () => {
  if (isTauri) import("@tauri-apps/api/window").then(m => m.getCurrentWindow().minimize());
};
const tauriHide = () => {
  if (isTauri) import("@tauri-apps/api/window").then(m => m.getCurrentWindow().hide());
};

const NORMAL_HEIGHT = 520;
const SETTINGS_HEIGHT = 780;

let resizeLock: Promise<void> = Promise.resolve();

export const resizeWindow = (targetHeight: number) => {
  resizeLock = resizeLock.then(() => doResize(targetHeight)).catch(() => {});
};

const doResize = async (targetHeight: number) => {
  if (!isTauri) return;
  const { getCurrentWindow, LogicalSize, LogicalPosition } = await import("@tauri-apps/api/window");
  const win = getCurrentWindow();
  const scale = await win.scaleFactor();
  const startPos = await win.outerPosition();
  const startSize = await win.outerSize();
  // Convert physical pixels to logical pixels
  const startH = Math.round(startSize.height / scale);
  const startY = Math.round(startPos.y / scale);
  const startX = Math.round(startPos.x / scale);
  const deltaH = targetHeight - startH;
  if (Math.abs(deltaH) < 2) return;

  const duration = 150;
  const steps = 10;
  const stepMs = duration / steps;

  for (let i = 1; i <= steps; i++) {
    const t = i / steps;
    const ease = 1 - Math.pow(1 - t, 3);
    const h = Math.round(startH + deltaH * ease);
    const y = Math.max(0, startY - Math.round((deltaH * ease) / 2));
    await win.setSize(new LogicalSize(750, h));
    await win.setPosition(new LogicalPosition(startX, y));
    if (i < steps) await new Promise(r => setTimeout(r, stepMs));
  }
};

type View = "home" | "apps";

export default function App() {
  const [view, setView] = useState<View>("home");
  const [connected, setConnected] = useState(false);
  const [running, setRunning] = useState(false);
  const [loading, setLoading] = useState(false);
  const [waitingForDiscord, setWaitingForDiscord] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [closingSettings, setClosingSettings] = useState(false);
  const [setupDone, setSetupDone] = useState(() => localStorage.getItem("setup_done") === "true");
  const [albumBg, setAlbumBg] = useState<string | null>(null);
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [showUpdatePopup, setShowUpdatePopup] = useState(false);
  const [updateDownloading, setUpdateDownloading] = useState(false);
  const [updateError, setUpdateError] = useState<string | null>(null);

  const openSettings = () => {
    setShowSettings(true);
    resizeWindow(SETTINGS_HEIGHT);
  };
  const closeSettings = () => {
    setClosingSettings(true);
    resizeWindow(NORMAL_HEIGHT);
    setTimeout(() => { setShowSettings(false); setClosingSettings(false); }, 180);
  };
  const { t } = useTranslation();
  useTheme();

  useEffect(() => {
    getRpcStatus().then((s) => setConnected(s.connected));
    // Auto-check for updates
    checkForUpdates().then((info) => {
      setUpdateInfo(info);
      if (info.has_update) {
        setShowUpdatePopup(true);
        resizeWindow(SETTINGS_HEIGHT);
      }
    }).catch(() => {});
    isPollerRunning().then((isRunning) => {
      setRunning(isRunning);
      // Auto-start service if configured, not already running, and setup is done
      if (!isRunning && localStorage.getItem("setup_done") === "true") {
        loadConfig().then((cfg) => {
          if (cfg.auto_start_service) {
            setWaitingForDiscord(true);
            const tryStart = () => {
              startPoller()
                .then(() => { setRunning(true); setWaitingForDiscord(false); })
                .catch(() => {
                  setTimeout(() => tryStart(), 5000);
                });
            };
            tryStart();
          }
        }).catch(() => {});
      }
    });
    const unlisten = listen<RpcStatusEvent>("rpc-status-changed", (event) => {
      setConnected(event.payload.connected);
      // Album art as background when music is playing
      if (event.payload.is_music && event.payload.large_image_key) {
        setAlbumBg(event.payload.large_image_key);
      } else if (!event.payload.is_music) {
        setAlbumBg(null);
      }
    });
    // Listen for tray events
    const unlistenNav = listen<string>("navigate", (event) => {
      if (event.payload === "settings") {
        openSettings();
      }
    });
    const unlistenUpdate = listen("check-update", () => {
      checkForUpdates().then((info) => {
        setUpdateInfo(info);
        if (info.has_update) {
          setShowUpdatePopup(true);
          resizeWindow(SETTINGS_HEIGHT);
        }
      }).catch(() => {});
    });
    const unlistenService = listen<boolean>("service-changed", (event) => {
      setRunning(event.payload);
      if (!event.payload) {
        setConnected(false);
      }
    });
    return () => {
      unlisten.then((fn) => fn());
      unlistenNav.then((fn) => fn());
      unlistenUpdate.then((fn) => fn());
      unlistenService.then((fn) => fn());
    };
  }, []);

  const handleToggle = async () => {
    setLoading(true);
    try {
      if (running) {
        await stopPoller();
        setRunning(false);
        setConnected(false);
        setWaitingForDiscord(false);
      } else {
        try {
          await startPoller();
          setRunning(true);
        } catch {
          // Discord probably not running — start retry loop
          setWaitingForDiscord(true);
          const tryStart = () => {
            startPoller()
              .then(() => { setRunning(true); setWaitingForDiscord(false); })
              .catch(() => {
                setTimeout(() => tryStart(), 5000);
              });
          };
          tryStart();
        }
      }
    } finally { setLoading(false); }
  };

  return (
    <div
      className={`app-shell ${albumBg ? "album-active" : ""}`}
      style={{ "--album-url": albumBg ? `url(${albumBg})` : "none" } as React.CSSProperties}
    >
      {/* Top bar */}
      <header data-tauri-drag-region className="unified-bar">
        {/* Left: back button (when on apps) + service toggle */}
        <div className="bar-left">
          {view === "apps" && (
            <button onClick={() => setView("home")} className="bar-icon-btn" title={t("nav.dashboard")}>
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                <polyline points="15 18 9 12 15 6" />
              </svg>
            </button>
          )}
          <button
            onClick={handleToggle}
            disabled={loading}
            className={`service-toggle ${running ? "on" : "off"}`}
          >
            <div className="service-toggle-dot" />
            <span>{running ? t("dashboard.stop") : t("dashboard.start")}</span>
          </button>
          <div className="bar-status">
            <StatusIndicator connected={connected} />
            <span className="bar-status-text">
              {connected ? t("sidebar.connected") : t("sidebar.disconnected")}
            </span>
          </div>
        </div>

        {/* Right: apps button + settings + window controls */}
        <div className="bar-right">
          <button
            onClick={() => setView(view === "apps" ? "home" : "apps")}
            className={`bar-text-btn ${view === "apps" ? "active" : ""}`}
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M12 2L2 7l10 5 10-5-10-5z" />
              <path d="M2 17l10 5 10-5" />
              <path d="M2 12l10 5 10-5" />
            </svg>
            {t("nav.apps")}
          </button>
          <button
            onClick={openSettings}
            className="bar-icon-btn"
            title={t("nav.settings")}
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <circle cx="12" cy="12" r="3" />
              <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
            </svg>
          </button>
          <div className="titlebar-btns">
            <button onClick={tauriMinimize} className="titlebar-btn">
              <svg width="10" height="2" viewBox="0 0 10 2">
                <rect width="10" height="1.5" rx="0.75" fill="currentColor" />
              </svg>
            </button>
            <button onClick={tauriHide} className="titlebar-btn titlebar-btn-close">
              <svg width="10" height="10" viewBox="0 0 10 10">
                <line x1="1.5" y1="1.5" x2="8.5" y2="8.5" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" />
                <line x1="8.5" y1="1.5" x2="1.5" y2="8.5" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" />
              </svg>
            </button>
          </div>
        </div>
      </header>

      {/* Content */}
      <main className="main-content">
        <div style={{ display: view === "home" ? "block" : "none" }}>
          <Dashboard waitingForDiscord={waitingForDiscord} />
        </div>
        <div style={{ display: view === "apps" ? "block" : "none" }}>
          <AppConfigs />
        </div>
      </main>

      {/* Settings modal */}
      {showSettings && (
        <div className="settings-overlay" style={{
          animation: closingSettings ? "backdropOut 0.18s ease forwards" : undefined,
        }} onClick={(e) => {
          if (e.target === e.currentTarget) closeSettings();
        }}>
          <div className="settings-modal" style={{
            animation: closingSettings ? "scaleOut 0.18s cubic-bezier(0.4, 0, 0.2, 1) forwards" : undefined,
          }}>
            <div className="settings-modal-header">
              <h2 style={{ fontSize: 16, fontWeight: 600, color: "var(--text-1)" }}>
                {t("nav.settings")}
              </h2>
              <button onClick={closeSettings} className="settings-close-btn">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                  <line x1="18" y1="6" x2="6" y2="18" />
                  <line x1="6" y1="6" x2="18" y2="18" />
                </svg>
              </button>
            </div>
            <div className="settings-modal-body">
              <Settings />
            </div>
          </div>
        </div>
      )}

      {/* Update popup */}
      {showUpdatePopup && updateInfo?.has_update && (
        <div className="settings-overlay" onClick={(e) => {
          if (e.target === e.currentTarget) { setShowUpdatePopup(false); resizeWindow(NORMAL_HEIGHT); };
        }}>
          <div className="update-popup">
            <div className="update-popup-icon">
              <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="var(--accent)" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                <polyline points="7 10 12 15 17 10" />
                <line x1="12" y1="15" x2="12" y2="3" />
              </svg>
            </div>
            <h3 className="update-popup-title">{t("update.available")}</h3>
            <p className="update-popup-version">
              v{updateInfo.current_version} → v{updateInfo.latest_version}
            </p>
            {updateInfo.release_notes && (
              <div className="update-popup-notes">
                {updateInfo.release_notes}
              </div>
            )}
            <div className="update-popup-meta">
              {updateInfo.file_name && <span>{updateInfo.file_name}</span>}
              {updateInfo.file_size && <span>{(updateInfo.file_size / 1024 / 1024).toFixed(1)} Mo</span>}
              <span>{updateInfo.install_type.toUpperCase()}</span>
            </div>
            {updateError && (
              <div style={{ color: "#ef4444", fontSize: 11, marginTop: 8 }}>{updateError}</div>
            )}
            <div className="update-popup-buttons">
              <button
                onClick={() => { setShowUpdatePopup(false); resizeWindow(NORMAL_HEIGHT); }}
                className="btn-secondary"
                style={{ flex: 1, padding: "8px 0", fontSize: 12, borderRadius: 8 }}
              >
                {t("update.later")}
              </button>
              {updateInfo.download_url && updateInfo.file_name && (
                <button
                  onClick={async () => {
                    if (!updateInfo.download_url || !updateInfo.file_name) return;
                    setUpdateDownloading(true);
                    setUpdateError(null);
                    try {
                      await startSilentUpdate(
                        updateInfo.download_url,
                        updateInfo.file_name,
                        updateInfo.file_sha256 ?? undefined
                      );
                    } catch (e) {
                      setUpdateError(String(e));
                      setUpdateDownloading(false);
                    }
                  }}
                  className="btn-primary"
                  style={{ flex: 1, padding: "8px 0", fontSize: 12, borderRadius: 8, opacity: updateDownloading ? 0.6 : 1 }}
                  disabled={updateDownloading}
                >
                  {updateDownloading ? t("update.downloading") : t("update.updateNow")}
                </button>
              )}
            </div>
          </div>
        </div>
      )}

      {/* First-launch setup wizard */}
      {!setupDone && (
        <SetupWizard onComplete={(serviceStarted) => {
          localStorage.setItem("setup_done", "true");
          setSetupDone(true);
          if (serviceStarted) {
            setRunning(true);
          }
        }} />
      )}
    </div>
  );
}
