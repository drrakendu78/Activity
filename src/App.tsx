import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import Dashboard from "./pages/Dashboard";
import AppConfigs from "./pages/AppConfigs";
import Settings from "./pages/Settings";
import StatusIndicator from "./components/StatusIndicator";
import SetupWizard from "./components/SetupWizard";
import { useTheme } from "./lib/theme";
import { listen } from "@tauri-apps/api/event";
import { getRpcStatus, type RpcStatusEvent } from "./lib/commands";

const isTauri = "__TAURI_INTERNALS__" in window;

const tauriMinimize = () => {
  if (isTauri) import("@tauri-apps/api/window").then(m => m.getCurrentWindow().minimize());
};
const tauriHide = () => {
  if (isTauri) import("@tauri-apps/api/window").then(m => m.getCurrentWindow().hide());
};

type Tab = "dashboard" | "apps" | "settings";

export default function App() {
  const [tab, setTab] = useState<Tab>("dashboard");
  const [connected, setConnected] = useState(false);
  const [setupDone, setSetupDone] = useState(() => localStorage.getItem("setup_done") === "true");
  const { t } = useTranslation();
  useTheme();

  useEffect(() => {
    getRpcStatus().then((s) => setConnected(s.connected));
    const unlisten = listen<RpcStatusEvent>("rpc-status-changed", (event) => {
      setConnected(event.payload.connected);
    });
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  const NAV_ITEMS: { id: Tab; labelKey: string; icon: JSX.Element }[] = [
    {
      id: "dashboard",
      labelKey: "nav.dashboard",
      icon: (
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <rect x="3" y="3" width="7" height="7" rx="2" />
          <rect x="14" y="3" width="7" height="7" rx="2" />
          <rect x="3" y="14" width="7" height="7" rx="2" />
          <rect x="14" y="14" width="7" height="7" rx="2" />
        </svg>
      ),
    },
    {
      id: "apps",
      labelKey: "nav.apps",
      icon: (
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <path d="M12 2L2 7l10 5 10-5-10-5z" />
          <path d="M2 17l10 5 10-5" />
          <path d="M2 12l10 5 10-5" />
        </svg>
      ),
    },
    {
      id: "settings",
      labelKey: "nav.settings",
      icon: (
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <circle cx="12" cy="12" r="3" />
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
        </svg>
      ),
    },
  ];

  return (
    <div className="app-shell">
      {/* Unified titlebar + tabs */}
      <header data-tauri-drag-region className="unified-bar">
        <div className="unified-bar-tabs">
          {NAV_ITEMS.map((item) => (
            <button
              key={item.id}
              onClick={() => setTab(item.id)}
              className={`tab-bar-item ${tab === item.id ? "active" : ""}`}
            >
              {item.icon}
              {t(item.labelKey)}
            </button>
          ))}
        </div>
        <div className="unified-bar-right">
          <div className="tab-bar-status">
            <StatusIndicator connected={connected} />
            <span className="tab-bar-status-text">
              {connected ? t("sidebar.connected") : t("sidebar.disconnected")}
            </span>
          </div>
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
        <div className="page-enter" key={tab}>
          {tab === "dashboard" && <Dashboard />}
          {tab === "apps" && <AppConfigs />}
          {tab === "settings" && <Settings />}
        </div>
      </main>

      {/* First-launch setup wizard */}
      {!setupDone && (
        <SetupWizard onComplete={() => {
          localStorage.setItem("setup_done", "true");
          setSetupDone(true);
        }} />
      )}
    </div>
  );
}
