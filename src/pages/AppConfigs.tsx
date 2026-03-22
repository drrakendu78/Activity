import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  getDetectedApps, loadConfig, saveConfig,
  type AppConfig, type DetectedApp, type RpcConfig,
} from "../lib/commands";
import AppConfigForm from "../components/AppConfigForm";
import IconPicker from "../components/IconPicker";

type ViewMode = "detected" | "custom";

export default function AppConfigs() {
  const { t } = useTranslation();
  const [config, setConfig] = useState<RpcConfig | null>(null);
  const [detectedApps, setDetectedApps] = useState<DetectedApp[]>([]);
  const [viewMode, setViewMode] = useState<ViewMode>("detected");
  const [editing, setEditing] = useState<{ exe: string; config: AppConfig } | null>(null);
  const [iconPicker, setIconPicker] = useState<{ exe: string; name: string } | null>(null);

  useEffect(() => {
    loadConfig().then(setConfig);
    getDetectedApps().then(setDetectedApps).catch(() => {});
    const interval = setInterval(() => {
      getDetectedApps().then(setDetectedApps).catch(() => {});
    }, 5000);
    return () => clearInterval(interval);
  }, []);

  const handleSaveCustom = async (exeName: string, appConfig: AppConfig) => {
    if (!config) return;
    const updated = { ...config, apps: { ...config.apps, [exeName]: appConfig } };
    await saveConfig(updated);
    setConfig(updated);
    setEditing(null);
  };

  const handleDeleteCustom = async (exeName: string) => {
    if (!config) return;
    const apps = { ...config.apps };
    delete apps[exeName];
    const updated = { ...config, apps };
    await saveConfig(updated);
    setConfig(updated);
  };

  if (!config) return <div style={{ padding: 24, color: "var(--text-3)" }}>{t("apps.loading")}</div>;

  const customEntries = Object.entries(config.apps);

  const views: { id: ViewMode; labelKey: string; count: number }[] = [
    { id: "detected", labelKey: "apps.detected", count: detectedApps.length },
    { id: "custom", labelKey: "apps.custom", count: customEntries.length },
  ];

  return (
    <div style={{ padding: 20, display: "flex", flexDirection: "column", gap: 14, height: "100%" }}>
      {/* Tabs */}
      <div className="underline-tabs">
        {views.map((v) => (
          <button key={v.id} onClick={() => setViewMode(v.id)}
            className={`underline-tab ${viewMode === v.id ? "active" : ""}`}>
            {t(v.labelKey)}
            <span className="underline-tab-count">{v.count}</span>
          </button>
        ))}
      </div>

      {/* Detected */}
      {viewMode === "detected" && (
        <div style={{ flex: 1, overflowY: "auto" }}>
          {detectedApps.length === 0 ? (
            <div className="dashboard-empty" style={{ padding: "48px 0" }}>
              <div className="dashboard-empty-text">{t("apps.noAppsDetected")}</div>
              <div className="dashboard-empty-hint">{t("apps.noAppsDetectedDesc")}</div>
            </div>
          ) : (
            <div className="apple-list">
              {detectedApps.map((app, i) => {
                const customApp = config.apps[app.exe_name];
                const iconUrl = customApp?.large_image_key || app.large_image_key;
                const hasIcon = iconUrl && iconUrl.startsWith("http");
                return (
                  <div key={app.exe_name}
                    className={`apple-list-item ${i === 0 ? "first" : ""} ${i === detectedApps.length - 1 ? "last" : ""}`}
                    onClick={() => {
                      setEditing({
                        exe: app.exe_name,
                        config: customApp || {
                          name: app.name, details: app.details, state: "",
                          large_image_key: null, large_image_text: null,
                          small_image_key: null, small_image_text: null,
                          category: app.category, show_timestamp: true, buttons: [],
                        },
                      });
                    }}>
                    <div className="apple-list-content">
                      <div className="apple-list-icon">
                        {hasIcon ? (
                          <img src={iconUrl!} alt="" style={{ width: 24, height: 24, objectFit: "contain" }} />
                        ) : (
                          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" style={{ color: "var(--text-3)" }}>
                            <rect x="3" y="3" width="18" height="18" rx="4" />
                          </svg>
                        )}
                      </div>
                      <span className="apple-list-name">{app.name}</span>
                      {customApp && <span className="badge badge-orange" style={{ fontSize: 9, padding: "1px 5px" }}>custom</span>}
                    </div>
                    <div className="apple-list-right">
                      <span className="apple-list-detail">{app.details}</span>
                      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" style={{ color: "var(--text-3)", opacity: 0.4 }}>
                        <polyline points="9 18 15 12 9 6" />
                      </svg>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>
      )}

      {/* Custom */}
      {viewMode === "custom" && (
        <div style={{ flex: 1, overflowY: "auto" }}>
          {customEntries.length === 0 ? (
            <div className="dashboard-empty" style={{ padding: "48px 0" }}>
              <div className="dashboard-empty-text">{t("apps.noCustomOverrides")}</div>
              <div className="dashboard-empty-hint">{t("apps.noCustomOverridesDesc")}</div>
            </div>
          ) : (
            <div className="apple-list">
              {customEntries.map(([exe, appConfig], i) => {
                const iconUrl = appConfig.large_image_key;
                const hasIcon = iconUrl && iconUrl.startsWith("http");
                return (
                <div key={exe}
                  className={`apple-list-item ${i === 0 ? "first" : ""} ${i === customEntries.length - 1 ? "last" : ""}`}
                  onClick={() => setEditing({ exe, config: appConfig })}>
                  <div className="apple-list-content">
                    <div className="apple-list-icon">
                      {hasIcon ? (
                        <img src={iconUrl!} alt="" style={{ width: 24, height: 24, objectFit: "contain" }} />
                      ) : (
                        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" style={{ color: "var(--text-3)" }}>
                          <rect x="3" y="3" width="18" height="18" rx="4" />
                        </svg>
                      )}
                    </div>
                    <span className="apple-list-name">{appConfig.name}</span>
                  </div>
                  <div className="apple-list-right">
                    <span className="apple-list-detail">{appConfig.details || ""}</span>
                    <button
                      onClick={(e) => { e.stopPropagation(); handleDeleteCustom(exe); }}
                      className="apple-list-delete">
                      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                        <line x1="18" y1="6" x2="6" y2="18" />
                        <line x1="6" y1="6" x2="18" y2="18" />
                      </svg>
                    </button>
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" style={{ color: "var(--text-3)", opacity: 0.4 }}>
                      <polyline points="9 18 15 12 9 6" />
                    </svg>
                  </div>
                </div>
                );
              })}
            </div>
          )}
        </div>
      )}

      {editing && (
        <AppConfigForm exeName={editing.exe} config={editing.config}
          onSave={handleSaveCustom} onCancel={() => setEditing(null)} />
      )}

      {iconPicker && (
        <IconPicker
          exeName={iconPicker.exe}
          appName={iconPicker.name}
          onClose={() => setIconPicker(null)}
          onPicked={() => {
            loadConfig().then(setConfig);
            setIconPicker(null);
          }}
        />
      )}
    </div>
  );
}
