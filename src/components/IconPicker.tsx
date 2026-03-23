import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { searchIcons, setAppIcon, type IconCandidate } from "../lib/commands";

interface IconPickerProps {
  exeName: string;
  appName: string;
  onClose: () => void;
  onPicked: (url: string) => void;
}

export default function IconPicker({ exeName, appName, onClose, onPicked }: IconPickerProps) {
  const { t } = useTranslation();
  const [candidates, setCandidates] = useState<IconCandidate[]>([]);
  const [loading, setLoading] = useState(true);
  const [customQuery, setCustomQuery] = useState(appName);
  const [saving, setSaving] = useState<string | null>(null);
  const [picked, setPicked] = useState<string | null>(null);
  const [closing, setClosing] = useState(false);

  const animateClose = () => {
    setClosing(true);
    setTimeout(() => onClose(), 200);
  };

  const doSearch = async (query: string) => {
    setLoading(true);
    try {
      const results = await searchIcons(query);
      setCandidates(results);
    } catch {
      setCandidates([]);
    }
    setLoading(false);
  };

  useEffect(() => {
    doSearch(appName);
  }, [appName]);

  const handlePick = async (candidate: IconCandidate) => {
    setSaving(candidate.icon_id);
    setPicked(candidate.icon_id);
    try {
      await setAppIcon(exeName, candidate.url);
      onPicked(candidate.url);
      // Small delay to show the pick animation
      setTimeout(() => onClose(), 300);
    } catch (e) {
      // silently ignore
      setSaving(null);
      setPicked(null);
    }
  };

  const handleCustomUrl = async () => {
    const url = prompt(t("iconPicker.enterUrl"));
    if (url && url.startsWith("http")) {
      setSaving("custom");
      try {
        await setAppIcon(exeName, url);
        onPicked(url);
        setTimeout(() => onClose(), 300);
      } catch (e) {
        // silently ignore
        setSaving(null);
      }
    }
  };

  return (
    <div
      className={closing ? "" : "popup-backdrop"}
      style={{
        position: "fixed", inset: 0, zIndex: 1000,
        background: "rgba(0,0,0,0.4)", backdropFilter: "blur(12px)",
        WebkitBackdropFilter: "blur(12px)",
        display: "flex", alignItems: "center", justifyContent: "center",
        animation: closing ? "backdropOut 0.2s ease forwards" : undefined,
      }}
      onClick={(e) => { if (e.target === e.currentTarget) animateClose(); }}
    >
      <div
        className={`card ${closing ? "" : "popup-card"}`}
        style={{
          width: 440, maxHeight: "80vh", padding: 24,
          display: "flex", flexDirection: "column", gap: 16,
          boxShadow: "0 24px 80px rgba(0,0,0,0.25), 0 0 1px rgba(0,0,0,0.1)",
          animation: closing ? "popOut 0.2s cubic-bezier(0.4, 0, 0.2, 1) forwards" : undefined,
        }}
      >
        {/* Header */}
        <div>
          <div style={{ fontSize: 17, fontWeight: 600, color: "var(--text-1)", letterSpacing: "-0.02em" }}>
            {t("iconPicker.title")}
          </div>
          <div style={{ fontSize: 12, color: "var(--text-3)", marginTop: 3 }}>
            {t("iconPicker.pickFor")} <strong style={{ color: "var(--text-2)" }}>{appName}</strong>
          </div>
        </div>

        {/* Search */}
        <div style={{ display: "flex", gap: 8 }}>
          <input
            className="input-apple"
            value={customQuery}
            onChange={(e) => setCustomQuery(e.target.value)}
            onKeyDown={(e) => { if (e.key === "Enter") doSearch(customQuery); }}
            placeholder={t("iconPicker.searchPlaceholder")}
            style={{ flex: 1 }}
            autoFocus
          />
          <button
            className="btn-primary"
            onClick={() => doSearch(customQuery)}
            style={{ padding: "7px 16px", fontSize: 12, borderRadius: 8 }}
          >
            {t("iconPicker.search")}
          </button>
        </div>

        {/* Icon Grid */}
        <div style={{ flex: 1, overflowY: "auto", minHeight: 200 }}>
          {loading ? (
            /* Skeleton loading grid */
            <div style={{
              display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: 10,
            }}>
              {Array.from({ length: 8 }).map((_, i) => (
                <div key={i} className="skeleton" style={{ height: 80, borderRadius: 12 }} />
              ))}
            </div>
          ) : candidates.length === 0 ? (
            <div style={{
              display: "flex", flexDirection: "column", alignItems: "center",
              justifyContent: "center", height: 200, gap: 10,
            }}>
              <span style={{ fontSize: 32, opacity: 0.6 }}>🔍</span>
              <span style={{ fontSize: 13, color: "var(--text-3)" }}>
                {t("iconPicker.noIcons")}
              </span>
            </div>
          ) : (
            <div style={{
              display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: 10,
            }}>
              {candidates.map((c, index) => (
                <button
                  key={c.icon_id}
                  className="icon-grid-item"
                  onClick={() => handlePick(c)}
                  disabled={saving !== null}
                  style={{
                    display: "flex", flexDirection: "column", alignItems: "center",
                    gap: 6, padding: 12, borderRadius: 12,
                    border: picked === c.icon_id
                      ? "2px solid var(--accent)"
                      : "1px solid var(--border-light)",
                    background: picked === c.icon_id
                      ? "var(--accent-bg)"
                      : "var(--bg-input)",
                    cursor: saving ? "wait" : "pointer",
                    opacity: saving && saving !== c.icon_id ? 0.3 : 1,
                    animationDelay: `${index * 30}ms`,
                    animationFillMode: "backwards",
                    animation: `popIn 0.3s cubic-bezier(0.34, 1.56, 0.64, 1) ${index * 30}ms backwards`,
                  }}
                  title={c.icon_id}
                >
                  <img
                    src={c.url}
                    alt={c.icon_id}
                    style={{
                      width: 44, height: 44, objectFit: "contain",
                      borderRadius: 6,
                    }}
                    onError={(e) => {
                      (e.target as HTMLImageElement).style.display = "none";
                    }}
                  />
                  <span style={{
                    fontSize: 9, color: "var(--text-3)",
                    maxWidth: "100%", overflow: "hidden",
                    textOverflow: "ellipsis", whiteSpace: "nowrap",
                    fontWeight: 500,
                  }}>
                    {c.set_name}
                  </span>
                  {picked === c.icon_id && (
                    <div style={{
                      position: "absolute", top: 4, right: 4,
                      width: 18, height: 18, borderRadius: "50%",
                      background: "var(--accent)", color: "white",
                      display: "flex", alignItems: "center", justifyContent: "center",
                      fontSize: 11, fontWeight: 700,
                      animation: "popIn 0.3s cubic-bezier(0.34, 1.56, 0.64, 1)",
                    }}>
                      ✓
                    </div>
                  )}
                </button>
              ))}
            </div>
          )}
        </div>

        {/* Footer */}
        <div style={{
          display: "flex", justifyContent: "space-between", alignItems: "center",
          paddingTop: 4, borderTop: "1px solid var(--border-light)",
        }}>
          <button
            className="btn-secondary"
            onClick={handleCustomUrl}
            style={{ padding: "7px 14px", fontSize: 11, borderRadius: 8 }}
          >
            {t("iconPicker.customUrl")}
          </button>
          <button
            className="btn-secondary"
            onClick={animateClose}
            style={{ padding: "7px 16px", fontSize: 12, borderRadius: 8 }}
          >
            {t("iconPicker.cancel")}
          </button>
        </div>
      </div>
    </div>
  );
}
