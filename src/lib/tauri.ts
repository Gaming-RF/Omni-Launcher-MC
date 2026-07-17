import { invoke } from "@tauri-apps/api/core";

// ── Auth ──────────────────────────────────────────────────────

export interface DeviceCodeInfo {
  user_code: string;
  verification_uri: string;
  message: string;
}

export interface AccountInfo {
  uuid: string;
  username: string;
  skin_url: string | null;
}

export async function startLogin(): Promise<DeviceCodeInfo> {
  return invoke("start_login");
}

export async function pollLogin(): Promise<AccountInfo> {
  return invoke("poll_login");
}

export async function getAccounts(): Promise<AccountInfo[]> {
  return invoke("get_accounts");
}

export async function removeAccount(uuid: string): Promise<void> {
  return invoke("remove_account", { uuid });
}

// ── Instances ─────────────────────────────────────────────────

export interface InstanceListItem {
  id: string;
  name: string;
  game_version: string;
  loader: string;
  loader_version: string | null;
  icon: string | null;
  created_at: string;
  last_played: string | null;
  play_time_secs: number;
  allocated_memory_mb: number;
}

export interface CreateInstancePayload {
  name: string;
  game_version: string;
  loader: string;
  loader_version: string | null;
  icon: string | null;
  java_args: string | null;
  allocated_memory_mb: number;
}

export async function getInstances(): Promise<InstanceListItem[]> {
  return invoke("get_instances");
}

export async function createInstance(
  payload: CreateInstancePayload
): Promise<InstanceListItem> {
  return invoke("create_instance", { payload });
}

export async function deleteInstance(id: string): Promise<void> {
  return invoke("delete_instance", { id });
}

export async function updateInstance(
  id: string,
  name?: string,
  javaArgs?: string,
  allocatedMemoryMb?: number
): Promise<void> {
  return invoke("update_instance", {
    id,
    name: name ?? null,
    javaArgs: javaArgs ?? null,
    allocatedMemoryMb: allocatedMemoryMb ?? null,
  });
}

// ── Minecraft ─────────────────────────────────────────────────

export interface VersionEntry {
  id: string;
  version_type: string;
  release_time: string;
}

export interface JavaInfo {
  found: boolean;
  path: string | null;
  error: string | null;
}

export async function getVersionManifest(): Promise<VersionEntry[]> {
  return invoke("get_version_manifest");
}

export async function prepareInstance(instanceId: string): Promise<string> {
  return invoke("prepare_instance", { instanceId });
}

export async function launchGame(instanceId: string): Promise<number> {
  return invoke("launch_game", { instanceId });
}

export async function checkJava(): Promise<JavaInfo> {
  return invoke("check_java");
}

// ── Mod Search ────────────────────────────────────────────────

export interface ModSearchResult {
  source: string;
  project_id: string;
  slug: string;
  title: string;
  description: string;
  icon_url: string;
  downloads: number;
  categories: string[];
}

export async function modrinthSearch(
  query: string,
  offset?: number,
  limit?: number
): Promise<ModSearchResult[]> {
  return invoke("modrinth_search", {
    query,
    offset: offset ?? 0,
    limit: limit ?? 20,
  });
}

export async function curseforgeSearch(
  query: string,
  offset?: number,
  limit?: number
): Promise<ModSearchResult[]> {
  return invoke("curseforge_search", {
    query,
    offset: offset ?? 0,
    limit: limit ?? 20,
  });
}

// ── Settings ──────────────────────────────────────────────────

export interface SettingsInfo {
  default_memory_mb: string;
  theme: string;
  language: string;
  java_path: string | null;
  curseforge_api_key: string | null;
}

export async function getSettings(): Promise<SettingsInfo> {
  return invoke("get_settings");
}

export async function updateSetting(key: string, value: string): Promise<void> {
  return invoke("update_setting", { key, value });
}

// ── Mod Loader Versions ───────────────────────────────────────

export interface LoaderVersionInfo {
  version: string;
  stable: boolean;
}

export async function getFabricLoaderVersions(
  mcVersion: string
): Promise<LoaderVersionInfo[]> {
  return invoke("get_fabric_loader_versions", { mcVersion });
}

export async function getQuiltLoaderVersions(
  mcVersion: string
): Promise<LoaderVersionInfo[]> {
  return invoke("get_quilt_loader_versions", { mcVersion });
}

export async function getForgeVersions(mcVersion: string): Promise<string[]> {
  return invoke("get_forge_versions", { mcVersion });
}

export async function getNeoForgeVersions(mcVersion: string): Promise<string[]> {
  return invoke("get_neoforge_versions", { mcVersion });
}

// ── Loader Installation ───────────────────────────────────────

