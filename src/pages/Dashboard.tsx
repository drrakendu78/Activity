import { useEffect, useState, useRef } from "react";
import { useTranslation } from "react-i18next";
import { listen } from "@tauri-apps/api/event";
import {
  isPollerRunning,
  getRpcStatus,
  mediaPlayPause,
  mediaNext,
  mediaPrevious,
  mediaSeek,
  openUrl,
  getAlbumTracks,
  getSystemVolume,
  setSystemVolume,
  toggleMute,
  type RpcStatusEvent,
  type AlbumInfo,
} from "../lib/commands";
import IconPicker from "../components/IconPicker";
import { resizeWindow } from "../App";

const CATEGORY_EMOJI: Record<string, string> = {
  gaming: "\u{1F3AE}", dev: "\u{1F4BB}", browser: "\u{1F310}", social: "\u{1F4AC}",
  media: "\u{1F3B5}", creative: "\u{1F3A8}", productivity: "\u{1F4C4}", system: "\u{2699}\u{FE0F}",
  discovered: "\u{1F50D}", other: "\u{1F4E6}",
};

function formatElapsed(startTimestamp: number | null): string {
  if (!startTimestamp) return "";
  const now = Math.floor(Date.now() / 1000);
  const elapsed = now - startTimestamp;
  if (elapsed < 0) return "";
  const h = Math.floor(elapsed / 3600);
  const m = Math.floor((elapsed % 3600) / 60);
  const s = elapsed % 60;
  if (h > 0) return `${h}h ${m.toString().padStart(2, "0")}m`;
  return `${m}m ${s.toString().padStart(2, "0")}s`;
}

