import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  HistoryItem,
  InspectionResult,
  LogLine,
  OsInfo,
  PermissionInfo,
  Settings,
} from "../store/app";

export interface RemoteConfig {
  host: string;
  port: number;
  username: string;
  auth: string; // "password" | "key"
  password: string;
  key_path: string;
}

export interface InspectionRequest {
  target_system: string; // windows | linux | macos
  mode: string; // local | remote
  modules: string[];
  fast: boolean;
  remote: RemoteConfig;
  output_dir: string;
}

export const startInspection = (req: InspectionRequest) =>
  invoke<string>("start_inspection", { req });

export const stopInspection = () => invoke<string>("stop_inspection");

export const readReport = (path: string) => invoke<string>("read_report", { path });

export const exportReport = (src: string, format: string, destDir: string) =>
  invoke<string>("export_report", { src, format, destDir });

export const getHistory = () => invoke<HistoryItem[]>("get_history");

export const deleteHistoryItem = (id: string) =>
  invoke<void>("delete_history_item", { id });

export const getSettings = () => invoke<Settings>("get_settings");

export const saveSettings = (settings: Settings) =>
  invoke<void>("save_settings", { settings });

export const getOsInfo = () => invoke<OsInfo>("get_os_info");

export const checkPermission = () => invoke<PermissionInfo>("check_permission");

export const pickDirectory = () => invoke<string | null>("pick_directory");

export const openPath = (path: string) => invoke<void>("open_path", { path });

export const getAppVersion = () => invoke<string>("get_app_version");

export const onInspectionLog = (cb: (line: LogLine) => void): Promise<UnlistenFn> =>
  listen<LogLine>("inspection-log", (e) => cb(e.payload));

export const onInspectionDone = (
  cb: (result: InspectionResult) => void
): Promise<UnlistenFn> => listen<InspectionResult>("inspection-done", (e) => cb(e.payload));
