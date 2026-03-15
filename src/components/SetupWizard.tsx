import { useState } from "react";
import { useTranslation } from "react-i18next";
import { LANGUAGES, getFlagUrl } from "../i18n";
import { useTheme } from "../lib/theme";
import { enableAutoStartup } from "../lib/commands";

interface Props {
  onComplete: () => void;
}

export default function SetupWizard({ onComplete }: Props) {
  const { t, i18n } = useTranslation();
  const { theme, toggle: toggleTheme } = useTheme();
  const [step, setStep] = useState(0);
  const [autoStart, setAutoStart] = useState(false);
  const [closing, setClosing] = useState(false);

  const totalSteps = 3;

  const handleFinish = async () => {
    if (autoStart) {
      try { await enableAutoStartup(); } catch (_) {}
    }
    setClosing(true);
    setTimeout(() => onComplete(), 300);
  };

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
          width: 420, maxHeight: "80vh", overflow: "hidden",
          display: "flex", flexDirection: "column",
          transform: closing ? "scale(0.95)" : undefined,
          transition: "transform 0.3s ease",
        }}
      >
        {/* Header */}
        <div style={{ padding: "24px 24px 0", textAlign: "center" }}>
          <div style={{ fontSize: 32, marginBottom: 8 }}>
            {step === 0 ? "\uD83C\uDF10" : step === 1 ? "\uD83C\uDFA8" : "\u26A1"}
          </div>
          <h2 style={{ fontSize: 20, fontWeight: 700, color: "var(--text-1)", marginBottom: 4 }}>
            {step === 0 ? t("setup.welcome") : step === 1 ? t("setup.chooseTheme") : t("setup.autoStart")}
          </h2>
          <p style={{ fontSize: 13, color: "var(--text-2)", marginBottom: 4 }}>
            {step === 0 ? t("setup.chooseLanguage") : step === 1 ? t("setup.welcomeDesc") : t("setup.autoStartDesc")}
          </p>
          <div style={{ fontSize: 11, color: "var(--text-3)" }}>
            {t("setup.step", { current: step + 1, total: totalSteps })}
          </div>
        </div>

        {/* Content */}
        <div className="wizard-step-content" key={step} style={{ padding: 20, flex: 1, overflowY: "auto" }}>
          {/* Step 0: Language */}
          {step === 0 && (
            <div style={{
              display: "grid", gridTemplateColumns: "1fr 1fr", gap: 6,
              maxHeight: 300, overflowY: "auto", overflowX: "hidden",
            }}>
              {LANGUAGES.map((lang, idx) => (
                <button
                  key={lang.code}
                  className="wizard-lang-item"
                  onClick={() => {
                    i18n.changeLanguage(lang.code);
                    localStorage.setItem("language", lang.code);
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

          {/* Step 2: Auto start */}
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
                {autoStart ? t("setup.autoStart") : t("setup.autoStartDesc")}
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