export default function Dashboard({ waitingForDiscord }: { waitingForDiscord?: boolean }) {
  const { t } = useTranslation();
  const [running, setRunning] = useState(false);
  const [connected, setConnected] = useState(false);
  const [currentApp, setCurrentApp] = useState<string | null>(null);
  const [exeName, setExeName] = useState<string | null>(null);
  const [windowTitle, setWindowTitle] = useState<string | null>(null);
  const [category, setCategory] = useState<string | null>(null);
  const [details, setDetails] = useState<string | null>(null);
  const [rpcState, setRpcState] = useState<string | null>(null);
  const [largeImageKey, setLargeImageKey] = useState<string | null>(null);
  const [smallImageKey, setSmallImageKey] = useState<string | null>(null);
  const [startTimestamp, setStartTimestamp] = useState<number | null>(null);
  const [playbackStatus, setPlaybackStatus] = useState<string | null>(null);
  const [albumName, setAlbumName] = useState<string | null>(null);
  const [listenUrl, setListenUrl] = useState<string | null>(null);
  const [isMusic, setIsMusic] = useState(false);
  const [positionSecs, setPositionSecs] = useState<number | null>(null);
  const [durationSecs, setDurationSecs] = useState<number | null>(null);
  const [lastUpdateTime, setLastUpdateTime] = useState<number>(Date.now());
  const [currentPosition, setCurrentPosition] = useState<number>(0);
  const [isSeeking, setIsSeeking] = useState(false);
  const [elapsed, setElapsed] = useState("");
  const [showIconPicker, setShowIconPicker] = useState(false);
  const [volume, setVolume] = useState(1);
  const [muted, setMuted] = useState(false);
  const [albumInfo, setAlbumInfo] = useState<AlbumInfo | null>(null);
  const [showTracklist, setShowTracklist] = useState(false);
  const [albumFetchKey, setAlbumFetchKey] = useState("");

  const isMedia = category === "media" && !isMusic;
  const hasIcon = largeImageKey && largeImageKey.startsWith("http");

  // Ref to always have the latest "details" value inside async closures
  const detailsRef = useRef(details);
  detailsRef.current = details;

  // Debounce for media control buttons (prevent double-clicks)


  useEffect(() => {
    isPollerRunning().then(setRunning);
    getRpcStatus().then((s) => {
      setConnected(s.connected);
      setCurrentApp(s.current_app);
    });
    const unlisten = listen<RpcStatusEvent>("rpc-status-changed", (event) => {
      const p = event.payload;
      setConnected(p.connected);
      // If we receive an event with a current_app, the poller must be running
      if (p.current_app) setRunning(true);
      setCurrentApp(p.current_app);
      setExeName(p.exe_name);
      setWindowTitle(p.window_title);
      setCategory(p.category);
      setDetails(p.details);
      setRpcState(p.state);
      setLargeImageKey(p.large_image_key);
      setSmallImageKey(p.small_image_key);
      setIsMusic(p.is_music ?? false);
      setAlbumName(p.album_name ?? null);
      setListenUrl(p.listen_url ?? null);
      setPlaybackStatus(p.playback_status ?? null);
      setPositionSecs(p.position_secs ?? null);
      setDurationSecs(p.duration_secs ?? null);
      setLastUpdateTime(Date.now());
      // Music: always update timestamp
      if (p.is_music || p.category === "media") {
        setStartTimestamp(p.start_timestamp ?? null);
      } else if (p.start_timestamp) {
        setStartTimestamp(p.start_timestamp);
      }
    });
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  useEffect(() => {
    if (!running) { setElapsed(""); return; }
    // Hide elapsed when music is paused
    if ((isMusic || isMedia) && playbackStatus && playbackStatus !== "Playing") {
      setElapsed(""); return;
    }
    if (!startTimestamp) { setElapsed(""); return; }
    setElapsed(formatElapsed(startTimestamp));
    const interval = setInterval(() => setElapsed(formatElapsed(startTimestamp)), 1000);
    return () => clearInterval(interval);
  }, [startTimestamp, running, playbackStatus, isMusic, isMedia]);

  // Interpolate position in real-time (updated every 250ms for smooth slider)
  useEffect(() => {
    if (positionSecs == null || !running || isSeeking) return;
    setCurrentPosition(positionSecs);
    if (playbackStatus !== "Playing") return;
    const interval = setInterval(() => {
      const elapsed = (Date.now() - lastUpdateTime) / 1000;
      const pos = positionSecs + elapsed;
      setCurrentPosition(durationSecs ? Math.min(pos, durationSecs) : pos);
    }, 250);
    return () => clearInterval(interval);
  }, [positionSecs, lastUpdateTime, playbackStatus, running, durationSecs, isSeeking]);

  // Load system volume and poll for changes
  useEffect(() => {
    if (!isMusic && !isMedia) return;
    getSystemVolume(exeName ?? undefined).then(([v, m]) => { setVolume(v); setMuted(m); }).catch(() => {});
    const interval = setInterval(() => {
      getSystemVolume(exeName ?? undefined).then(([v, m]) => { setVolume(v); setMuted(m); }).catch(() => {});
    }, 2000);
    return () => clearInterval(interval);
  }, [isMusic, isMedia, exeName]);

  // Fetch album tracks when song changes
  useEffect(() => {
    if (!isMusic || !rpcState || !details) {
      // Don't clear albumInfo if we just temporarily lost music state
      if (!isMusic) {
        setAlbumInfo(null);
        setAlbumFetchKey(""); // Reset so we re-fetch when music comes back
      }
      return;
    }
    // rpcState may contain "Artist — Album", extract just the artist
    const artist = rpcState.split(" — ")[0].split(" - ")[0].trim();
    // Skip placeholder/loading states from music players
    const skipArtists = ["apple music", "spotify", "deezer", "tidal", "music"];
    const skipTitles = ["connexion", "connecting", "loading", "chargement"];
    if (skipArtists.includes(artist.toLowerCase()) || skipTitles.some(s => details.toLowerCase().includes(s))) {
      return;
    }
    const key = `${artist}|${details}`;
    if (key === albumFetchKey) return;
    setAlbumFetchKey(key);
    getAlbumTracks(artist, details)
      .then((info) => {
        console.log("[Tracklist] fetched:", info);
        setAlbumInfo(info);
      })
      .catch((err) => {
        console.error("[Tracklist] error:", err);
        setAlbumInfo(null);
      });
  }, [isMusic, rpcState, details]);


  // Fuzzy track name matching — strips "(feat. ...)", "[Bonus Track]", etc.
  const normalizeTrack = (s: string) =>
    s.toLowerCase().replace(/\s*[\(\[].*?[\)\]]/g, "").trim();
  const matchesTrack = (a: string | null | undefined, b: string) => {
    if (!a) return false;
    const na = normalizeTrack(a), nb = normalizeTrack(b);
    return na === nb || na.startsWith(nb) || nb.startsWith(na);
  };

  // Skip to a specific track — calculate once, spam skips fast, verify at end
  const [skipping, setSkipping] = useState(false);
  const skipToTrack = async (targetNum: number) => {
    if (!albumInfo || skipping) return;
    const currentTrack = albumInfo.tracks.find(
      (t) => matchesTrack(detailsRef.current, t.name)
    );
    if (!currentTrack) return;
    const diff = targetNum - currentTrack.track_number;
    if (diff === 0) return;

    setSkipping(true);
    const fn = diff > 0 ? mediaNext : mediaPrevious;
    const count = Math.abs(diff);

    // Send each skip and wait for the track to actually change before the next one
    for (let i = 0; i < count; i++) {
      const before = detailsRef.current?.toLowerCase();
      await fn().catch(() => {});
      // Poll until track name changes (max 200ms per skip)
      for (let t = 0; t < 4; t++) {
        await new Promise((r) => setTimeout(r, 50));
        if (detailsRef.current?.toLowerCase() !== before) break;
      }
    }

    setSkipping(false);
  };

  const formatTime = (secs: number) => {
    const m = Math.floor(secs / 60);
    const s = Math.floor(secs % 60);
    return `${m}:${s.toString().padStart(2, "0")}`;
  };

  const handleSeek = async (value: number) => {
    setIsSeeking(false);
    setCurrentPosition(value);
    try { await mediaSeek(value); } catch {}
  };

  const emoji = CATEGORY_EMOJI[category || "other"] || "\u{1F4E6}";

  /* ═══════════════════════════════════════════════════
     WAITING FOR DISCORD
     ═══════════════════════════════════════════════════ */
  if (waitingForDiscord && !running) {
    return (
      <div className="player-fullscreen player-centered">
        <div className="spinner" style={{ width: 48, height: 48, borderWidth: 4 }} />
        <div className="player-info" style={{ textAlign: "center" }}>
          <div className="player-title">{t("dashboard.waitingForDiscord")}</div>
          <div className="player-artist">{t("dashboard.waitingForDiscordHint")}</div>
        </div>
      </div>
    );
  }

  /* ═══════════════════════════════════════════════════
     MUSIC / MEDIA — Album page layout (like streaming apps)
     ═══════════════════════════════════════════════════ */
  if (running && currentApp && (isMusic || isMedia)) {
    const artistName = rpcState?.split(" — ")[0]?.split(" - ")[0]?.trim();
    const albumDisplay = albumInfo?.album_name || albumName;

    return (
      <div className="album-page">
        {/* Foreground app toast */}
        {windowTitle && isMusic && (
          <div className="fg-app-toast">
            {smallImageKey && <img src={smallImageKey} alt="" className="fg-app-toast-icon" />}
            <span>{t("update.onPrefix")} {windowTitle}</span>
          </div>
        )}
        {/* Scrollable content */}
        <div className="album-page-content">
          {/* Album art */}
          <div className="album-page-cover">
            {hasIcon ? (
              <img
                src={largeImageKey!}
                alt={details || currentApp}
                onError={(e) => {
                  (e.target as HTMLImageElement).style.display = "none";
                  (e.target as HTMLImageElement).parentElement!.innerHTML =
                    `<span style="font-size:64px">${isMusic ? "\u{1F3B5}" : "\u{1F3AC}"}</span>`;
                }}
              />
            ) : (
              <span style={{ fontSize: 64 }}>{isMusic ? "\u{1F3B5}" : "\u{1F3AC}"}</span>
            )}
          </div>

          {/* Album info */}
          <div className="album-page-info">
            <div className="album-page-title">{details || currentApp}</div>
            <div className="album-page-artist">{artistName || ""}</div>
            <div className="album-page-album">{albumDisplay}</div>
            {albumInfo && (
              <div className="album-page-meta">
                {albumInfo.year && <span>{albumInfo.year}</span>}
                <span>{albumInfo.total_tracks} titres</span>
                <span>{Math.floor(albumInfo.total_duration_secs / 60)} min</span>
              </div>
            )}
          </div>

          {/* Action buttons */}
          <div className="album-page-actions">
            {(albumInfo?.deezer_url || listenUrl) && (
              <button onClick={() => {
                const url = albumInfo?.deezer_url
                  ? `https://song.link/${encodeURIComponent(albumInfo.deezer_url)}`
                  : listenUrl!;
                openUrl(url).catch(() => {});
              }} className="album-action-btn">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
                  <polyline points="15 3 21 3 21 9" />
                  <line x1="10" y1="14" x2="21" y2="3" />
                </svg>
              </button>
            )}
            {isMusic && albumInfo && albumInfo.tracks.length > 0 && (
              <button
                className={`album-action-btn ${showTracklist ? "album-action-active" : ""}`}
                onClick={() => {
                  if (!showTracklist) {
                    setShowTracklist(true);
                    const tracklistHeight = Math.min(80 + (albumInfo?.tracks.length ?? 0) * 42 + 20, 1000);
                    resizeWindow(Math.max(520, tracklistHeight + 80));
                  } else {
                    setShowTracklist(false);
                    resizeWindow(520);
                  }
                }}
                title="Tracklist"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <line x1="8" y1="6" x2="21" y2="6" />
                  <line x1="8" y1="12" x2="21" y2="12" />
                  <line x1="8" y1="18" x2="21" y2="18" />
                  <line x1="3" y1="6" x2="3.01" y2="6" />
                  <line x1="3" y1="12" x2="3.01" y2="12" />
                  <line x1="3" y1="18" x2="3.01" y2="18" />
                </svg>
              </button>
            )}
          </div>

        </div>

        {/* Bottom bar — controls + progress + volume */}
        <div className="album-bottom-bar">
          <div className="album-bottom-controls">
            <button className="mc-btn mc-skip" onClick={() => mediaPrevious().catch(() => {})}>
              <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor"><path d="M3.3 1a.7.7 0 0 1 .7.7v5.15l9.95-5.744a.7.7 0 0 1 1.05.606v12.575a.7.7 0 0 1-1.05.607L4 9.149V14.3a.7.7 0 0 1-.7.7H2.7a.7.7 0 0 1-.7-.7V1.7a.7.7 0 0 1 .7-.7h.6z"/></svg>
            </button>
            <button className="mc-btn mc-play" onClick={() => mediaPlayPause().catch(() => {})}>
              {playbackStatus === "Playing" ? (
                <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M2.7 1a.7.7 0 0 0-.7.7v12.6a.7.7 0 0 0 .7.7h2.6a.7.7 0 0 0 .7-.7V1.7a.7.7 0 0 0-.7-.7H2.7zm8 0a.7.7 0 0 0-.7.7v12.6a.7.7 0 0 0 .7.7h2.6a.7.7 0 0 0 .7-.7V1.7a.7.7 0 0 0-.7-.7h-2.6z"/></svg>
              ) : (
                <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor"><path d="M3 1.713a.7.7 0 0 1 1.05-.607l10.89 6.288a.7.7 0 0 1 0 1.212L4.05 14.894A.7.7 0 0 1 3 14.288V1.713z"/></svg>
              )}
            </button>
            <button className="mc-btn mc-skip" onClick={() => mediaNext().catch(() => {})}>
              <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor"><path d="M12.7 1a.7.7 0 0 0-.7.7v5.15L2.05 1.107A.7.7 0 0 0 1 1.712v12.575a.7.7 0 0 0 1.05.607L12 9.149V14.3a.7.7 0 0 0 .7.7h.6a.7.7 0 0 0 .7-.7V1.7a.7.7 0 0 0-.7-.7h-.6z"/></svg>
            </button>
          </div>

          {durationSecs != null && durationSecs > 0 && (
            <div className="album-bottom-progress">
              <span className="player-time">{formatTime(currentPosition)}</span>
              <input
                type="range"
                className="player-slider"
                min={0}
                max={durationSecs}
                step={0.5}
                value={isSeeking ? undefined : currentPosition}
                onChange={(e) => { setIsSeeking(true); setCurrentPosition(Number(e.target.value)); }}
                onMouseUp={(e) => handleSeek(Number((e.target as HTMLInputElement).value))}
                onTouchEnd={(e) => handleSeek(Number((e.target as HTMLInputElement).value))}
              />
              <span className="player-time">{formatTime(durationSecs)}</span>
            </div>
          )}

          <div className="album-bottom-right">
            {hasIcon && (
              <img src={largeImageKey!} alt="" className="album-bottom-thumb" />
            )}
            <div className="album-bottom-info">
              <div className="album-bottom-title">{details || currentApp}</div>
              <div className="album-bottom-sub">{artistName} · {albumDisplay}</div>
            </div>
            <div className="player-volume">
              <button className="volume-btn" onClick={() => toggleMute(exeName ?? undefined).then(setMuted).catch(() => {})}>
                {muted || volume === 0 ? (
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />
                    <line x1="23" y1="9" x2="17" y2="15" />
                    <line x1="17" y1="9" x2="23" y2="15" />
                  </svg>
                ) : volume < 0.5 ? (
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />
                    <path d="M15.54 8.46a5 5 0 0 1 0 7.07" />
                  </svg>
                ) : (
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />
                    <path d="M15.54 8.46a5 5 0 0 1 0 7.07" />
                    <path d="M19.07 4.93a10 10 0 0 1 0 14.14" />
                  </svg>
                )}
              </button>
              <input
                type="range"
                className="volume-slider"
                min={0} max={1} step={0.02}
                value={muted ? 0 : volume}
                onChange={(e) => {
                  const v = Number(e.target.value);
                  setVolume(v); setMuted(v === 0);
                  setSystemVolume(v, exeName ?? undefined).catch(() => {});
                }}
              />
            </div>
          </div>
        </div>

        {/* Tracklist modal overlay */}
        {showTracklist && albumInfo && (
          <div
            className="settings-overlay"
            onClick={(e) => { if (e.target === e.currentTarget) { setShowTracklist(false); resizeWindow(520); } }}
          >
            <div className="settings-modal" style={{ maxHeight: "85vh" }}>
              <div className="settings-modal-header">
                <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
                  {albumInfo.album_art_url && (
                    <img src={albumInfo.album_art_url} alt="" style={{ width: 40, height: 40, borderRadius: 6, objectFit: "cover" }} />
                  )}
                  <div>
                    <h2 style={{ fontSize: 14, fontWeight: 600, color: "var(--text-1)", margin: 0 }}>
                      {albumInfo.album_name}
                    </h2>
                    <div style={{ fontSize: 11, color: "var(--text-2)" }}>
                      {albumInfo.artist_name} {albumInfo.year && `· ${albumInfo.year}`} · {albumInfo.total_tracks} titres · {Math.floor(albumInfo.total_duration_secs / 60)} min
                    </div>
                  </div>
                </div>
                <button onClick={() => { setShowTracklist(false); resizeWindow(520); }} className="settings-close-btn">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                    <line x1="18" y1="6" x2="6" y2="18" />
                    <line x1="6" y1="6" x2="18" y2="18" />
                  </svg>
                </button>
              </div>
              <div className="settings-modal-body" style={{ padding: 0 }}>
                <div className="tracklist">
                  {albumInfo.tracks.map((track) => {
                    const isCurrent = matchesTrack(details, track.name);
                    return (
                      <div
                        key={track.track_number}
                        className={`tracklist-item ${isCurrent ? "tracklist-active" : ""} ${skipping ? "tracklist-disabled" : ""}`}
                        onClick={() => !isCurrent && !skipping && skipToTrack(track.track_number)}
                        style={{ cursor: isCurrent || skipping ? "default" : "pointer" }}
                      >
                        <div className="tracklist-num">
                          {isCurrent ? (
                            playbackStatus === "Playing" ? (
                              <svg width="12" height="12" viewBox="0 0 16 16" fill="var(--accent)">
                                <rect x="1" y="4" width="3" height="8" rx="0.5"><animate attributeName="height" values="8;3;8" dur="0.8s" repeatCount="indefinite" /><animate attributeName="y" values="4;8.5;4" dur="0.8s" repeatCount="indefinite" /></rect>
                                <rect x="6.5" y="2" width="3" height="12" rx="0.5"><animate attributeName="height" values="12;5;12" dur="0.6s" repeatCount="indefinite" /><animate attributeName="y" values="2;7.5;2" dur="0.6s" repeatCount="indefinite" /></rect>
                                <rect x="12" y="5" width="3" height="6" rx="0.5"><animate attributeName="height" values="6;10;6" dur="0.7s" repeatCount="indefinite" /><animate attributeName="y" values="5;3;5" dur="0.7s" repeatCount="indefinite" /></rect>
                              </svg>
                            ) : (
                              <svg width="12" height="12" viewBox="0 0 16 16" fill="var(--accent)"><path d="M2.7 1a.7.7 0 0 0-.7.7v12.6a.7.7 0 0 0 .7.7h2.6a.7.7 0 0 0 .7-.7V1.7a.7.7 0 0 0-.7-.7H2.7zm8 0a.7.7 0 0 0-.7.7v12.6a.7.7 0 0 0 .7.7h2.6a.7.7 0 0 0 .7-.7V1.7a.7.7 0 0 0-.7-.7h-2.6z"/></svg>
                            )
                          ) : (
                            <span>{track.track_number}</span>
                          )}
                        </div>
                        <div className="tracklist-info">
                          <div className={`tracklist-name ${isCurrent ? "tracklist-name-active" : ""}`}>
                            {track.name}
                          </div>
                          {track.is_explicit && <span className="tracklist-explicit">E</span>}
                        </div>
                        <div className="tracklist-duration">{formatTime(track.duration_secs)}</div>
                      </div>
                    );
                  })}
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    );
  }

  /* ═══════════════════════════════════════════════════
     APP MODE — card layout
     ═══════════════════════════════════════════════════ */
  return (
    <div style={{ padding: 24, display: "flex", flexDirection: "column", gap: 16 }}>
      <div>
        <h1 style={{ fontSize: 22, fontWeight: 700, letterSpacing: "-0.02em", color: "var(--text-1)" }}>
          {t("dashboard.title")}
        </h1>
        <p style={{ fontSize: 13, color: "var(--text-2)", marginTop: 2 }}>
          {t("dashboard.subtitle")}
        </p>
      </div>

      <div className="card" style={{ padding: 16 }}>
        <div className="section-label" style={{ marginBottom: 12 }}>
          {t("dashboard.currentActivity")}
        </div>

        {running && currentApp ? (
          <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
            <div style={{ display: "flex", alignItems: "flex-start", gap: 14 }}>
              <div
                className={`icon-editable${connected ? "" : " breathe-red"}`}
                onClick={() => setShowIconPicker(true)}
                style={{
                  width: 48, height: 48, borderRadius: 12,
                  display: "flex", alignItems: "center", justifyContent: "center",
                  background: hasIcon ? "var(--bg-input)" : "var(--accent-bg)",
                  flexShrink: 0, border: "1px solid var(--border-light)",
                  cursor: "pointer", position: "relative",
                }}
              >
                {hasIcon ? (
                  <img
                    src={largeImageKey!}
                    alt={currentApp}
                    style={{ width: 34, height: 34, objectFit: "contain" }}
                    onError={(e) => {
                      (e.target as HTMLImageElement).style.display = "none";
                      (e.target as HTMLImageElement).parentElement!.innerHTML = `<span style="font-size:24px">${emoji}</span>`;
                    }}
                  />
                ) : (
                  <span style={{ fontSize: 24 }}>{emoji}</span>
                )}
                <div className="icon-edit-badge">
                  <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="white" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M17 3a2.83 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z" />
                  </svg>
                </div>
                <div className="icon-edit-overlay">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="white" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
                    <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
                  </svg>
                </div>
              </div>

              <div style={{ minWidth: 0, flex: 1, paddingTop: 2 }}>
                <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                  <span style={{ fontSize: 15, fontWeight: 600, color: "var(--text-1)" }}>
                    {currentApp}
                  </span>
                  {category && (
                    <span className="badge badge-gray" style={{ textTransform: "capitalize" }}>
                      {t(`categories.${category}`, category)}
                    </span>
                  )}
                </div>
                {details && (
                  <div style={{ fontSize: 13, color: "var(--text-2)", marginTop: 2 }}>{details}</div>
                )}
                {rpcState && (
                  <div style={{ fontSize: 12, color: "var(--text-3)" }}>{rpcState}</div>
                )}
                {elapsed && (
                  <div style={{ display: "flex", alignItems: "center", gap: 6, marginTop: 6, fontSize: 12, color: "var(--text-2)" }}>
                    <span className="pulse-dot" style={{
                      display: "inline-block", width: 6, height: 6, borderRadius: "50%",
                      background: "var(--green)",
                    }} />
                    {elapsed} {t("dashboard.elapsed")}
                  </div>
                )}
              </div>
            </div>

            {(exeName || windowTitle) && (
              <div style={{ display: "flex", alignItems: "center", gap: 8, marginLeft: 62 }}>
                {exeName && (
                  <span style={{
                    fontSize: 10, fontFamily: "monospace", padding: "2px 6px",
                    borderRadius: 4, background: "var(--bg-input)", color: "var(--text-3)",
                  }}>
                    {exeName}
                  </span>
                )}
                {windowTitle && (
                  <span style={{
                    fontSize: 10, color: "var(--text-3)",
                    overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", maxWidth: 200,
                  }} title={windowTitle}>
                    {windowTitle}
                  </span>
                )}
              </div>
            )}
          </div>
        ) : (
          <div style={{ textAlign: "center", padding: "24px 0", color: "var(--text-3)", fontSize: 13 }}>
            {running ? t("dashboard.waitingForFocus") : t("dashboard.startServicePrompt")}
          </div>
        )}
      </div>

      <div style={{ textAlign: "center", fontSize: 11, color: "var(--text-3)", lineHeight: 1.5 }}>
        {t("dashboard.footer1")}<br />
        {t("dashboard.footer2")}
      </div>

      {showIconPicker && exeName && currentApp && (
        <IconPicker
          exeName={exeName}
          appName={currentApp}
          onClose={() => setShowIconPicker(false)}
          onPicked={(url) => setLargeImageKey(url)}
        />
      )}
    </div>
  );
}
