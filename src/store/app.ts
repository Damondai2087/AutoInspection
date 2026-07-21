import { reactive } from "vue";

export interface LogLine {
  level: "info" | "error" | "warn";
  text: string;
  ts: string;
}

export interface HistoryItem {
  id: string;
  timestamp: string;
  target_system: string;
  mode: string;
  target: string;
  report_path: string;
  status: string;
  summary: string;
}

export interface Settings {
  output_dir: string;
  theme: string;
  custom_script_dir: string;
}

export interface OsInfo {
  platform: string;
  name: string;
}

export interface PermissionInfo {
  is_elevated: boolean;
  method: string;
  message: string;
}

export interface InspectionResult {
  success: boolean;
  report_path: string | null;
  report_format: string;
  message: string;
  status: string;
}

interface AppStore {
  activeNav: string;
  osInfo: OsInfo;
  permission: PermissionInfo;
  settings: Settings;
  running: boolean;
  logs: LogLine[];
  report: { path: string; html: string; loaded: boolean };
  history: HistoryItem[];
}

export const store = reactive<AppStore>({
  activeNav: "local",
  osInfo: { platform: "", name: "" },
  permission: { is_elevated: false, method: "", message: "" },
  settings: { output_dir: "", theme: "light", custom_script_dir: "" },
  running: false,
  logs: [],
  report: { path: "", html: "", loaded: false },
  history: [],
});

export function resetLogs() {
  store.logs = [];
}

export function pushLog(line: LogLine) {
  store.logs.push(line);
  if (store.logs.length > 2000) {
    store.logs.splice(0, store.logs.length - 2000);
  }
}

export function clearReport() {
  store.report = { path: "", html: "", loaded: false };
}
