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

export async function switchActiveAccount(uuid: string): Promise<AccountInfo> {
  return invoke("switch_active_account", { uuid });
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

export async function launchGameOffline(
  instanceId: string,
  username: string
): Promise<number> {
  return invoke("launch_game_offline", { instanceId, username });
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

export async function installModFromCurseforge(
  instanceId: string,
  modId: string,
  gameVersion: string,
  loader: string
): Promise<InstalledModInfo> {
  return invoke("install_mod_from_curseforge", {
    instanceId,
    modId,
    gameVersion,
    loader,
  });
}

/** Unified install — routes to Modrinth or CurseForge based on source. */
export async function installMod(
  instanceId: string,
  source: string,
  projectId: string,
  gameVersion: string,
  loader: string
): Promise<InstalledModInfo> {
  return invoke("install_mod", {
    instanceId,
    source,
    projectId,
    gameVersion,
    loader,
  });
}

export interface ModVersionInfo {
  version_id: string;
  name: string;
  version_number: string;
  date_published: string;
  download_count: number;
  file_name: string | null;
  file_url: string | null;
}

export async function getModrinthVersions(
  projectId: string,
  gameVersion: string,
  loader: string
): Promise<ModVersionInfo[]> {
  return invoke("get_modrinth_versions", { projectId, gameVersion, loader });
}

export async function getCurseforgeVersions(
  modId: string,
  gameVersion: string,
  loader: string
): Promise<ModVersionInfo[]> {
  return invoke("get_curseforge_versions", { modId, gameVersion, loader });
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

// ── Modpack Browsing + One-Click Install ──────────────────────

export interface ModpackSearchResult {
  source: string;
  project_id: string;
  slug: string;
  title: string;
  description: string;
  icon_url: string;
  downloads: number;
  categories: string[];
  game_versions: string[];
}

export interface ModVersionInfo {
  version_id: string;
  name: string;
  version_number: string;
  date_published: string;
  download_count: number;
  file_name: string | null;
  file_url: string | null;
  game_versions: string[];
}

export async function searchModpacksModrinth(
  query: string,
  offset: number = 0,
  limit: number = 20
): Promise<ModpackSearchResult[]> {
  return invoke("search_modpacks_modrinth", { query, offset, limit });
}

export async function searchModpacksCurseforge(
  query: string,
  offset: number = 0,
  limit: number = 20
): Promise<ModpackSearchResult[]> {
  return invoke("search_modpacks_curseforge", { query, offset, limit });
}

export async function getModpackVersionsModrinth(
  projectId: string
): Promise<ModVersionInfo[]> {
  return invoke("get_modpack_versions_modrinth", { projectId });
}

export async function downloadAndInstallModpack(
  downloadUrl: string,
  source: string,
  name: string
): Promise<InstanceListItem> {
  return invoke("download_and_install_modpack", { downloadUrl, source, name });
}

// ── Mod Update Checker ───────────────────────────────────────

export interface ModUpdateInfo {
  mod_id: string;
  mod_name: string;
  source: string;
  installed_version: string;
  latest_version: string;
  latest_file_url: string | null;
  update_available: boolean;
}

export async function checkModUpdates(
  instanceId: string
): Promise<ModUpdateInfo[]> {
  return invoke("check_mod_updates", { instanceId });
}

// ── Resource Packs & Shaders ─────────────────────────────────

export interface InstalledPackInfo {
  file_name: string;
  enabled: boolean;
}

export async function listInstalledPacks(
  instanceId: string,
  packType: "resourcepacks" | "shaderpacks"
): Promise<InstalledPackInfo[]> {
  return invoke("list_installed_packs", { instanceId, packType });
}

export async function togglePack(
  instanceId: string,
  packType: "resourcepacks" | "shaderpacks",
  fileName: string
): Promise<boolean> {
  return invoke("toggle_pack", { instanceId, packType, fileName });
}

export async function deletePack(
  instanceId: string,
  packType: "resourcepacks" | "shaderpacks",
  fileName: string
): Promise<void> {
  return invoke("delete_pack", { instanceId, packType, fileName });
}

// ── Import from Other Launchers ───────────────────────────────

export interface ImportableInstance {
  name: string;
  game_version: string;
  loader: string;
  loader_version: string | null;
  source_path: string;
  source_launcher: string;
  icon: string | null;
}

export async function scanLauncherInstances(
  launcherType: string,
  basePath?: string
): Promise<ImportableInstance[]> {
  return invoke("scan_launcher_instances", {
    launcherType,
    basePath: basePath ?? null,
  });
}

export async function importLauncherInstance(
  launcherType: string,
  sourcePath: string,
  name: string,
  gameVersion: string,
  loader: string,
  loaderVersion?: string
): Promise<InstanceListItem> {
  return invoke("import_launcher_instance", {
    launcherType,
    sourcePath,
    name,
    gameVersion,
    loader,
    loaderVersion: loaderVersion ?? null,
  });
}

// ── Desktop Shortcuts ─────────────────────────────────────────

export interface ShortcutResult {
  path: string;
  success: boolean;
  error: string | null;
}

export async function createDesktopShortcut(
  instanceId: string,
  instanceName: string,
  outputDir?: string,
  serverAddress?: string
): Promise<ShortcutResult> {
  return invoke("create_desktop_shortcut", {
    instanceId,
    instanceName,
    outputDir: outputDir ?? null,
    serverAddress: serverAddress ?? null,
  });
}

export async function getShortcutDefaultDir(): Promise<string> {
  return invoke("get_shortcut_default_dir");
}

// ── Worlds & Servers ──────────────────────────────────────────

export interface ServerEntry {
  name: string;
  address: string;
  icon: string | null;
  is_hidden: boolean;
  index: number;
}

export interface SingleplayerWorld {
  name: string;
  folder_name: string;
  game_mode: string;
  last_played: string | null;
  size_bytes: number;
  icon: string | null;
  seed: string | null;
}

export interface WorldsInfo {
  servers: ServerEntry[];
  singleplayer: SingleplayerWorld[];
}

export interface ServerStatus {
  online: boolean;
  players_online: number | null;
  players_max: number | null;
  version: string | null;
  motd: string | null;
  latency_ms: number;
}

export async function getInstanceWorlds(
  instanceId: string
): Promise<WorldsInfo> {
  return invoke("get_instance_worlds", { instanceId });
}

export async function addServer(
  instanceId: string,
  name: string,
  address: string
): Promise<ServerEntry> {
  return invoke("add_server", { instanceId, name, address });
}

export async function editServer(
  instanceId: string,
  index: number,
  name: string,
  address: string
): Promise<void> {
  return invoke("edit_server", { instanceId, index, name, address });
}

export async function removeServer(
  instanceId: string,
  index: number
): Promise<void> {
  return invoke("remove_server", { instanceId, index });
}

export async function pingServer(address: string): Promise<ServerStatus> {
  return invoke("ping_server", { address });
}

export async function deleteWorld(
  instanceId: string,
  folderName: string
): Promise<void> {
  return invoke("delete_world", { instanceId, folderName });
}

export async function renameWorld(
  instanceId: string,
  folderName: string,
  newName: string
): Promise<void> {
  return invoke("rename_world", { instanceId, folderName, newName });
}

export async function backupWorld(
  instanceId: string,
  folderName: string
): Promise<string> {
  return invoke("backup_world", { instanceId, folderName });
}

// ── Skins ─────────────────────────────────────────────────────

export interface SkinInfo {
  texture_url: string | null;
  variant: string;
  cape_url: string | null;
}

export interface CapeInfo {
  id: string;
  name: string;
  url: string;
  state: string;
}

export async function getSkinInfo(accountUuid: string): Promise<SkinInfo> {
  return invoke("get_skin_info", { accountUuid });
}

export async function uploadSkin(
  accountUuid: string,
  skinData: number[],
  variant: string
): Promise<SkinInfo> {
  return invoke("upload_skin", { accountUuid, skinData, variant });
}

export async function resetSkin(accountUuid: string): Promise<void> {
  return invoke("reset_skin", { accountUuid });
}

export async function getCapes(accountUuid: string): Promise<CapeInfo[]> {
  return invoke("get_capes", { accountUuid });
}

// ── Instance Hooks ────────────────────────────────────────────

export interface InstanceHooks {
  pre_launch_cmd: string | null;
  post_exit_cmd: string | null;
  hook_env_vars: string | null;
}

export async function getInstanceHooks(
  instanceId: string
): Promise<InstanceHooks> {
  return invoke("get_instance_hooks", { instanceId });
}

export async function updateInstanceHooks(
  instanceId: string,
  hooks: InstanceHooks
): Promise<void> {
  return invoke("update_instance_hooks", { instanceId, hooks });
}

// ── Advanced Logs ─────────────────────────────────────────────

export interface LogFileInfo {
  filename: string;
  size_bytes: number;
  modified: string;
  log_type: string;
}

export interface LogCursor {
  content: string;
  new_cursor: number;
  has_more: boolean;
}

export async function getLogFiles(
  instanceId: string
): Promise<LogFileInfo[]> {
  return invoke("get_log_files", { instanceId });
}

export async function readLogCursor(
  instanceId: string,
  filename: string,
  cursor: number,
  maxBytes?: number
): Promise<LogCursor> {
  return invoke("read_log_cursor", {
    instanceId,
    filename,
    cursor,
    maxBytes: maxBytes ?? null,
  });
}

export async function readLogFile(
  instanceId: string,
  filename: string
): Promise<string> {
  return invoke("read_log_file", { instanceId, filename });
}

export async function deleteLogFile(
  instanceId: string,
  filename: string
): Promise<void> {
  return invoke("delete_log_file", { instanceId, filename });
}

export async function deleteAllLogs(instanceId: string): Promise<number> {
  return invoke("delete_all_logs", { instanceId });
}

export async function getLogSize(
  instanceId: string,
  filename: string
): Promise<number> {
  return invoke("get_log_size", { instanceId, filename });
}

