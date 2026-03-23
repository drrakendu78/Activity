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

  const stepIcons = ["\uD83C\uDF10", "\uD83C\uDFA8", "\u26A1", "\uD83D\uDD27", "\uD83C\uDFB5", "\uD83C\uDFAE"];
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
          <div style={{ fontSize: 32, marginBottom: 8 }}>
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
                  <div style={{ fontSize: 36, marginBottom: 8 }}>
                    {t_ === "light" ? "\u2600\uFE0F" : "\uD83C\uDF19"}
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
                <span style={{ fontSize: 36 }}>{autoStart ? "\u2705" : "\uD83D\uDCA4"}</span>
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
                <span style={{ fontSize: 36 }}>{autoService ? "\uD83D\uDE80" : "\u23F8\uFE0F"}</span>
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
                <span style={{ fontSize: 36 }}>{musicEnabled ? "\uD83C\uDFB6" : "\uD83D\uDD07"}</span>
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
                <span style={{ fontSize: 36, width: 72, height: 72, lineHeight: "72px", textAlign: "center", display: "block" }}>{steamGridKey.trim() ? "\uD83C\uDFAE" : "\uD83D\uDD11"}</span>
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
