import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { LANGUAGES, getFlagUrl } from "../i18n";
import { useTheme } from "../lib/theme";
import { enableAutoStartup, loadConfig, saveConfig, startPoller, openUrl, setTrayLanguage } from "../lib/commands";
import { resizeWindow } from "../App";

interface Props {
  onComplete: (serviceStarted: boolean) => void;
}

export default function SetupWizard({ onComplete }: Props) {
  const { t, i18n } = useTranslation();
  const { theme, toggle: toggleTheme } = useTheme();
  const [step, setStep] = useState(0);
  const [autoStart, setAutoStart] = useState(false);
  const [autoService, setAutoService] = useState(true);
  const [musicEnabled, setMusicEnabled] = useState(true);
  const [steamGridKey, setSteamGridKey] = useState("");
  const [closing, setClosing] = useState(false);

  const totalSteps = 6;

  // Step 0 (languages) needs more height, other steps are smaller
  const stepHeights = [700, 520, 520, 520, 520, 560];

  useEffect(() => {
    // Small delay on first mount to let the window fully initialize
    if (step === 0) {
      const t = setTimeout(() => resizeWindow(stepHeights[0]), 200);
      return () => clearTimeout(t);
    }
    resizeWindow(stepHeights[step]);
  }, [step]);

  // Restore normal height on unmount
  useEffect(() => {
    return () => { resizeWindow(520); };
  }, []);

  const handleFinish = async () => {
    if (autoStart) {
      try { await enableAutoStartup(); } catch (_) {}
    }
    // Save config options set in the wizard
    try {
      const cfg = await loadConfig();
      cfg.auto_start_service = autoService;
      cfg.music_enabled = musicEnabled;
      if (steamGridKey.trim()) cfg.steamgriddb_api_key = steamGridKey.trim();
      await saveConfig(cfg);
    } catch (_) {}
    // Start the service directly if user enabled it
    let serviceStarted = false;
    if (autoService) {
      try {
        await startPoller();
        serviceStarted = true;
      } catch (_) {}
    }
    setClosing(true);
    setTimeout(() => onComplete(serviceStarted), 300);
  };

  const svgProps = { width: 28, height: 28, fill: "none", stroke: "currentColor", strokeWidth: 1.6, strokeLinecap: "round" as const, strokeLinejoin: "round" as const };
  const stepIcons = [
    /* Globe */ <svg {...svgProps} viewBox="0 0 24 24"><circle cx="12" cy="12" r="10"/><path d="M2 12h20M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10A15.3 15.3 0 0 1 12 2z"/></svg>,
    /* Palette */ <svg {...svgProps} viewBox="0 0 24 24"><circle cx="13.5" cy="6.5" r="1.5" fill="currentColor" stroke="none"/><circle cx="17.5" cy="10.5" r="1.5" fill="currentColor" stroke="none"/><circle cx="8.5" cy="7.5" r="1.5" fill="currentColor" stroke="none"/><circle cx="6.5" cy="12" r="1.5" fill="currentColor" stroke="none"/><path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.9 0 1.5-.7 1.5-1.5 0-.4-.1-.7-.4-1-.2-.3-.4-.6-.4-1 0-.8.7-1.5 1.5-1.5H16c3.3 0 6-2.7 6-6 0-5.5-4.5-9-10-9z"/></svg>,
    /* Zap */ <svg {...svgProps} viewBox="0 0 24 24"><path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"/></svg>,
    /* Settings */ <svg {...svgProps} viewBox="0 0 24 24"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>,
    /* Music */ <svg {...svgProps} viewBox="0 0 24 24"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>,
    /* Gamepad */ <svg {...svgProps} viewBox="0 0 24 24"><path d="M6 11h4M8 9v4"/><line x1="15" y1="12" x2="15.01" y2="12"/><line x1="18" y1="10" x2="18.01" y2="10"/><path d="M17.32 5H6.68a4 4 0 0 0-3.978 3.59c-.006.052-.01.101-.017.152C2.604 9.416 2 14.456 2 16a3 3 0 0 0 3 3c1 0 1.5-.5 2-1l1.414-1.414A2 2 0 0 1 9.828 16h4.344a2 2 0 0 1 1.414.586L17 18c.5.5 1 1 2 1a3 3 0 0 0 3-3c0-1.544-.604-6.584-.685-7.258-.007-.05-.011-.1-.017-.151A4 4 0 0 0 17.32 5z"/></svg>,
  ];
  const stepTitles = [
    t("setup.welcome"),
    t("setup.chooseTheme"),
    t("setup.autoStart"),
    t("setup.serviceTitle"),
    t("setup.musicTitle"),
    t("setup.steamgridTitle"),
  ];
  const stepDescs = [
    t("setup.chooseLanguage"),
    t("setup.welcomeDesc"),
    t("setup.autoStartDesc"),
    t("setup.serviceDesc"),
    t("setup.musicDesc"),
    t("setup.steamgridDesc"),
  ];

  return (
    <div
      className="popup-backdrop"
      style={{
        position: "fixed", inset: 0, zIndex: 9999,
        background: "var(--bg-overlay)", backdropFilter: "blur(8px)",
        display: "flex", alignItems: "center", justifyContent: "center",
        opacity: closing ? 0 : 1,
        transition: "opacity 0.3s ease",
      }}
    >
      <div
        className="popup-card"
        style={{
          background: "var(--bg-elevated)", borderRadius: 16,
          border: "1px solid var(--border)", boxShadow: "var(--shadow-lg)",
          width: 460, overflow: "hidden",
          display: "flex", flexDirection: "column",
          transform: closing ? "scale(0.95)" : undefined,
          transition: "transform 0.3s ease",
        }}
      >
        {/* Header */}
        <div style={{ padding: "24px 24px 0", textAlign: "center" }}>
          <div style={{ marginBottom: 8, color: "var(--accent)", display: "flex", justifyContent: "center" }}>
            {stepIcons[step]}
          </div>
          <h2 style={{ fontSize: 20, fontWeight: 700, color: "var(--text-1)", marginBottom: 4 }}>
            {stepTitles[step]}
          </h2>
          <p style={{ fontSize: 13, color: "var(--text-2)", marginBottom: 4 }}>
            {stepDescs[step]}
          </p>
          <div style={{ fontSize: 11, color: "var(--text-3)" }}>
            {t("setup.step", { current: step + 1, total: totalSteps })}
          </div>
        </div>

        {/* Content */}
        <div className="wizard-step-content" key={step} style={{ padding: 20, flex: 1 }}>
          {/* Step 0: Language */}
          {step === 0 && (
            <div style={{
              display: "grid", gridTemplateColumns: "1fr 1fr", gap: 6,
              maxHeight: 420, overflowY: "auto", paddingRight: 4,
            }}>
              {LANGUAGES.map((lang, idx) => (
                <button
                  key={lang.code}
                  className="wizard-lang-item"
                  onClick={async () => {
                    i18n.changeLanguage(lang.code);
                    localStorage.setItem("language", lang.code);
                    setTrayLanguage(lang.code);
                    try {
                      const cfg = await loadConfig();
                      cfg.language = lang.code;
                      await saveConfig(cfg);
                    } catch {}
                  }}
                  style={{
                    display: "flex", alignItems: "center", gap: 8,
                    padding: "8px 12px", borderRadius: 10,
                    border: i18n.language === lang.code
                      ? "2px solid var(--accent)"
                      : "1px solid var(--border)",
                    background: i18n.language === lang.code
                      ? "var(--accent-bg)"
                      : "var(--bg-card)",
                    cursor: "pointer", fontSize: 13, color: "var(--text-1)",
                    fontWeight: i18n.language === lang.code ? 600 : 400,
                    transition: "all 0.15s ease",
                    fontFamily: "inherit",
                    animationDelay: `${idx * 20}ms`,
                  }}
                >
                  <img src={getFlagUrl(lang.country)} alt={lang.country} style={{ width: 24, height: 18, borderRadius: 2, objectFit: "cover" }} />
                  <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                    {lang.name}
                  </span>
                </button>
              ))}
            </div>
          )}

          {/* Step 1: Theme */}
          {step === 1 && (
            <div style={{ display: "flex", gap: 12, justifyContent: "center" }}>
              {(["light", "dark"] as const).map((t_, idx) => (
                <button
                  key={t_}
                  className="wizard-theme-card"
                  onClick={() => { if (theme !== t_) toggleTheme(); }}
                  style={{
                    flex: 1, maxWidth: 160, padding: "20px 16px",
                    borderRadius: 14,
                    border: theme === t_
                      ? "2px solid var(--accent)"
                      : "1px solid var(--border)",
                    background: theme === t_ ? "var(--accent-bg)" : "var(--bg-card)",
                    cursor: "pointer", textAlign: "center",
                    animationDelay: `${idx * 100}ms`,
                  }}
                >
                  <div style={{ marginBottom: 8, display: "flex", justifyContent: "center", color: theme === t_ ? "var(--accent)" : "var(--text-2)" }}>
                    {t_ === "light"
                      ? <svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="5"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/></svg>
                      : <svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/></svg>
                    }
                  </div>
                  <div style={{
                    fontSize: 14, fontWeight: 600,
                    color: theme === t_ ? "var(--accent)" : "var(--text-1)",
                  }}>
                    {t_ === "light" ? t("setup.light") : t("setup.dark")}
                  </div>
                </button>
              ))}
            </div>
          )}

          {/* Step 2: Auto start on boot */}
          {step === 2 && (
            <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 16, padding: "12px 0" }}>
              <div style={{
                width: 72, height: 72, borderRadius: "50%",
                background: autoStart ? "var(--green-bg)" : "var(--bg-input)",
                display: "flex", alignItems: "center", justifyContent: "center",
                transition: "all 0.3s ease",
              }}>
                {autoStart
                  ? <svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="var(--green)" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><path d="M22 4L12 14.01l-3-3"/></svg>
                  : <svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="var(--text-3)" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round"><path d="M18.36 5.64a9 9 0 1 1-12.73 0"/><line x1="12" y1="2" x2="12" y2="12"/></svg>
                }
              </div>
              <button
                onClick={() => setAutoStart(!autoStart)}
                className={`toggle-apple ${autoStart ? "active" : ""}`}
                style={{ transform: "scale(1.3)" }}
              >
                <span className="thumb" />
              </button>
              <p style={{ fontSize: 13, color: "var(--text-2)", textAlign: "center" }}>
                {autoStart ? t("setup.autoStartEnabled") : t("setup.autoStartDisabled")}
              </p>
            </div>
          )}

          {/* Step 3: Auto start service */}
          {step === 3 && (
            <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 16, padding: "12px 0" }}>
              <div style={{
                width: 72, height: 72, borderRadius: "50%",
                background: autoService ? "var(--green-bg)" : "var(--bg-input)",
                display: "flex", alignItems: "center", justifyContent: "center",
                transition: "all 0.3s ease",
              }}>
                {autoService
                  ? <svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="var(--green)" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10"/><path d="M8 14s1.5 2 4 2 4-2 4-2"/><line x1="9" y1="9" x2="9.01" y2="9"/><line x1="15" y1="9" x2="15.01" y2="9"/></svg>
                  : <svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="var(--text-3)" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round"><rect x="6" y="4" width="4" height="16"/><rect x="14" y="4" width="4" height="16"/></svg>
                }
              </div>
              <button
                onClick={() => setAutoService(!autoService)}
                className={`toggle-apple ${autoService ? "active" : ""}`}
                style={{ transform: "scale(1.3)" }}
              >
                <span className="thumb" />
              </button>
              <p style={{ fontSize: 13, color: "var(--text-2)", textAlign: "center" }}>
                {autoService ? t("setup.serviceEnabled") : t("setup.serviceDisabled")}
              </p>
            </div>
          )}

          {/* Step 4: Music detection */}
          {step === 4 && (
            <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 16, padding: "12px 0" }}>
              <div style={{
                width: 72, height: 72, borderRadius: "50%",
                background: musicEnabled ? "var(--accent-bg)" : "var(--bg-input)",
                display: "flex", alignItems: "center", justifyContent: "center",
                transition: "all 0.3s ease",
              }}>
                {musicEnabled
                  ? <svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="var(--accent)" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>
                  : <svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="var(--text-3)" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round"><path d="M11 5L6 9H2v6h4l5 4V5z"/><line x1="23" y1="9" x2="17" y2="15"/><line x1="17" y1="9" x2="23" y2="15"/></svg>
                }
              </div>
              <button
                onClick={() => setMusicEnabled(!musicEnabled)}
                className={`toggle-apple ${musicEnabled ? "active" : ""}`}
                style={{ transform: "scale(1.3)" }}
              >
                <span className="thumb" />
              </button>
              <p style={{ fontSize: 13, color: "var(--text-2)", textAlign: "center" }}>
                {musicEnabled ? t("setup.musicEnabled") : t("setup.musicDisabled")}
              </p>
            </div>
          )}

          {/* Step 5: SteamGridDB API key (optional) */}
          {step === 5 && (
            <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 16, padding: "12px 0" }}>
              <div style={{
                width: 72, height: 72, borderRadius: "50%",
                background: steamGridKey.trim() ? "var(--green-bg)" : "var(--bg-input)",
                display: "flex", alignItems: "center", justifyContent: "center",
                transition: "all 0.3s ease",
              }}>
                {steamGridKey.trim()
                  ? <svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="var(--green)" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round"><path d="M6 11h4M8 9v4"/><line x1="15" y1="12" x2="15.01" y2="12"/><line x1="18" y1="10" x2="18.01" y2="10"/><path d="M17.32 5H6.68a4 4 0 0 0-3.978 3.59c-.006.052-.01.101-.017.152C2.604 9.416 2 14.456 2 16a3 3 0 0 0 3 3c1 0 1.5-.5 2-1l1.414-1.414A2 2 0 0 1 9.828 16h4.344a2 2 0 0 1 1.414.586L17 18c.5.5 1 1 2 1a3 3 0 0 0 3-3c0-1.544-.604-6.584-.685-7.258-.007-.05-.011-.1-.017-.151A4 4 0 0 0 17.32 5z"/></svg>
                  : <svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="var(--text-3)" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round"><path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4"/></svg>
                }
              </div>
              <div style={{ position: "relative", width: "100%", maxWidth: 320 }}>
                <input
                  type="text"
                  value={steamGridKey}
                  onChange={(e) => setSteamGridKey(e.target.value)}
                  placeholder={t("setup.steamgridPlaceholder")}
                  style={{
                    width: "100%", padding: "10px 14px",
                    paddingRight: steamGridKey.trim() ? 38 : 14,
                    borderRadius: 10,
                    border: steamGridKey.trim()
                      ? "2px solid var(--accent)"
                      : "1px solid var(--border)",
                    background: "var(--bg-input)", color: "var(--text-1)",
                    fontSize: 13, fontFamily: "inherit", outline: "none",
                    textAlign: "center",
                    transition: "border 0.2s ease",
                    boxSizing: "border-box",
                  }}
                />
                {steamGridKey.trim() && (
                  <div style={{
                    position: "absolute", right: 10, top: "50%", transform: "translateY(-50%)",
                    color: "var(--accent)", fontSize: 18, lineHeight: 1,
                  }}>
                    {"\u2713"}
                  </div>
                )}
              </div>
              {steamGridKey.trim() && (
                <p style={{ fontSize: 12, color: "var(--accent)", textAlign: "center", fontWeight: 600 }}>
                  {t("setup.steamgridValid")}
                </p>
              )}
              <p style={{ fontSize: 12, color: "var(--text-2)", textAlign: "center", lineHeight: 1.5 }}>
                {t("setup.steamgridOptional").split("steamgriddb.com").map((part, i, arr) =>
                  i < arr.length - 1 ? (
                    <span key={i}>{part}<a href="#" onClick={(e) => { e.preventDefault(); openUrl("https://www.steamgriddb.com"); }} style={{ color: "var(--accent)", textDecoration: "underline", cursor: "pointer" }}>steamgriddb.com</a></span>
                  ) : part
                )}
              </p>
            </div>
          )}
        </div>

        {/* Footer */}
        <div style={{
          padding: "12px 20px 20px",
          display: "flex", justifyContent: "space-between", alignItems: "center",
        }}>
          {step > 0 ? (
            <button
              onClick={() => setStep(step - 1)}
              className="btn-secondary"
              style={{ padding: "8px 20px" }}
            >
              {t("setup.back")}
            </button>
          ) : <div />}

          <button
            onClick={() => {
              if (step < totalSteps - 1) setStep(step + 1);
              else handleFinish();
            }}
            className="btn-primary"
            style={{ padding: "8px 28px", fontSize: 14 }}
          >
            {step < totalSteps - 1 ? t("setup.next") : t("setup.getStarted")}
          </button>
        </div>

        {/* Step dots */}
        <div style={{ display: "flex", justifyContent: "center", gap: 6, paddingBottom: 16 }}>
          {Array.from({ length: totalSteps }).map((_, i) => (
            <div
              key={i}
              style={{
                width: i === step ? 20 : 6, height: 6,
                borderRadius: 3,
                background: i === step ? "var(--accent)" : "var(--bg-input)",
                transition: "all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1)",
              }}
            />
          ))}
        </div>
      </div>
    </div>
  );
}
