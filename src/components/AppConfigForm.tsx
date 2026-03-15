import { useState } from "react";
import { useTranslation } from "react-i18next";
import type { AppConfig, ButtonConfig } from "../lib/commands";

interface Props {
  exeName?: string;
  config?: AppConfig;
  onSave: (exeName: string, config: AppConfig) => void;
  onCancel: () => void;
}

export default function AppConfigForm({ exeName: initialExe, config, onSave, onCancel }: Props) {
  const { t } = useTranslation();
  const [exeName, setExeName] = useState(initialExe || "");
  const [name, setName] = useState(config?.name || "");
  const [details, setDetails] = useState(config?.details || "");
  const [state, setState] = useState(config?.state || "");
  const [largeImageKey, setLargeImageKey] = useState(config?.large_image_key || "");
  const [largeImageText, setLargeImageText] = useState(config?.large_image_text || "");
  const [smallImageKey, setSmallImageKey] = useState(config?.small_image_key || "");
  const [smallImageText, setSmallImageText] = useState(config?.small_image_text || "");
  const [showTimestamp, setShowTimestamp] = useState(config?.show_timestamp ?? true);
  const [buttons, setButtons] = useState<ButtonConfig[]>(config?.buttons || []);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!exeName.trim() || !name.trim()) return;
    onSave(exeName.toLowerCase().trim(), {
      name: name.trim(), details: details.trim(), state: state.trim(),
      large_image_key: largeImageKey.trim() || null, large_image_text: largeImageText.trim() || null,
      small_image_key: smallImageKey.trim() || null, small_image_text: smallImageText.trim() || null,
      category: config?.category || "other", show_timestamp: showTimestamp,
      buttons: buttons.filter((b) => b.label.trim() && b.url.trim()),
    });
  };

  const addButton = () => { if (buttons.length < 2) setButtons([...buttons, { label: "", url: "" }]); };
  const removeButton = (i: number) => setButtons(buttons.filter((_, idx) => idx !== i));
  const updateButton = (i: number, field: keyof ButtonConfig, value: string) => {
    const u = [...buttons]; u[i] = { ...u[i], [field]: value }; setButtons(u);
  };

  const labelStyle = { display: "block" as const, fontSize: 12, color: "var(--text-2)", marginBottom: 6 };

  return (
    <div style={{
      position: "fixed", inset: 0, background: "var(--bg-overlay)",
      display: "flex", alignItems: "center", justifyContent: "center", zIndex: 50,
      backdropFilter: "blur(4px)",
    }}>
      <form onSubmit={handleSubmit} className="modal-enter" style={{
        background: "var(--bg-elevated)", borderRadius: 14, padding: 24,
        width: 440, maxHeight: "85vh", overflowY: "auto",
        boxShadow: "var(--shadow-lg)", border: "1px solid var(--border)",
      }}>
        <h3 style={{ fontSize: 17, fontWeight: 600, color: "var(--text-1)", marginBottom: 20 }}>
          {initialExe ? t("appConfigForm.editTitle") : t("appConfigForm.addTitle")}
        </h3>

        <div style={{ display: "flex", flexDirection: "column", gap: 14 }}>
          <div>
            <label style={labelStyle}>{t("appConfigForm.executableName")}</label>
            <input className="input-apple" value={exeName} onChange={(e) => setExeName(e.target.value)}
              placeholder="app.exe" disabled={!!initialExe} />
          </div>
          <div>
            <label style={labelStyle}>{t("appConfigForm.displayName")}</label>
            <input className="input-apple" value={name} onChange={(e) => setName(e.target.value)} placeholder="My App" />
          </div>
          <div>
            <label style={labelStyle}>{t("appConfigForm.details")}</label>
            <input className="input-apple" value={details} onChange={(e) => setDetails(e.target.value)} placeholder={t("appConfigForm.detailsPlaceholder")} />
          </div>
          <div>
            <label style={labelStyle}>{t("appConfigForm.state")}</label>
            <input className="input-apple" value={state} onChange={(e) => setState(e.target.value)} placeholder={t("appConfigForm.statePlaceholder")} />
          </div>

          {/* Images */}
          <div style={{ borderTop: "1px solid var(--border-light)", paddingTop: 14 }}>
            <div className="section-label" style={{ marginBottom: 10 }}>{t("appConfigForm.images")}</div>
            <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
              <div>
                <label style={labelStyle}>{t("appConfigForm.largeImage")}</label>
                <input className="input-apple" value={largeImageKey}
                  onChange={(e) => setLargeImageKey(e.target.value)} placeholder={t("appConfigForm.largeImagePlaceholder")} />
                {largeImageKey && largeImageKey.startsWith("http") && (
                  <div style={{ display: "flex", alignItems: "center", gap: 8, marginTop: 8 }}>
                    <img src={largeImageKey} alt="preview"
                      style={{ width: 36, height: 36, borderRadius: 8, background: "var(--bg-input)", objectFit: "contain", padding: 3 }}
                      onError={(e) => { (e.target as HTMLImageElement).style.display = "none"; }} />
                    <span style={{ fontSize: 10, color: "var(--text-3)" }}>{t("appConfigForm.preview")}</span>
                  </div>
                )}
              </div>
              <div>
                <label style={labelStyle}>{t("appConfigForm.largeImageTooltip")}</label>
                <input className="input-apple" value={largeImageText}
                  onChange={(e) => setLargeImageText(e.target.value)} placeholder={t("appConfigForm.hoverText")} />
              </div>
              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 10 }}>
                <div>
                  <label style={labelStyle}>{t("appConfigForm.smallImage")}</label>
                  <input className="input-apple" value={smallImageKey} onChange={(e) => setSmallImageKey(e.target.value)} />
                </div>
                <div>
                  <label style={labelStyle}>{t("appConfigForm.smallTooltip")}</label>
                  <input className="input-apple" value={smallImageText} onChange={(e) => setSmallImageText(e.target.value)} />
                </div>
              </div>
            </div>
          </div>

          {/* Timestamp */}
          <div style={{ borderTop: "1px solid var(--border-light)", paddingTop: 14 }}>
            <label style={{ display: "flex", alignItems: "center", gap: 8, cursor: "pointer" }}>
              <input type="checkbox" checked={showTimestamp} onChange={(e) => setShowTimestamp(e.target.checked)}
                style={{ width: 16, height: 16, accentColor: "var(--accent)" }} />
              <span style={{ fontSize: 13, color: "var(--text-1)" }}>{t("appConfigForm.showElapsedTime")}</span>
            </label>
          </div>

          {/* Buttons */}
          <div style={{ borderTop: "1px solid var(--border-light)", paddingTop: 14 }}>
            <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 10 }}>
              <span className="section-label">{t("appConfigForm.buttons")} ({buttons.length}/2)</span>
              {buttons.length < 2 && (
                <button type="button" onClick={addButton} style={{
                  fontSize: 11, padding: "3px 10px", borderRadius: 6, border: "none", cursor: "pointer",
                  background: "var(--accent-bg)", color: "var(--accent)", fontWeight: 600, fontFamily: "inherit",
                }}>+ {t("appConfigForm.add")}</button>
              )}
            </div>
            {buttons.map((btn, i) => (
              <div key={i} style={{ display: "flex", gap: 6, marginBottom: 8 }}>
                <input className="input-apple" style={{ flex: 1 }} value={btn.label}
                  onChange={(e) => updateButton(i, "label", e.target.value)} placeholder={t("appConfigForm.buttonText")} />
                <input className="input-apple" style={{ flex: 2 }} value={btn.url}
                  onChange={(e) => updateButton(i, "url", e.target.value)} placeholder="https://..." />
                <button type="button" onClick={() => removeButton(i)} style={{
                  background: "none", border: "none", color: "var(--red)", fontSize: 16, cursor: "pointer", padding: "0 4px",
                }}>{"\u00D7"}</button>
              </div>
            ))}
            {buttons.length === 0 && (
              <div style={{ fontSize: 11, color: "var(--text-3)" }}>{t("appConfigForm.buttonLinkDesc")}</div>
            )}
          </div>
        </div>

        <div style={{ display: "flex", justifyContent: "flex-end", gap: 8, marginTop: 20 }}>
          <button type="button" onClick={onCancel} className="btn-secondary">{t("appConfigForm.cancel")}</button>
          <button type="submit" className="btn-primary">{t("appConfigForm.save")}</button>
        </div>
      </form>
    </div>
  );
}
