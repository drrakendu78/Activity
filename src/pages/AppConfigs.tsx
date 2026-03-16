import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  getBuiltinAppsList, getDetectedApps, loadConfig, saveConfig,
  type AppConfig, type DetectedApp, type RpcConfig,
} from "../lib/commands";
import AppConfigForm from "../components/AppConfigForm";
import IconPicker from "../components/IconPicker";

type ViewMode = "detected" | "database" | "custom";

export default function AppConfigs() {
  const { t } = useTranslation();
  const [config, setConfig] = useState<RpcConfig | null>(null);
  const [builtinApps, setBuiltinApps] = useState<Record<string, AppConfig>>({});
  const [detectedApps, setDetectedApps] = useState<DetectedApp[]>([]);
  const [viewMode, setViewMode] = useState<ViewMode>("detected");
  const [search, setSearch] = useState("");
  const [editing, setEditing] = useState<{ exe: string; config: AppConfig } | null>(null);
  const [iconPicker, setIconPicker] = useState<{ exe: string; name: string } | null>(null);

  const CATEGORY_LABELS: Record<string, { labelKey: string; emoji: string }> = {
    gaming: { labelKey: "categories.gaming", emoji: "\uD83C\uDFAE" },
    dev: { labelKey: "categories.dev", emoji: "\uD83D\uDCBB" },
    browser: { labelKey: "categories.browser", emoji: "\uD83C\uDF10" },
    social: { labelKey: "categories.social", emoji: "\uD83D\uDCAC" },
    media: { labelKey: "categories.media", emoji: "\uD83C\uDFB5" },
    creative: { labelKey: "categories.creative", emoji: "\uD83C\uDFA8" },
    productivity: { labelKey: "categories.productivity", emoji: "\uD83D\uDCC4" },
    system: { labelKey: "categories.system", emoji: "\u2699\uFE0F" },
    discovered: { labelKey: "categories.discovered", emoji: "\uD83D\uDD0D" },
    other: { labelKey: "categories.other", emoji: "\uD83D\uDCE6" },
  };

  useEffect(() => {
    loadConfig().then(setConfig);
    getBuiltinAppsList().then(setBuiltinApps).catch(() => {});
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

  const allApps: Record<string, AppConfig> = { ...builtinApps, ...config.apps };

  const allByCategory: Record<string, { exe: string; config: AppConfig; isCustom: boolean }[]> = {};
  Object.entries(allApps)
    .filter(([exe, app]) => {
      if (!search) return true;
      const q = search.toLowerCase();
      return exe.includes(q) || app.name.toLowerCase().includes(q) || app.category.toLowerCase().includes(q);
    })
    .forEach(([exe, app]) => {
      const cat = app.category || "other";
      if (!allByCategory[cat]) allByCategory[cat] = [];
      const isCustom = config.apps[exe] !== undefined;
      allByCategory[cat].push({ exe, config: app, isCustom });
    });

  const customEntries = Object.entries(config.apps);
  const views: { id: ViewMode; labelKey: string; count: number }[] = [
    { id: "detected", labelKey: "apps.detected", count: detectedApps.length },
    { id: "database", labelKey: "apps.database", count: Object.keys(allApps).length },
    { id: "custom", labelKey: "apps.custom", count: customEntries.length },
  ];

  return (
    <div style={{ padding: 24, display: "flex", flexDirection: "column", gap: 16, height: "100%" }}>
      <div>
        <h1 style={{ fontSize: 22, fontWeight: 700, letterSpacing: "-0.02em", color: "var(--text-1)" }}>{t("apps.title")}</h1>
        <p style={{ fontSize: 13, color: "var(--text-2)", marginTop: 2 }}>{t("apps.subtitle")}</p>
      </div>

      {/* Underline Tabs */}
      <div className="underline-tabs" style={{ marginTop: -4 }}>
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
        <div style={{ flex: 1, overflowY: "auto", display: "flex", flexDirection: "column", gap: 6 }}>
          {detectedApps.length === 0 ? (
            <div className="card" style={{ padding: "40px 20px", textAlign: "center" }}>
              <div style={{ fontSize: 28, marginBottom: 8 }}>{"\uD83D\uDC40"}</div>
              <div style={{ fontSize: 14, fontWeight: 500, color: "var(--text-1)" }}>{t("apps.noAppsDetected")}</div>
              <div style={{ fontSize: 12, color: "var(--text-3)", marginTop: 4, maxWidth: 240, margin: "4px auto 0" }}>
                {t("apps.noAppsDetectedDesc")}
              </div>
            </div>
          ) : (
            detectedApps.map((app) => {
              const cat = CATEGORY_LABELS[app.category] || CATEGORY_LABELS.other;
              const isCustom = config.apps[app.exe_name] !== undefined;
              return (
                <div key={app.exe_name} className="card-interactive"
                  style={{ padding: "10px 14px", display: "flex", alignItems: "center", gap: 10 }}>
                  <span style={{ fontSize: 18, flexShrink: 0 }}>{cat.emoji}</span>
                  <div style={{ minWidth: 0, flex: 1 }}>
                    <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
                      <span style={{ fontSize: 13, fontWeight: 500, color: "var(--text-1)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{app.name}</span>
                      {app.is_builtin && <span className="badge badge-blue">auto</span>}
                      {isCustom && <span className="badge badge-orange">custom</span>}
                    </div>
                    <div style={{ fontSize: 11, color: "var(--text-3)", marginTop: 2 }}>
                      {app.exe_name} · {app.details}
                    </div>
                  </div>
                  <div style={{ display: "flex", gap: 4, flexShrink: 0 }}>
                    <button onClick={() => setIconPicker({ exe: app.exe_name, name: app.name })}
                      className="btn-secondary" style={{ padding: "4px 8px", fontSize: 11 }}
                      title={t("apps.changeIcon")}>
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" /><circle cx="12" cy="12" r="3" /><path d="M12 3v6m0 6v6M3 12h6m6 0h6" /></svg>
                    </button>
                    <button onClick={() => {
                      const existing = config.apps[app.exe_name];
                      setEditing({
                        exe: app.exe_name,
                        config: existing || {
                          name: app.name, details: app.details, state: "",
                          large_image_key: null, large_image_text: null,
                          small_image_key: null, small_image_text: null,
                          category: app.category, show_timestamp: true, buttons: [],
                        },
                      });
                    }} className="btn-secondary" style={{ padding: "4px 10px", fontSize: 11 }}>
                      {t("apps.edit")}
                    </button>
                  </div>
                </div>
              );
            })
          )}
        </div>
      )}

      {/* Database */}
      {viewMode === "database" && (
        <div style={{ flex: 1, overflowY: "auto", display: "flex", flexDirection: "column", gap: 12 }}>
          <input type="text" placeholder={t("apps.searchApps")} value={search}
            onChange={(e) => setSearch(e.target.value)} className="input-apple" />
          {Object.entries(CATEGORY_LABELS).map(([catKey, cat]) => {
            const apps = allByCategory[catKey];
            if (!apps || apps.length === 0) return null;
            return (
              <div key={catKey}>
                <div className="section-label" style={{ marginBottom: 6, display: "flex", alignItems: "center", gap: 5 }}>
                  <span>{cat.emoji}</span> {t(cat.labelKey)}
                  <span style={{ fontWeight: 400, opacity: 0.5 }}>({apps.length})</span>
                </div>
                <div style={{ display: "flex", flexDirection: "column", gap: 2 }}>
                  {apps.map(({ exe, config: appConfig, isCustom }) => (
                    <div key={exe} className="card-interactive"
                      style={{ padding: "8px 14px", display: "flex", alignItems: "center", justifyContent: "space-between" }}>
                      <div style={{ minWidth: 0, display: "flex", alignItems: "center", gap: 6 }}>
                        <span style={{ fontSize: 13, fontWeight: 500, color: "var(--text-1)" }}>{appConfig.name}</span>
                        <span style={{ fontSize: 11, color: "var(--text-3)" }}>{exe}</span>
                        {isCustom && <span className="badge badge-orange">custom</span>}
                      </div>
                      <span style={{ fontSize: 11, color: "var(--text-3)", flexShrink: 0 }}>{appConfig.details}</span>
                    </div>
                  ))}
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* Custom */}
      {viewMode === "custom" && (
        <div style={{ flex: 1, overflowY: "auto", display: "flex", flexDirection: "column", gap: 6 }}>
          {customEntries.length === 0 ? (
            <div className="card" style={{ padding: "40px 20px", textAlign: "center" }}>
              <div style={{ fontSize: 28, marginBottom: 8 }}>{"\u2728"}</div>
              <div style={{ fontSize: 14, fontWeight: 500, color: "var(--text-1)" }}>{t("apps.noCustomOverrides")}</div>
              <div style={{ fontSize: 12, color: "var(--text-3)", marginTop: 4 }}>
                {t("apps.noCustomOverridesDesc")}
              </div>
            </div>
          ) : (
            customEntries.map(([exe, appConfig]) => (
              <div key={exe} className="card-interactive group"
                style={{ padding: "10px 14px", display: "flex", alignItems: "center", justifyContent: "space-between" }}>
                <div style={{ minWidth: 0, flex: 1 }}>
                  <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
                    <span style={{ fontSize: 13, fontWeight: 500, color: "var(--text-1)" }}>{appConfig.name}</span>
                    <span style={{ fontSize: 11, color: "var(--text-3)", fontFamily: "monospace" }}>{exe}</span>
                  </div>
                  <div style={{ fontSize: 11, color: "var(--text-3)", marginTop: 2 }}>
                    {appConfig.details || t("apps.noDetails")} {appConfig.state ? `\u00B7 ${appConfig.state}` : ""}
                  </div>
                </div>
                <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                  <button onClick={() => setEditing({ exe, config: appConfig })}
                    className="btn-secondary" style={{ padding: "4px 10px", fontSize: 11 }}>{t("apps.edit")}</button>
                  <button onClick={() => handleDeleteCustom(exe)}
                    className="btn-danger" style={{ padding: "4px 10px", fontSize: 11 }}>{t("apps.delete")}</button>
                </div>
              </div>
            ))
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