export async function installFabricLoader(
  instanceId: string,
  loaderVersion: string
): Promise<string> {
  return invoke("install_fabric_loader", { instanceId, loaderVersion });
}

export async function installQuiltLoader(
  instanceId: string,
  loaderVersion: string
): Promise<string> {
  return invoke("install_quilt_loader", { instanceId, loaderVersion });
}

export async function installForgeLoader(
  instanceId: string,
  forgeVersion: string
): Promise<string> {
  return invoke("install_forge_loader", { instanceId, forgeVersion });
}

export async function installNeoForgeLoader(
  instanceId: string,
  neoforgeVersion: string
): Promise<string> {
  return invoke("install_neoforge_loader", { instanceId, neoforgeVersion });
}

// ── Per-Instance Mod Management ───────────────────────────────

export interface InstalledModInfo {
  id: number;
  mod_id: string;
  source: string;
  name: string;
  version: string;
  file_name: string;
  enabled: boolean;
  installed_at: string;
}

export async function getInstanceMods(
  instanceId: string
): Promise<InstalledModInfo[]> {
  return invoke("get_instance_mods", { instanceId });
}

export async function installModFromModrinth(
  instanceId: string,
  projectId: string,
  gameVersion: string,
  loader: string
): Promise<InstalledModInfo> {
  return invoke("install_mod_from_modrinth", {
    instanceId,
    projectId,
    gameVersion,
    loader,
  });
}

export async function toggleModEnabled(
  modId: number,
  instanceId: string
): Promise<boolean> {
  return invoke("toggle_mod_enabled", { modId, instanceId });
}

export async function removeMod(
  modId: number,
  instanceId: string
): Promise<void> {
  return invoke("remove_mod", { modId, instanceId });
}

// ── Modpack Import ────────────────────────────────────────────

export interface ModpackInfo {
  name: string;
  version: string;
  summary: string | null;
  game_version: string;
  loader: string;
  loader_version: string;
  file_count: number;
}

export async function parseMrpackFile(filePath: string): Promise<ModpackInfo> {
  return invoke("parse_mrpack_file", { filePath });
}

export async function parseCfModpackFile(filePath: string): Promise<ModpackInfo> {
  return invoke("parse_cf_modpack_file", { filePath });
}

export async function installMrpackModpack(
  filePath: string,
  instanceName: string
): Promise<InstanceListItem> {
  return invoke("install_mrpack_modpack", { filePath, instanceName });
}

// ── Java Management ───────────────────────────────────────────

export interface JavaCheckResult {
  found: boolean;
  path: string | null;
  major_version: number;
  auto_downloaded: boolean;
  error: string | null;
}

export async function getRequiredJavaVersion(mcVersion: string): Promise<number> {
  return invoke("get_required_java_version", { mcVersion });
}

export async function ensureJavaForMc(
  mcVersion: string,
  customPath?: string
): Promise<JavaCheckResult> {
  return invoke("ensure_java_for_mc", {
    mcVersion,
    customPath: customPath ?? null,
  });
}

export async function downloadJavaVersion(javaMajor: number): Promise<string> {
  return invoke("download_java_version", { javaMajor });
}

// ── Instance Sharing ──────────────────────────────────────────

export interface ShareCode {
  code: string;
  name: string;
  mod_count: number;
}

export async function exportInstanceShare(instanceId: string): Promise<ShareCode> {
  return invoke("export_instance_share", { instanceId });
}

export async function importInstanceShare(code: string): Promise<InstanceListItem> {
  return invoke("import_instance_share", { payload: { code } });
}

// ── Game Process ───────────────────────────────────────────────

export async function getGameLogs(instanceId: string): Promise<string[]> {
  return invoke("get_game_logs", { instanceId });
}

export async function getRunningInstances(): Promise<string[]> {
  return invoke("get_running_instances");
}

export async function isInstanceRunning(id: string): Promise<boolean> {
  return invoke("is_instance_running", { id });
}

export async function killGame(id: string): Promise<void> {
  return invoke("kill_game", { id });
}

// ── Instance Duplication ─────────────────────────────────────

export async function duplicateInstance(
  instanceId: string,
  newName: string
): Promise<InstanceListItem> {
  return invoke("duplicate_instance", { instanceId, newName });
}

// ── Aggregated Search ─────────────────────────────────────────

export interface AggregatedSearchResult {
  source: string;
  project_id: string;
  slug: string;
  title: string;
  description: string;
  icon_url: string;
  downloads: number;
  categories: string[];
}

export async function aggregatedSearch(
  query: string,
  offset?: number,
  limit?: number
): Promise<AggregatedSearchResult[]> {
  return invoke("aggregated_search", {
    query,
    offset: offset ?? 0,
    limit: limit ?? 20,
  });
}

