import { invoke } from "@tauri-apps/api/core";

// Type-safe wrapper around Tauri invoke for all backend commands.
// Each function maps to a #[tauri::command] in the Rust backend.

// ── Auth ──────────────────────────────────────────────────────
export interface DeviceCodeResponse {
  device_code: string;
  user_code: string;
  verification_uri: string;
  expires_in: number;
  interval: number;
  message: string;
}

export interface Account {
  uuid: string;
  username: string;
  access_token: string;
  refresh_token: string;
  skin_url: string | null;
}

export async function loginStart(): Promise<DeviceCodeResponse> {
  return invoke("login_start");
}

export async function loginPoll(deviceCode: string): Promise<Account> {
  return invoke("login_poll", { deviceCode });
}

export async function getProfile() {
  return invoke("get_profile");
}

export async function logout(uuid: string) {
  return invoke("logout", { uuid });
}

export async function listAccounts(): Promise<Account[]> {
  return invoke("list_accounts");
}

// ── Instances ─────────────────────────────────────────────────
export interface Instance {
  id: string;
  name: string;
  game_version: string;
  mod_loader: string | null;
  mod_loader_version: string | null;
  icon: string | null;
  created_at: number;
  last_played: number | null;
  play_time_seconds: number;
  source: string | null;
  source_id: string | null;
}

export interface CreateInstanceParams {
  name: string;
  game_version: string;
  mod_loader?: string;
  mod_loader_version?: string;
  icon?: string;
}

export async function listInstances(): Promise<Instance[]> {
  return invoke("list_instances");
}

export async function createInstance(
  params: CreateInstanceParams,
): Promise<Instance> {
  return invoke("create_instance", { params });
}

export async function deleteInstance(id: string) {
  return invoke("delete_instance", { id });
}

export async function updateInstance(id: string, name?: string, icon?: string) {
  return invoke("update_instance", { id, name, icon });
}

// ── Minecraft ─────────────────────────────────────────────────
export interface VersionManifestEntry {
  id: string;
  type: string;
  url: string;
  time: string;
  releaseTime: string;
}

export interface VersionManifest {
  latest: { release: string; snapshot: string };
  versions: VersionManifestEntry[];
}

export async function getVersionManifest(): Promise<VersionManifest> {
  return invoke("get_version_manifest");
}

export async function getVersionDetails(versionId: string) {
  return invoke("get_version_details", { versionId });
}

export async function downloadVersion(
  instanceId: string,
  versionId: string,
) {
  return invoke("download_version", { instanceId, versionId });
}
