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

export interface RpcConfig {
  apps: Record<string, AppConfig>;
  default: DefaultConfig;
  discord_app_id: string;
  poll_interval_ms: number;
  steamgriddb_api_key: string;
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
}

export interface DetectedApp {
  exe_name: string;
  name: string;
  category: string;
  details: string;
  last_seen: number;
  is_builtin: boolean;
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
