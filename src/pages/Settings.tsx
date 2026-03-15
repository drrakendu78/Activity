import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  loadConfig, saveConfig, isAutoStartupEnabled,
  enableAutoStartup, disableAutoStartup, type RpcConfig,
} from "../lib/commands";
import { useTheme } from "../lib/theme";
import { LANGUAGES, getFlagUrl } from "../i18n";

export default function Settings() {
  const { t, i18n } = useTranslation();
  const [config, setConfig] = useState<RpcConfig | null>(null);
  const [autoStart, setAutoStart] = useState(false);
  const [saved, setSaved] = useState(false);
  const { theme, toggle: toggleTheme } = useTheme();

  useEffect(() => {
    loadConfig().then(setConfig);
    isAutoStartupEnabled().then(setAutoStart).catch(() => {});
  }, []);

  const handleSave = async () => {
    if (!config) return;
    await saveConfig(config);
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  const handleAutoStart = async () => {
    try {
      if (autoStart) { await disableAutoStartup(); setAutoStart(false); }
      else { await enableAutoStartup(); setAutoStart(true); }
    } catch (e) { console.error(e); }
  };

  const handleLanguage = (code: string) => {
    i18n.changeLanguage(code);
    localStorage.setItem("language", code);
  };

  if (!config) return <div style={{ padding: 24, color: "var(--text-3)" }}>{t("apps.loading")}</div>;

  return (
    <div style={{ padding: 24, display: "flex", flexDirection: "column", gap: 16 }}>
      <div>
        <h1 style={{ fontSize: 22, fontWeight: 700, letterSpacing: "-0.02em", color: "var(--text-1)" }}>{t("settings.title")}</h1>
        <p style={{ fontSize: 13, color: "var(--text-2)", marginTop: 2 }}>{t("settings.subtitle")}</p>
      </div>

      {/* Appearance */}
      <div className="card" style={{ padding: 16 }}>
        <div className="section-label" style={{ marginBottom: 12 }}>{t("settings.appearance")}</div>
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
          <div>
            <div style={{ fontSize: 13, fontWeight: 500, color: "var(--text-1)" }}>{t("settings.darkMode")}</div>
            <div style={{ fontSize: 11, color: "var(--text-3)", marginTop: 1 }}>{t("settings.darkModeDesc")}</div>
          </div>
          <button onClick={toggleTheme} className={`toggle-apple ${theme === "dark" ? "active" : ""}`}>
            <span className="thumb" />
          </button>
        </div>
      </div>

      {/* Language */}
      <div className="card" style={{ padding: 16 }}>
        <div className="section-label" style={{ marginBottom: 12 }}>{t("settings.language")}</div>
        <div>
          <div style={{ fontSize: 13, fontWeight: 500, color: "var(--text-1)", marginBottom: 4 }}>{t("settings.languageLabel")}</div>
          <div style={{ fontSize: 11, color: "var(--text-3)", marginBottom: 10 }}>{t("settings.languageDesc")}</div>
          <div style={{
            display: "grid", gridTemplateColumns: "1fr 1fr", gap: 4,
            maxHeight: 200, overflowY: "auto", overflowX: "hidden", padding: 2,
          }}>
            {LANGUAGES.map((lang) => (
              <button
                key={lang.code}
                onClick={() => handleLanguage(lang.code)}
                style={{
                  display: "flex", alignItems: "center", gap: 8,
                  padding: "6px 10px", borderRadius: 8,
                  border: i18n.language === lang.code ? "2px solid var(--accent)" : "1px solid var(--border)",
                  background: i18n.language === lang.code ? "var(--accent-bg)" : "transparent",
                  cursor: "pointer", fontSize: 12, color: "var(--text-1)",
                  fontWeight: i18n.language === lang.code ? 600 : 400,
                  transition: "all 0.15s", fontFamily: "inherit",
                }}
              >
                <img src={getFlagUrl(lang.country)} alt={lang.country} style={{ width: 20, height: 15, borderRadius: 2, objectFit: "cover" }} />
                <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{lang.name}</span>
              </button>
            ))}
          </div>
        </div>
      </div>


      {/* Polling */}
      <div className="card" style={{ padding: 16 }}>
        <div className="section-label" style={{ marginBottom: 12 }}>{t("settings.polling")}</div>
        <div>
          <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 8 }}>
            <label style={{ fontSize: 12, color: "var(--text-2)" }}>{t("settings.pollInterval")}</label>
            <span style={{ fontSize: 12, fontWeight: 600, color: "var(--text-1)" }}>
              {(config.poll_interval_ms / 1000).toFixed(1)}s
            </span>
          </div>
          <input type="range" min="1000" max="10000" step="500"
            value={config.poll_interval_ms}
            onChange={(e) => setConfig({ ...config, poll_interval_ms: Number(e.target.value) })} />
          <div style={{ display: "flex", justifyContent: "space-between", fontSize: 10, color: "var(--text-3)", marginTop: 4 }}>
            <span>1s</span><span>10s</span>
          </div>
        </div>
      </div>

      {/* Default Presence */}
      <div className="card" style={{ padding: 16 }}>
        <div className="section-label" style={{ marginBottom: 12 }}>{t("settings.defaultPresence")}</div>
        <p style={{ fontSize: 11, color: "var(--text-3)", marginBottom: 14 }}>
          {t("settings.defaultPresenceDesc2")}
        </p>

        {/* Discord-style live preview */}
        <div style={{
          background: "var(--bg-input)", borderRadius: 12, padding: 14,
          display: "flex", gap: 12, alignItems: "flex-start", marginBottom: 16,
          border: "1px solid var(--border)",
        }}>
          <div style={{
            width: 52, height: 52, borderRadius: 10, background: "var(--accent-bg)",
            display: "flex", alignItems: "center", justifyContent: "center", flexShrink: 0,
          }}>
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="var(--accent)" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
              <rect x="3" y="3" width="18" height="18" rx="3" /><circle cx="8.5" cy="8.5" r="1.5" /><path d="m21 15-5-5L5 21" />
            </svg>
          </div>
          <div style={{ minWidth: 0, flex: 1 }}>
            <div style={{ fontSize: 13, fontWeight: 600, color: "var(--text-1)", marginBottom: 2 }}>
              Spotify
            </div>
            <div style={{ fontSize: 11, color: "var(--text-2)" }}>
              {config.default.details.replace("{app_name}", "Spotify")}
            </div>
            {config.default.state && (
              <div style={{ fontSize: 11, color: "var(--text-3)", marginTop: 1 }}>
                {config.default.state}
              </div>
            )}
            <div style={{ fontSize: 10, color: "var(--text-3)", marginTop: 3, opacity: 0.7 }}>
              00:12:34 {t("dashboard.elapsed")}
            </div>
          </div>
        </div>

        {/* Main text — clickable presets */}
        <div style={{ marginBottom: 14 }}>
          <div style={{ fontSize: 12, color: "var(--text-2)", marginBottom: 8, fontWeight: 500 }}>
            {t("settings.presenceStyle")}
          </div>
          <div style={{ display: "flex", flexWrap: "wrap", gap: 6 }}>
            {[
              { label: t("settings.styleUsing"), value: "Using {app_name}" },
              { label: t("settings.stylePlaying"), value: "Playing {app_name}" },
              { label: t("settings.styleOn"), value: "On {app_name}" },
              { label: t("settings.styleWatching"), value: "Watching {app_name}" },
              { label: t("settings.styleListening"), value: "Listening to {app_name}" },
            ].map((opt) => (
              <button
                key={opt.value}
                onClick={() => setConfig({ ...config, default: { ...config.default, details: opt.value } })}
                style={{
                  padding: "6px 14px", borderRadius: 20, fontSize: 12, fontFamily: "inherit",
                  border: config.default.details === opt.value ? "2px solid var(--accent)" : "1px solid var(--border)",
                  background: config.default.details === opt.value ? "var(--accent-bg)" : "transparent",
                  color: config.default.details === opt.value ? "var(--accent)" : "var(--text-1)",
                  fontWeight: config.default.details === opt.value ? 600 : 400,
                  cursor: "pointer", transition: "all 0.15s",
                }}
              >
                {opt.label}
              </button>
            ))}
          </div>
        </div>

        {/* Secondary text — optional presets */}
        <div>
          <div style={{ fontSize: 12, color: "var(--text-2)", marginBottom: 8, fontWeight: 500 }}>
            {t("settings.presenceStatus")}
          </div>
          <div style={{ display: "flex", flexWrap: "wrap", gap: 6, alignItems: "center" }}>
            {[
              { label: t("settings.statusNone"), value: "" },
              { label: t("settings.statusChilling"), value: "Chilling" },
              { label: t("settings.statusWorking"), value: "Working" },
              { label: t("settings.statusStreaming"), value: "Streaming" },
              { label: t("settings.statusAFK"), value: "AFK" },
            ].map((opt) => {
              const presets = ["", "Chilling", "Working", "Streaming", "AFK"];
              const isSelected = config.default.state === opt.value;
              return (
                <button
                  key={opt.value}
                  onClick={() => setConfig({ ...config, default: { ...config.default, state: opt.value } })}
                  style={{
                    padding: "6px 14px", borderRadius: 20, fontSize: 12, fontFamily: "inherit",
                    border: isSelected ? "2px solid var(--accent)" : "1px solid var(--border)",
                    background: isSelected ? "var(--accent-bg)" : "transparent",
                    color: isSelected ? "var(--accent)" : "var(--text-1)",
                    fontWeight: isSelected ? 600 : 400,
                    cursor: "pointer", transition: "all 0.15s",
                  }}
                >
                  {opt.label}
                </button>
              );
            })}
            <input
              className="input-apple"
              value={!["", "Chilling", "Working", "Streaming", "AFK"].includes(config.default.state) ? config.default.state : ""}
              onChange={(e) => setConfig({ ...config, default: { ...config.default, state: e.target.value } })}
              placeholder={t("settings.statusCustom")}
              style={{ width: 130, padding: "6px 12px", borderRadius: 20, fontSize: 12 }}
            />
          </div>
        </div>
      </div>

      {/* SteamGridDB */}
      <div className="card" style={{ padding: 16 }}>
        <div className="section-label" style={{ marginBottom: 12 }}>{t("settings.steamgriddb")}</div>
        <div>
          <label style={{ display: "block", fontSize: 12, color: "var(--text-2)", marginBottom: 6 }}>{t("settings.apiKey")}</label>
          <input className="input-apple" type="password"
            value={config.steamgriddb_api_key || ""}
            onChange={(e) => setConfig({ ...config, steamgriddb_api_key: e.target.value })}
            placeholder={t("settings.apiKeyPlaceholder")} />
          <p style={{ fontSize: 11, color: "var(--text-3)", marginTop: 6 }}>
            {t("settings.apiKeyDesc")}
          </p>
        </div>
      </div>

      {/* Auto Start */}
      <div className="card" style={{ padding: 16 }}>
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
          <div>
            <div style={{ fontSize: 13, fontWeight: 500, color: "var(--text-1)" }}>{t("settings.startOnBoot")}</div>
            <div style={{ fontSize: 11, color: "var(--text-3)", marginTop: 1 }}>{t("settings.startOnBootDesc")}</div>
          </div>
          <button onClick={handleAutoStart} className={`toggle-apple ${autoStart ? "active" : ""}`}>
            <span className="thumb" />
          </button>
        </div>
      </div>

      {/* Save */}
      <button onClick={handleSave} className="btn-primary"
        style={{
          width: "100%", padding: "10px 0", fontSize: 14, borderRadius: 10,
          background: saved ? "var(--green)" : undefined,
        }}>
        {saved ? `\u2713 ${t("settings.saved")}` : t("settings.saveSettings")}
      </button>
    </div>
  );
}
