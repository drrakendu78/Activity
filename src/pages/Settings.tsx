import { useEffect, useState, useRef } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import {
  loadConfig, saveConfig, isAutoStartupEnabled,
  enableAutoStartup, disableAutoStartup, type RpcConfig,
  checkForUpdates, startSilentUpdate, relaunchApp, setTrayLanguage, type UpdateInfo,
} from "../lib/commands";
import { useTheme } from "../lib/theme";
import { LANGUAGES, getFlagUrl } from "../i18n";

export default function Settings() {
  const { t, i18n } = useTranslation();
  const [config, setConfig] = useState<RpcConfig | null>(null);
  const [autoStart, setAutoStart] = useState(false);
  const [saved, setSaved] = useState(false);
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [updateChecking, setUpdateChecking] = useState(false);
  const [updateDownloading, setUpdateDownloading] = useState(false);
  const [updateError, setUpdateError] = useState<string | null>(null);
  const [showRestartDialog, setShowRestartDialog] = useState(false);
  const { theme, toggle: toggleTheme } = useTheme();

  useEffect(() => {
    loadConfig().then(setConfig);
    isAutoStartupEnabled().then(setAutoStart).catch(() => {});
    // Auto-check for updates on load
    checkForUpdates().then(setUpdateInfo).catch(() => {});
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
    } catch {}
  };

  const handleLanguage = (code: string) => {
    i18n.changeLanguage(code);
    localStorage.setItem("language", code);
    setTrayLanguage(code);
    if (config) {
      const updated = { ...config, language: code };
      setConfig(updated);
      saveConfig(updated);
    }
  };

  if (!config) return <div style={{ padding: 24, color: "var(--text-3)" }}>{t("apps.loading")}</div>;

  return (
    <div style={{ padding: 20, display: "flex", flexDirection: "column", gap: 14 }}>
      {/* Appearance */}
      <div className="card" style={{ padding: 16 }}>
        <div className="section-label" style={{ marginBottom: 12 }}>{t("settings.appearance")}</div>
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
          <div>
            <div style={{ fontSize: 13, fontWeight: 500, color: "var(--text-1)" }}>{t("settings.darkMode")}</div>
            <div style={{ fontSize: 11, color: "var(--text-2)", marginTop: 1 }}>{t("settings.darkModeDesc")}</div>
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
          <div style={{ fontSize: 11, color: "var(--text-2)", marginBottom: 10 }}>{t("settings.languageDesc")}</div>
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


      {/* Music Detection */}
      <div className="card" style={{ padding: 16 }}>
        <div className="section-label" style={{ marginBottom: 12 }}>{t("settings.musicDetection")}</div>
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: config.music_enabled ? 16 : 0 }}>
          <div>
            <div style={{ fontSize: 13, fontWeight: 500, color: "var(--text-1)" }}>{t("settings.musicEnabled")}</div>
            <div style={{ fontSize: 11, color: "var(--text-2)", marginTop: 1 }}>{t("settings.musicEnabledDesc")}</div>
          </div>
          <button
            onClick={() => setConfig({ ...config, music_enabled: !config.music_enabled })}
            className={`toggle-apple ${config.music_enabled ? "active" : ""}`}
          >
            <span className="thumb" />
          </button>
        </div>

        {config.music_enabled && (() => {
          const mc = config.music_config;
          const setMc = (patch: Partial<typeof mc>) =>
            setConfig({ ...config, music_config: { ...mc, ...patch } });

          const CheckRow = ({ label, checked, onChange }: { label: string; checked: boolean; onChange: (v: boolean) => void }) => (
            <label style={{ display: "flex", alignItems: "center", gap: 8, padding: "5px 0", fontSize: 12, color: "var(--text-1)", cursor: "pointer" }}>
              <input type="checkbox" checked={checked} onChange={(e) => onChange(e.target.checked)}
                style={{ accentColor: "var(--accent)", width: 15, height: 15 }} />
              {label}
            </label>
          );

          const CustomSelect = ({ label, value, onChange, options }: {
            label: string; value: string; onChange: (v: string) => void;
            options: { value: string; label: string }[];
          }) => {
            const [open, setOpen] = useState(false);
            const btnRef = useRef<HTMLButtonElement>(null);
            const dropRef = useRef<HTMLDivElement>(null);
            const [dropPos, setDropPos] = useState({ top: 0, left: 0, width: 0 });
            const selected = options.find((o) => o.value === value);

            useEffect(() => {
              if (!open) return;
              const handler = (e: MouseEvent) => {
                if (btnRef.current?.contains(e.target as Node)) return;
                if (dropRef.current?.contains(e.target as Node)) return;
                setOpen(false);
              };
              document.addEventListener("mousedown", handler);
              return () => document.removeEventListener("mousedown", handler);
            }, [open]);

            const handleOpen = () => {
              if (!open && btnRef.current) {
                const rect = btnRef.current.getBoundingClientRect();
                setDropPos({ top: rect.bottom + 4, left: rect.left, width: rect.width });
              }
              setOpen(!open);
            };

            return (
              <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "5px 0" }}>
                <span style={{ fontSize: 12, color: "var(--text-1)" }}>{label}</span>
                <div style={{ position: "relative", minWidth: 170 }}>
                  <button
                    ref={btnRef}
                    onClick={handleOpen}
                    style={{
                      width: "100%", display: "flex", alignItems: "center", justifyContent: "space-between",
                      padding: "5px 10px", fontSize: 11, fontFamily: "inherit",
                      background: "var(--bg-input)", border: "1px solid var(--border)",
                      borderRadius: 7, color: "var(--text-1)", cursor: "pointer",
                    }}
                  >
                    <span>{selected?.label || value}</span>
                    <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="var(--text-3)" strokeWidth="2.5"><path d="M6 9l6 6 6-6" /></svg>
                  </button>
                  {open && createPortal(
                    <div ref={dropRef} style={{
                      position: "fixed", top: dropPos.top, left: dropPos.left, width: dropPos.width, zIndex: 9999,
                      background: "var(--bg-elevated)", border: "1px solid var(--border)",
                      borderRadius: 8, overflow: "hidden", boxShadow: "0 8px 24px rgba(0,0,0,0.3)",
                    }}>
                      {options.map((opt) => (
                        <div
                          key={opt.value}
                          onClick={() => { onChange(opt.value); setOpen(false); }}
                          style={{
                            padding: "7px 12px", fontSize: 11, cursor: "pointer",
                            color: opt.value === value ? "var(--accent)" : "var(--text-1)",
                            background: opt.value === value ? "var(--accent-bg)" : "transparent",
                            fontWeight: opt.value === value ? 600 : 400,
                          }}
                          onMouseEnter={(e) => { if (opt.value !== value) e.currentTarget.style.background = "var(--bg-content)"; }}
                          onMouseLeave={(e) => { e.currentTarget.style.background = opt.value === value ? "var(--accent-bg)" : "transparent"; }}
                        >
                          {opt.label}
                        </div>
                      ))}
                    </div>,
                    document.body
                  )}
                </div>
              </div>
            );
          };

          return (
            <>
              {/* Présence */}
              <div style={{ fontSize: 11, fontWeight: 600, color: "var(--text-2)", textTransform: "uppercase", letterSpacing: "0.04em", marginBottom: 8 }}>
                {t("settings.musicPresence")}
              </div>

              <CustomSelect label={t("settings.musicActivityType")} value={mc.activity_type} onChange={(v) => setMc({ activity_type: v })}
                options={[
                  { value: "listening", label: t("settings.musicActivityListening") },
                  { value: "watching", label: t("settings.musicActivityWatching") },
                  { value: "playing", label: t("settings.musicActivityPlaying") },
                ]} />

              <CustomSelect label={t("settings.musicDisplayText")} value={mc.display_text} onChange={(v) => setMc({ display_text: v })}
                options={[
                  { value: "player", label: t("settings.musicDisplayPlayer") },
                  { value: "artist", label: t("settings.musicDisplayArtist") },
                  { value: "title", label: t("settings.musicDisplayTitle") },
                  { value: "media_type", label: t("settings.musicDisplayMediaType") },
                  { value: "custom", label: t("settings.musicDisplayCustom") },
                ]} />
              {mc.display_text === "custom" && (
                <input className="input-apple" value={mc.display_text_custom}
                  onChange={(e) => setMc({ display_text_custom: e.target.value })}
                  placeholder={t("settings.musicDisplayCustom")}
                  style={{ fontSize: 11, padding: "4px 8px", marginBottom: 4, width: "100%" }} />
              )}

              <CustomSelect label={t("settings.musicProfileText")} value={mc.profile_text} onChange={(v) => setMc({ profile_text: v })}
                options={[
                  { value: "player", label: t("settings.musicDisplayPlayer") },
                  { value: "media_type", label: t("settings.musicDisplayMediaType") },
                  { value: "custom", label: t("settings.musicDisplayCustom") },
                ]} />
              {mc.profile_text === "custom" && (
                <input className="input-apple" value={mc.profile_text_custom}
                  onChange={(e) => setMc({ profile_text_custom: e.target.value })}
                  placeholder={t("settings.musicDisplayCustom")}
                  style={{ fontSize: 11, padding: "4px 8px", marginBottom: 4, width: "100%" }} />
              )}

              {/* Informations sur la musique */}
              <div style={{ fontSize: 11, fontWeight: 600, color: "var(--text-2)", textTransform: "uppercase", letterSpacing: "0.04em", margin: "14px 0 8px" }}>
                {t("settings.musicInfo")}
              </div>

              <CheckRow label={t("settings.musicTitleArtistOneLine")} checked={mc.title_artist_one_line} onChange={(v) => setMc({ title_artist_one_line: v })} />
              <CheckRow label={t("settings.musicArtistAlbumOneLine")} checked={mc.artist_album_one_line} onChange={(v) => setMc({ artist_album_one_line: v })} />
              <CheckRow label={t("settings.musicReverseOrder")} checked={mc.reverse_title_artist} onChange={(v) => setMc({ reverse_title_artist: v })} />
              <CheckRow label={t("settings.musicPrefixBy")} checked={mc.prefix_artist_by} onChange={(v) => setMc({ prefix_artist_by: v })} />
              <CheckRow label={t("settings.musicPrefixOn")} checked={mc.prefix_album_on} onChange={(v) => setMc({ prefix_album_on: v })} />
              <CheckRow label={t("settings.musicShowAlbum")} checked={mc.show_album} onChange={(v) => setMc({ show_album: v })} />
              <CheckRow label={t("settings.musicShowAlbumNoArtist")} checked={mc.show_album_when_no_artist} onChange={(v) => setMc({ show_album_when_no_artist: v })} />
              <CheckRow label={t("settings.musicShowPlayback")} checked={mc.show_playback_info} onChange={(v) => setMc({ show_playback_info: v })} />
              <CheckRow label={t("settings.musicHideInfo")} checked={mc.hide_music_info} onChange={(v) => setMc({ hide_music_info: v })} />

              {/* Média en pause */}
              <div style={{ fontSize: 11, fontWeight: 600, color: "var(--text-2)", textTransform: "uppercase", letterSpacing: "0.04em", margin: "14px 0 8px" }}>
                {t("settings.musicPaused")}
              </div>

              <CheckRow label={t("settings.musicShowPaused")} checked={mc.show_paused} onChange={(v) => setMc({ show_paused: v })} />
              <CheckRow label={t("settings.musicShowPauseIcon")} checked={mc.show_pause_icon} onChange={(v) => setMc({ show_pause_icon: v })} />
              <CheckRow label={t("settings.musicFreezeProgress")} checked={mc.freeze_progress_when_paused} onChange={(v) => setMc({ freeze_progress_when_paused: v })} />
              <CheckRow label={t("settings.musicShowPausedDuration")} checked={mc.show_paused_duration} onChange={(v) => setMc({ show_paused_duration: v })} />

              {/* Lecteurs hors-ligne */}
              <div style={{ fontSize: 11, fontWeight: 600, color: "var(--text-2)", textTransform: "uppercase", letterSpacing: "0.04em", margin: "14px 0 8px" }}>
                {t("settings.musicOffline")}
              </div>
              <CheckRow label={t("settings.musicShowPlayIcon")} checked={mc.show_play_icon} onChange={(v) => setMc({ show_play_icon: v })} />
              <CheckRow label={t("settings.musicShowPlayerLogo")} checked={mc.show_player_logo} onChange={(v) => setMc({ show_player_logo: v })} />
              <div style={{ fontSize: 10, color: "var(--text-2)", marginTop: 2, fontStyle: "italic" }}>
                {t("settings.musicOfflineNote")}
              </div>

              {/* Boutons */}
              <div style={{ fontSize: 11, fontWeight: 600, color: "var(--text-2)", textTransform: "uppercase", letterSpacing: "0.04em", margin: "14px 0 8px" }}>
                {t("settings.musicButtons")}
              </div>

              <CheckRow label={t("settings.musicShowGetStatusButton")} checked={mc.show_get_status_button} onChange={(v) => setMc({ show_get_status_button: v })} />
              <CheckRow label={t("settings.musicShowListenButton")} checked={mc.show_listen_button} onChange={(v) => setMc({ show_listen_button: v })} />

              {/* Divers */}
              <div style={{ fontSize: 11, fontWeight: 600, color: "var(--text-2)", textTransform: "uppercase", letterSpacing: "0.04em", margin: "14px 0 8px" }}>
                {t("settings.musicMisc")}
              </div>

              <CustomSelect label={t("settings.musicFallbackCover")} value={mc.fallback_cover} onChange={(v) => setMc({ fallback_cover: v })}
                options={[
                  { value: "player", label: t("settings.musicFallbackPlayer") },
                  { value: "note", label: t("settings.musicFallbackNote") },
                  { value: "playback", label: t("settings.musicFallbackPlayback") },
                  { value: "cd", label: t("settings.musicFallbackCd") },
                  { value: "app", label: t("settings.musicFallbackApp") },
                ]} />
            </>
          );
        })()}
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
          <div style={{ display: "flex", justifyContent: "space-between", fontSize: 10, color: "var(--text-2)", marginTop: 4 }}>
            <span>1s</span><span>10s</span>
          </div>
        </div>
      </div>

      {/* Default Presence */}
      <div className="card" style={{ padding: 16 }}>
        <div className="section-label" style={{ marginBottom: 12 }}>{t("settings.defaultPresence")}</div>
        <p style={{ fontSize: 11, color: "var(--text-2)", marginBottom: 14 }}>
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
              <div style={{ fontSize: 11, color: "var(--text-2)", marginTop: 1 }}>
                {config.default.state}
              </div>
            )}
            <div style={{ fontSize: 10, color: "var(--text-2)", marginTop: 3 }}>
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
          <p style={{ fontSize: 11, color: "var(--text-2)", marginTop: 6 }}>
            {t("settings.apiKeyDesc")}
          </p>
        </div>
      </div>

      {/* Behavior */}
      <div className="card" style={{ padding: 16 }}>
        <div className="section-label" style={{ marginBottom: 12 }}>{t("settings.behavior")}</div>

        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 12 }}>
          <div>
            <div style={{ fontSize: 13, fontWeight: 500, color: "var(--text-1)" }}>{t("settings.startOnBoot")}</div>
            <div style={{ fontSize: 11, color: "var(--text-2)", marginTop: 1 }}>{t("settings.startOnBootDesc")}</div>
          </div>
          <button onClick={handleAutoStart} className={`toggle-apple ${autoStart ? "active" : ""}`}>
            <span className="thumb" />
          </button>
        </div>

        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 12 }}>
          <div>
            <div style={{ fontSize: 13, fontWeight: 500, color: "var(--text-1)" }}>{t("settings.autoStartService")}</div>
            <div style={{ fontSize: 11, color: "var(--text-2)", marginTop: 1 }}>{t("settings.autoStartServiceDesc")}</div>
          </div>
          <button
            onClick={() => setConfig({ ...config, auto_start_service: !config.auto_start_service })}
            className={`toggle-apple ${config.auto_start_service ? "active" : ""}`}
          >
            <span className="thumb" />
          </button>
        </div>

        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
          <div>
            <div style={{ fontSize: 13, fontWeight: 500, color: "var(--text-1)" }}>{t("settings.hideTrayIcon")}</div>
            <div style={{ fontSize: 11, color: "var(--text-2)", marginTop: 1 }}>{t("settings.hideTrayIconDesc")}</div>
          </div>
          <button
            onClick={async () => {
              const newConfig = { ...config, hide_tray_icon: !config.hide_tray_icon };
              setConfig(newConfig);
              await saveConfig(newConfig);
              setShowRestartDialog(true);
            }}
            className={`toggle-apple ${config.hide_tray_icon ? "active" : ""}`}
          >
            <span className="thumb" />
          </button>
        </div>
      </div>

      {/* Updates */}
      <div className="card" style={{ padding: 16 }}>
        <div className="section-label" style={{ marginBottom: 12 }}>{t("update.title")}</div>
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
          <div>
            <div style={{ fontSize: 13, fontWeight: 500, color: "var(--text-1)" }}>
              {t("update.currentVersion")} : {updateInfo?.current_version || "..."}
            </div>
            {updateInfo?.has_update ? (
              <div style={{ fontSize: 11, color: "var(--green)", marginTop: 2, fontWeight: 600 }}>
                v{updateInfo.latest_version} {t("update.latestAvailable")}
              </div>
            ) : updateInfo ? (
              <div style={{ fontSize: 11, color: "var(--text-2)", marginTop: 2 }}>
                {t("update.upToDate")}
              </div>
            ) : null}
            {updateInfo?.has_update && updateInfo.install_type && (
              <div style={{ fontSize: 10, color: "var(--text-2)", marginTop: 2 }}>
                {t("update.installType")} : {updateInfo.install_type.toUpperCase()}
              </div>
            )}
          </div>
          <div style={{ display: "flex", gap: 6 }}>
            <button
              onClick={async () => {
                setUpdateChecking(true);
                setUpdateError(null);
                try {
                  const info = await checkForUpdates();
                  setUpdateInfo(info);
                } catch (e) {
                  const msg = String(e);
                  if (msg.includes("403") || msg.includes("rate")) {
                    setUpdateError(t("update.rateLimited"));
                  } else {
                    setUpdateError(t("update.checkError") + (msg ? ` (${msg})` : ""));
                  }
                }
                setUpdateChecking(false);
              }}
              className="btn-secondary"
              style={{ fontSize: 11, padding: "6px 12px", borderRadius: 8, opacity: updateChecking ? 0.6 : 1 }}
              disabled={updateChecking}
            >
              {updateChecking ? t("update.checking") : t("update.check")}
            </button>
            {updateInfo?.has_update && updateInfo.download_url && updateInfo.file_name && (
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
                style={{ fontSize: 11, padding: "6px 12px", borderRadius: 8, opacity: updateDownloading ? 0.6 : 1 }}
                disabled={updateDownloading}
              >
                {updateDownloading ? t("update.downloading") : t("update.install")}
              </button>
            )}
          </div>
        </div>
        {updateError && (
          <div style={{ fontSize: 11, color: "#ef4444", marginTop: 8 }}>{updateError}</div>
        )}
        {updateInfo?.has_update && updateInfo.release_notes && (
          <div style={{
            marginTop: 10, padding: 10, borderRadius: 8,
            background: "var(--bg-content)", fontSize: 11, color: "var(--text-2)",
            maxHeight: 100, overflowY: "auto", whiteSpace: "pre-wrap", lineHeight: 1.5,
          }}>
            {updateInfo.release_notes}
          </div>
        )}
      </div>

      {/* Save */}
      <button onClick={handleSave} className="btn-primary"
        style={{
          width: "100%", padding: "10px 0", fontSize: 14, borderRadius: 10,
          background: saved ? "var(--green)" : undefined,
        }}>
        {saved ? `\u2713 ${t("settings.saved")}` : t("settings.saveSettings")}
      </button>

      {showRestartDialog && createPortal(
        <div style={{
          position: "fixed", top: 0, left: 0, right: 0, bottom: 0,
          background: "rgba(0,0,0,0.6)", display: "flex", alignItems: "center", justifyContent: "center",
          zIndex: 10000,
        }} onClick={() => setShowRestartDialog(false)}>
          <div style={{
            background: "#1e1f22", borderRadius: 12, padding: 24, minWidth: 320, maxWidth: 400,
            boxShadow: "0 8px 32px rgba(0,0,0,0.4)", border: "1px solid rgba(255,255,255,0.08)",
          }} onClick={e => e.stopPropagation()}>
            <div style={{ fontSize: 16, fontWeight: 600, color: "var(--text-1)", marginBottom: 8 }}>
              {t("settings.restartRequired")}
            </div>
            <div style={{ fontSize: 13, color: "var(--text-2)", marginBottom: 20 }}>
              {t("settings.restartRequiredDesc")}
            </div>
            <div style={{ display: "flex", gap: 8, justifyContent: "flex-end" }}>
              <button
                onClick={() => setShowRestartDialog(false)}
                className="btn-secondary"
                style={{ padding: "8px 16px", borderRadius: 8, fontSize: 13 }}
              >
                {t("settings.restartLater")}
              </button>
              <button
                onClick={() => relaunchApp()}
                className="btn-primary"
                style={{ padding: "8px 16px", borderRadius: 8, fontSize: 13 }}
              >
                {t("settings.restartNow")}
              </button>
            </div>
          </div>
        </div>,
        document.body
      )}
    </div>
  );
}
