import { invoke } from "@tauri-apps/api/core";

export interface ButtonConfig {
  label: string;
  url: string;
}

export interface AppConfig {
  name: string;
  details: string;
  state: string;
  large_image_key: string | null;
  large_image_text: string | null;
  small_image_key: string | null;
  small_image_text: string | null;
  category: string;
  show_timestamp: boolean;
  buttons: ButtonConfig[];
}

export interface DefaultConfig {
  details: string;
  state: string;
  large_image_key: string;
}

export interface MusicConfig {
  activity_type: string;
  display_text: string;
  display_text_custom: string;
  profile_text: string;
  profile_text_custom: string;
  title_artist_one_line: boolean;
  artist_album_one_line: boolean;
  reverse_title_artist: boolean;
  prefix_artist_by: boolean;
  prefix_album_on: boolean;
  show_album: boolean;
  show_album_when_no_artist: boolean;
  show_playback_info: boolean;
  hide_music_info: boolean;
  show_paused: boolean;
  show_pause_icon: boolean;
  freeze_progress_when_paused: boolean;
  show_paused_duration: boolean;
  show_play_icon: boolean;
  show_player_logo: boolean;
  show_get_status_button: boolean;
  show_listen_button: boolean;
  fallback_cover: string;
}

export interface RpcConfig {
  apps: Record<string, AppConfig>;
  default: DefaultConfig;
  discord_app_id: string;
  poll_interval_ms: number;
  steamgriddb_api_key: string;
  music_enabled: boolean;
  music_config: MusicConfig;
  auto_start_service: boolean;
  hide_tray_icon: boolean;
  language: string;
}

export interface RpcStatus {
  connected: boolean;
  current_app: string | null;
}

export interface ForegroundApp {
  exe_name: string;
  window_title: string;
  exe_path: string;
}

export interface RpcStatusEvent {
  connected: boolean;
  current_app: string | null;
  exe_name: string | null;
  window_title: string | null;
  category: string | null;
  details: string | null;
  state: string | null;
  large_image_key: string | null;
  start_timestamp: number | null;
  is_music: boolean;
  small_image_key: string | null;
  album_name: string | null;
  listen_url: string | null;
  playback_status: string | null;
  position_secs: number | null;
  duration_secs: number | null;
}

export interface MediaInfo {
  title: string;
  artist: string;
  album: string;
  player_name: string;
  playback_status: string;
  start_time_unix: number | null;
  is_browser: boolean;
}

export interface DetectedApp {
  exe_name: string;
  name: string;
  category: string;
  details: string;
  last_seen: number;
  is_builtin: boolean;
  large_image_key: string | null;
}

export const loadConfig = () => invoke<RpcConfig>("load_config");
export const saveConfig = (config: RpcConfig) =>
  invoke<void>("save_config", { config });

export const getActiveWindow = () =>
  invoke<ForegroundApp | null>("get_active_window");

export const startPoller = () => invoke<void>("start_poller");
export const stopPoller = () => invoke<void>("stop_poller");
export const isPollerRunning = () => invoke<boolean>("is_poller_running");

export const getRpcStatus = () => invoke<RpcStatus>("get_rpc_status");
export const disconnectRpc = () => invoke<void>("disconnect_rpc");

export const enableAutoStartup = () => invoke<void>("enable_auto_startup");
export const disableAutoStartup = () => invoke<void>("disable_auto_startup");
export const isAutoStartupEnabled = () =>
  invoke<boolean>("is_auto_startup_enabled");

export const getBuiltinAppsList = () =>
  invoke<Record<string, AppConfig>>("get_builtin_apps_list");

export const getDetectedApps = () =>
  invoke<DetectedApp[]>("get_detected_apps");

export interface IconCandidate {
  icon_id: string;
  url: string;
  set_name: string;
}

export const searchIcons = (query: string) =>
  invoke<IconCandidate[]>("search_icons", { query });

export const setAppIcon = (exeName: string, iconUrl: string) =>
  invoke<void>("set_app_icon", { exeName, iconUrl });

export const getCurrentMedia = () =>
  invoke<MediaInfo | null>("get_current_media_cmd");

export const mediaPlayPause = () => invoke<void>("media_play_pause");
export const mediaNext = () => invoke<void>("media_next");
export const mediaPrevious = () => invoke<void>("media_previous");
export const mediaSeek = (positionSecs: number) => invoke<void>("media_seek", { positionSecs });
export const getSystemVolume = (exeName?: string) => invoke<[number, boolean]>("get_system_volume", { exeName });
export const setSystemVolume = (level: number, exeName?: string) => invoke<void>("set_system_volume", { level, exeName });
export const toggleMute = (exeName?: string) => invoke<boolean>("toggle_mute", { exeName });

export const validateSpotify = (clientId: string, clientSecret: string) =>
  invoke<boolean>("validate_spotify", { clientId, clientSecret });

export const openUrl = (url: string) => invoke<void>("open_url", { url });

export interface TrackInfo {
  name: string;
  duration_secs: number;
  track_number: number;
  is_explicit: boolean;
}

export interface AlbumInfo {
  album_name: string;
  artist_name: string;
  album_art_url: string;
  year: string;
  total_tracks: number;
  total_duration_secs: number;
  tracks: TrackInfo[];
  deezer_url: string;
}

export const getAlbumTracks = (artist: string, title: string) =>
  invoke<AlbumInfo | null>("get_album_tracks_cmd", { artist, title });

export interface UpdateInfo {
  current_version: string;
  latest_version: string;
  has_update: boolean;
  release_url: string;
  release_notes: string | null;
  download_url: string | null;
  file_name: string | null;
  file_size: number | null;
  file_sha256: string | null;
  install_type: string;
}

export const checkForUpdates = () =>
  invoke<UpdateInfo>("check_for_updates");

export const startSilentUpdate = (downloadUrl: string, fileName?: string, fileSha256?: string) =>
  invoke<void>("start_silent_update", { downloadUrl, fileName, fileSha256 });

export const relaunchApp = () => invoke<void>("relaunch_app");
export const setTrayLanguage = (lang: string) => invoke<void>("set_tray_language", { lang });
