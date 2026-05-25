/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { useEffect, useMemo, useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { GamesTable } from "@/features/game-library/ui/games-table";
import {
  addManualGame,
  clearFailedDownloads,
  clearFinishedDownloads,
  getAppPaths,
  getAppSettings,
  getPlatform,
  getStagingDir,
  installTheme,
  listSupportedProfiles,
  listThemes,
  openPath,
  openThemesFolder,
  removeManualGame,
  scanGames,
  updateManualGameNexusDomain,
  updateAppSettings,
  validateNexusApiKey,
} from "@/lib/tauri";
import { checkForUpdates, installUpdate } from "@/lib/api/updates";
import type { Update } from "@tauri-apps/plugin-updater";
import { mergeSettings } from "@/lib/settings-defaults";
import { isTauri } from "@/lib/env";
import { listenSettingsChanged } from "@/lib/settings-events";
import { useThemeLayout } from "@/features/themes/theme-provider";
import { formatInvokeError } from "@/lib/errors";
import type {
  AppPathsInfo,
  AppSettings,
  DetectedGame,
  GameProfileSummary,
  ThemeSummary,
  UpdateCheckMode,
} from "@/types";

type Props = {
  onBack?: () => void;
  embedded?: boolean;
  selectedGame?: DetectedGame | null;
  games?: DetectedGame[];
  gamesLoading?: boolean;
  onSelectGame?: (game: DetectedGame) => void;
  onGamesLoaded?: (games: DetectedGame[]) => void;
  nxmNotice?: string | null;
  onClearNxmNotice?: () => void;
};

export function SettingsPage({
  onBack,
  embedded,
  selectedGame,
  games = [],
  gamesLoading = false,
  onSelectGame,
  onGamesLoaded,
  nxmNotice,
  onClearNxmNotice,
}: Props) {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [paths, setPaths] = useState<AppPathsInfo | null>(null);
  const [profiles, setProfiles] = useState<GameProfileSummary[]>([]);
  const [gameStaging, setGameStaging] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [apiKeyDraft, setApiKeyDraft] = useState("");
  const [lootDraft, setLootDraft] = useState("");
  const [modEngineDraft, setModEngineDraft] = useState("");
  const [stagingOverrideDraft, setStagingOverrideDraft] = useState("");
  const [profileFilter, setProfileFilter] = useState("");
  const [refreshing, setRefreshing] = useState(false);
  const [platform, setPlatform] = useState("windows");
  const [appVersion, setAppVersion] = useState<string | null>(null);
  const [themes, setThemes] = useState<ThemeSummary[]>([]);
  const [apiKeyValidating, setApiKeyValidating] = useState(false);
  const [apiKeyStatus, setApiKeyStatus] = useState<string | null>(null);
  const [themeInstalling, setThemeInstalling] = useState(false);
  const [themeStatus, setThemeStatus] = useState<string | null>(null);
  const [gameStatus, setGameStatus] = useState<string | null>(null);
  const [updateCheckBusy, setUpdateCheckBusy] = useState(false);
  const [updateBanner, setUpdateBanner] = useState<string | null>(null);
  const [pendingAppUpdate, setPendingAppUpdate] = useState<Update | null>(null);
  const [installingUpdate, setInstallingUpdate] = useState(false);
  const { applyTheme, getSlot } = useThemeLayout();
  const compactBarVisible =
    ((settings?.compactGameSidebar ?? false) || getSlot("shell.compactBar")?.enabled === true) &&
    !(settings?.compactGameSidebarHidden ?? false);

  const handleCheckAppUpdates = async () => {
    if (!isTauri()) {
      setUpdateBanner("App updates run in the desktop build.");
      return;
    }
    setUpdateCheckBusy(true);
    setUpdateBanner(null);
    setPendingAppUpdate(null);
    try {
      const available = await checkForUpdates();
      if (available) {
        setPendingAppUpdate(available);
        const note = typeof available.body === "string" ? available.body : "";
        setUpdateBanner(`Update ${available.version} available.${note ? ` ${note}` : ""}`);
      } else {
        setUpdateBanner("Supervisor is up to date.");
      }
    } catch (e) {
      setUpdateBanner(formatInvokeError(e));
    } finally {
      setUpdateCheckBusy(false);
    }
  };

  const handleInstallAppUpdate = async () => {
    const u = pendingAppUpdate;
    if (!u) return;
    setInstallingUpdate(true);
    try {
      await installUpdate(u);
    } catch (e) {
      setUpdateBanner(formatInvokeError(e));
    } finally {
      setInstallingUpdate(false);
    }
  };

  const nexusApiKeyPlaceholder =
    settings?.hasNexusApiKey && !settings?.nexusApiKey
      ? "API key saved in system keychain"
      : "Paste your Nexus API key";

  const refreshScan = async (includeAll = settings?.showUnmoddableGames ?? false) => {
    setRefreshing(true);
    setGameStatus(null);
    try {
      const result = await scanGames({ includeAll });
      onGamesLoaded?.(result.games);
    } catch (e) {
      setGameStatus(formatInvokeError(e));
    } finally {
      setRefreshing(false);
    }
  };

  const addManual = async () => {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const path = await open({ directory: true, multiple: false });
    if (!path || typeof path !== "string") return;
    setGameStatus(null);
    try {
      const game = await addManualGame(path);
      onSelectGame?.(game);
      onGamesLoaded?.([...games.filter((g) => g.id !== game.id), game]);
    } catch (e) {
      setGameStatus(formatInvokeError(e));
    }
  };

  const removeGame = async (game: DetectedGame) => {
    setGameStatus(null);
    try {
      await removeManualGame(game.id);
      const next = games.filter((g) => g.id !== game.id);
      onGamesLoaded?.(next);
    } catch (e) {
      setGameStatus(formatInvokeError(e));
    }
  };

  const updateNexusDomain = async (game: DetectedGame, domain: string) => {
    setGameStatus(null);
    try {
      const updated = await updateManualGameNexusDomain(game.id, domain || null);
      onGamesLoaded?.(games.map((g) => (g.id === updated.id ? updated : g)));
    } catch (e) {
      setGameStatus(formatInvokeError(e));
    }
  };

  useEffect(() => {
    void getAppSettings().then((s) => {
      const merged = mergeSettings(s);
      setSettings(merged);
      setApiKeyDraft(merged.nexusApiKey ?? "");
      setLootDraft(merged.lootPath ?? "");
      setModEngineDraft(merged.modEngineLauncherPath ?? "");
      setStagingOverrideDraft(merged.stagingRootOverride ?? "");
    });
    void getAppPaths().then(setPaths);
    void listSupportedProfiles().then(setProfiles);
    void getPlatform().then(setPlatform).catch(() => setPlatform("windows"));
    void listThemes().then(setThemes).catch(console.error);
    if (isTauri()) {
      void import("@tauri-apps/api/app")
        .then(({ getVersion }) => getVersion())
        .then(setAppVersion)
        .catch(() => setAppVersion(null));
    }

    let unlisten: (() => void) | undefined;
    void listenSettingsChanged((next) => {
      const merged = mergeSettings(next);
      setSettings(merged);
      setApiKeyDraft(merged.nexusApiKey ?? "");
      setLootDraft(merged.lootPath ?? "");
      setModEngineDraft(merged.modEngineLauncherPath ?? "");
      setStagingOverrideDraft(merged.stagingRootOverride ?? "");
    }).then((fn) => {
      unlisten = fn;
    });

    return () => unlisten?.();
  }, []);

  useEffect(() => {
    if (!selectedGame) {
      setGameStaging(null);
      return;
    }
    void getStagingDir(selectedGame.id)
      .then(setGameStaging)
      .catch(() => setGameStaging(null));
  }, [selectedGame]);

  const persist = async (patch: Partial<AppSettings>) => {
    if (!settings) return;
    setSaving(true);
    try {
      const next = await updateAppSettings(mergeSettings({ ...settings, ...patch }));
      setSettings(mergeSettings(next));
    } catch (e) {
      setGameStatus(formatInvokeError(e));
    } finally {
      setSaving(false);
    }
  };

  useEffect(() => {
    if (!settings || themes.length === 0) return;
    const active = settings.activeThemeId ?? "default";
    if (themes.some((t) => t.id === active)) return;
    void (async () => {
      await applyTheme("default");
      await persist({ activeThemeId: "default" });
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps -- reconcile stale activeThemeId once themes load
  }, [settings?.activeThemeId, themes.length]);

  const filteredProfiles = useMemo(() => {
    const q = profileFilter.trim().toLowerCase();
    if (!q) return profiles;
    return profiles.filter(
      (p) =>
        p.name.toLowerCase().includes(q) ||
        p.id.toLowerCase().includes(q) ||
        p.primaryModPath.toLowerCase().includes(q) ||
        p.nexusDomains?.some((d) => d.toLowerCase().includes(q)),
    );
  }, [profiles, profileFilter]);

  const saveApiKey = () => {
    void persist({ nexusApiKey: apiKeyDraft.trim() || undefined });
  };

  const saveLootPath = () => {
    void persist({ lootPath: lootDraft.trim() || undefined });
  };

  const saveModEnginePath = () => {
    void persist({ modEngineLauncherPath: modEngineDraft.trim() || undefined });
  };

  const saveStagingOverride = () => {
    void persist({ stagingRootOverride: stagingOverrideDraft.trim() || undefined });
  };

  const testApiKey = async () => {
    setApiKeyValidating(true);
    setApiKeyStatus(null);
    try {
      if (apiKeyDraft.trim() !== (settings?.nexusApiKey ?? "")) {
        await persist({ nexusApiKey: apiKeyDraft.trim() || undefined });
      }
      await validateNexusApiKey();
      setApiKeyStatus("API key is valid.");
    } catch (e) {
      setApiKeyStatus(formatInvokeError(e));
    } finally {
      setApiKeyValidating(false);
    }
  };

  const handleClearFailedDownloads = async () => {
    try {
      const removed = await clearFailedDownloads();
      setApiKeyStatus(removed > 0 ? `Removed ${removed} failed download(s).` : "No failed downloads to clear.");
    } catch (e) {
      console.error(e);
    }
  };

  const handleClearFinishedDownloads = async () => {
    try {
      const removed = await clearFinishedDownloads();
      setApiKeyStatus(
        removed > 0 ? `Removed ${removed} finished download(s).` : "No finished downloads to clear.",
      );
    } catch (e) {
      console.error(e);
    }
  };

  const handleInstallTheme = async () => {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const path = await open({
      filters: [
        { name: "Supervisor Theme", extensions: ["svtheme", "zip"] },
      ],
      multiple: false,
    });
    if (!path || typeof path !== "string") return;
    setThemeInstalling(true);
    setThemeStatus(null);
    try {
      const installed = await installTheme(path);
      const nextThemes = await listThemes();
      setThemes(nextThemes);
      const ok = await applyTheme(installed.id);
      if (ok) {
        await persist({ activeThemeId: installed.id });
        setThemeStatus(`Installed "${installed.name}".`);
      } else {
        setThemeStatus(`Installed "${installed.name}" but could not activate it. Try selecting it again.`);
      }
    } catch (e) {
      setThemeStatus(formatInvokeError(e));
    } finally {
      setThemeInstalling(false);
    }
  };

  const handleThemeChange = async (themeId: string) => {
    setThemeStatus(null);
    const ok = await applyTheme(themeId);
    if (ok) {
      await persist({ activeThemeId: themeId });
    } else {
      setThemeStatus(
        themeId === "default"
          ? "Could not apply the default theme."
          : `Theme "${themeId}" is missing or incomplete. Install it from Settings → Appearance, or pick Default.`,
      );
      await persist({ activeThemeId: "default" });
    }
  };

  const handleOpenThemesFolder = async () => {
    setThemeStatus(null);
    try {
      await openThemesFolder();
    } catch (e) {
      setThemeStatus(formatInvokeError(e));
    }
  };

  return (
    <div className="flex h-full min-h-0 flex-col overflow-hidden">
      <div className="flex shrink-0 items-center justify-between px-4 py-3">
        <h1 className="text-lg font-light text-text-primary">Settings</h1>
        {!embedded && onBack && (
          <Button variant="ghost" size="sm" onClick={onBack}>
            Back
          </Button>
        )}
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto p-6">
        <div className="mx-auto flex max-w-3xl flex-col gap-8">
          {nxmNotice && (
            <div className="rounded-xl border border-primary/30 bg-primary/10 px-4 py-3 text-sm text-text-primary">
              <p>{nxmNotice}</p>
              {onClearNxmNotice && (
                <button
                  type="button"
                  className="mt-2 text-xs text-primary hover:underline"
                  onClick={onClearNxmNotice}
                >
                  Dismiss
                </button>
              )}
            </div>
          )}

          <Section title="Games">
            <p className="text-sm text-text-muted">
              Refresh launcher scans, add local game folders, and manage detected installs.
            </p>
            <div className="overflow-hidden rounded-lg border border-border -m-1">
              <GamesTable
                games={games}
                selectedId={selectedGame?.id}
                refreshing={refreshing || gamesLoading}
                maxListHeight="30rem"
                onSelect={(g) => onSelectGame?.(g)}
                onRefresh={() => void refreshScan()}
                onAddLocal={() => void addManual()}
                onRemove={(g) => void removeGame(g)}
                onOpenFolder={(g) => void openPath(g.installPath).catch((e) => setGameStatus(formatInvokeError(e)))}
                onUpdateNexusDomain={(g, domain) => void updateNexusDomain(g, domain)}
              />
            </div>
            {gameStatus && <p className="text-sm text-error">{gameStatus}</p>}
          </Section>

          <Section title="General">
            <SelectRow
              label="Update check frequency"
              description="When Supervisor checks Nexus for mod updates."
              value={settings?.updateCheckMode ?? "onRefresh"}
              onChange={(v) => void persist({ updateCheckMode: v as UpdateCheckMode })}
              options={[
                { value: "manual", label: "Manual only" },
                { value: "onRefresh", label: "When refreshing mod list" },
                { value: "onStartup", label: "On app startup" },
              ]}
            />
            <div className="flex flex-wrap items-center gap-2">
              <Button
                variant="secondary"
                size="sm"
                disabled={updateCheckBusy || !isTauri()}
                onClick={() => void handleCheckAppUpdates()}
              >
                {updateCheckBusy ? "Checking…" : "Check for app updates"}
              </Button>
              {pendingAppUpdate && (
                <Button
                  variant="accent"
                  size="sm"
                  disabled={installingUpdate}
                  onClick={() => void handleInstallAppUpdate()}
                >
                  {installingUpdate ? "Installing…" : "Download and restart"}
                </Button>
              )}
            </div>
            {updateBanner && <p className="text-xs text-text-muted">{updateBanner}</p>}
            {!isTauri() && (
              <p className="text-xs text-text-muted">Install Supervisor for Windows to manage app updates.</p>
            )}
            <ToggleRow
              label="Remember last selected game"
              description="Restore the game you were managing when reopening Supervisor."
              checked={settings?.rememberLastGame ?? true}
              onChange={(checked) => void persist({ rememberLastGame: checked })}
            />
            <ToggleRow
              label="Compact game bar"
              description="Show a narrow game icon bar beside the main sidebar. Themes can suggest this too; you can override here."
              checked={compactBarVisible}
              onChange={(checked) =>
                void persist({
                  compactGameSidebar: checked,
                  compactGameSidebarHidden: !checked,
                })
              }
            />
            <ToggleRow
              label="Always show Plugins"
              description="Keep the Plugins sidebar entry visible even for games without load-order plugin support."
              checked={settings?.alwaysShowPlugins ?? false}
              onChange={(checked) => void persist({ alwaysShowPlugins: checked })}
            />
            <ToggleRow
              label="Compact mod list"
              description="Use tighter rows in the mod library table."
              checked={settings?.compactModList ?? false}
              onChange={(checked) => void persist({ compactModList: checked })}
            />
            <ToggleRow
              label="Show profile warnings"
              description="Surface guidance when a game uses a generic profile or mismatched file types."
              checked={settings?.showProfileWarnings ?? true}
              onChange={(checked) => void persist({ showProfileWarnings: checked })}
            />
            <ToggleRow
              label="Developer tools"
              description="Enable native right-click menus, text selection, and WebView developer tools."
              checked={settings?.developerTools ?? false}
              onChange={(checked) => void persist({ developerTools: checked })}
            />
          </Section>

          <Section title="Appearance">
            <SelectRow
              label="Theme"
              description="Install .svtheme packages to customize the app with CSS. See docs/reference/themes/ and themes/packages/example/."
              value={settings?.activeThemeId ?? "default"}
              onChange={(value) => void handleThemeChange(value)}
              options={
                themes.length > 0
                  ? themes.map((t) => ({ value: t.id, label: t.name }))
                  : [{ value: "default", label: "Default" }]
              }
            />
            <div className="flex flex-wrap gap-2">
              <Button
                variant="secondary"
                size="sm"
                disabled={themeInstalling}
                onClick={() => void handleInstallTheme()}
              >
                Install theme…
              </Button>
              {paths?.themesDir && (
                <Button variant="ghost" size="sm" onClick={() => void handleOpenThemesFolder()}>
                  Open themes folder
                </Button>
              )}
            </div>
            {themeStatus && <p className="text-xs text-text-muted">{themeStatus}</p>}
          </Section>

          <Section title="Nexus Mods">
            <p className="text-sm text-text-muted">
              API key required for mod downloads (via download_link API), metadata, and update
              checks. Get one at{" "}
              <a
                href="https://www.nexusmods.com/users/myaccount?tab=api"
                className="text-primary underline"
                target="_blank"
                rel="noreferrer"
              >
                nexusmods.com → My Account → API
              </a>
              .
            </p>
            <TextFieldRow
              label="API key"
              type="password"
              value={apiKeyDraft}
              onChange={setApiKeyDraft}
              placeholder={nexusApiKeyPlaceholder}
              onSave={saveApiKey}
              saving={saving}
            />
            {settings?.hasNexusApiKey && (
              <p className="text-xs text-text-muted">
                A key is stored on this PC. Paste a new value to replace it, or leave the field unchanged.
              </p>
            )}
            <div className="flex items-center gap-2">
              <Button variant="secondary" size="sm" disabled={apiKeyValidating} onClick={() => void testApiKey()}>
                {apiKeyValidating ? "Testing…" : "Test API key"}
              </Button>
              {apiKeyStatus && <span className="text-xs text-text-muted">{apiKeyStatus}</span>}
            </div>
          </Section>

          <Section title="Downloads">
            <NumberRow
              label="Max concurrent downloads"
              description="How many NXM downloads run at once (1–6)."
              value={settings?.maxConcurrentDownloads ?? 2}
              min={1}
              max={6}
              onChange={(v) => void persist({ maxConcurrentDownloads: v })}
            />
            <ToggleRow
              label="Auto-start queued downloads"
              description="Begin downloads as soon as they are added to the queue."
              checked={settings?.autoStartDownloads ?? true}
              onChange={(checked) => void persist({ autoStartDownloads: checked })}
            />
            <NumberRow
              label="Speed limit (KB/s)"
              description="Leave at 0 for unlimited. Applies to future download sessions."
              value={settings?.downloadSpeedLimitKbps ?? 0}
              min={0}
              max={100000}
              onChange={(v) =>
                void persist({ downloadSpeedLimitKbps: v > 0 ? v : undefined })
              }
            />
            <Button variant="secondary" size="sm" onClick={() => void handleClearFailedDownloads()}>
              Clear failed downloads
            </Button>
            <Button variant="ghost" size="sm" onClick={() => void handleClearFinishedDownloads()}>
              Clear finished downloads
            </Button>
          </Section>

          <Section title="Deployment">
            <p className="text-sm text-text-secondary">
              Method:{" "}
              <span className="text-text-muted">{settings?.deployMethod ?? "hardlink"}</span>
            </p>
            <p className="text-xs text-text-muted">
              Mods are hardlinked into the game folder when staging and game share a drive.
            </p>
            <ToggleRow
              label="Auto-deploy when enabling or disabling mods"
              checked={settings?.autoDeployOnChange ?? true}
              onChange={(checked) => void persist({ autoDeployOnChange: checked })}
            />
            <ToggleRow
              label="Prefer script extender (Bethesda)"
              description="Launch SKSE / F4SE / SFSE when present in the game folder."
              checked={settings?.preferScriptExtender ?? true}
              onChange={(checked) => void persist({ preferScriptExtender: checked })}
            />
            <ToggleRow
              label="Purge before deploy"
              description="Remove previously deployed hardlinks before each new deploy."
              checked={settings?.autoPurgeBeforeDeploy ?? false}
              onChange={(checked) => void persist({ autoPurgeBeforeDeploy: checked })}
            />
            <ToggleRow
              label="Confirm before deploy"
              description="Require explicit confirmation in the deploy flow (UI hook)."
              checked={settings?.confirmBeforeDeploy ?? false}
              onChange={(checked) => void persist({ confirmBeforeDeploy: checked })}
            />
            <ToggleRow
              label="Verify after deploy"
              description="Re-check hardlinks after deployment completes."
              checked={settings?.verifyAfterDeploy ?? true}
              onChange={(checked) => void persist({ verifyAfterDeploy: checked })}
            />
            <ToggleRow
              label="Ignore deploy requirements"
              description="Skip BepInEx, SMF, and other prerequisite checks when deploying."
              checked={settings?.ignoreDeployRequirements ?? false}
              onChange={(checked) => void persist({ ignoreDeployRequirements: checked })}
            />
          </Section>

          <Section title="LOOT & plugins">
            <p className="text-sm text-text-muted">
              Optional path to loot.exe for automatic plugin sorting on Bethesda games.
            </p>
            <TextFieldRow
              label="LOOT executable"
              value={lootDraft}
              onChange={setLootDraft}
              placeholder="C:\\Program Files\\LOOT\\loot.exe"
              onSave={saveLootPath}
              saving={saving}
            />
            <ToggleRow
              label="Auto-sort plugins with LOOT"
              description="Run LOOT when sorting plugins if loot.exe is configured."
              checked={settings?.autoSortPlugins ?? true}
              onChange={(checked) => void persist({ autoSortPlugins: checked })}
            />
          </Section>

          <Section title="Game detection">
            <p className="text-sm text-text-muted">
              Choose which launchers to scan. Rescan from Settings → Games after changing these.
            </p>
            <ToggleRow
              label="Scan Steam libraries"
              checked={settings?.scanSteam ?? true}
              onChange={(checked) => void persist({ scanSteam: checked })}
            />
            <ToggleRow
              label="Scan Epic Games"
              checked={settings?.scanEpic ?? true}
              onChange={(checked) => void persist({ scanEpic: checked })}
            />
            <ToggleRow
              label="Scan GOG Galaxy"
              checked={settings?.scanGog ?? true}
              onChange={(checked) => void persist({ scanGog: checked })}
            />
            {platform !== "windows" && (
              <ToggleRow
                label="Scan Heroic (Linux)"
                checked={settings?.scanHeroic ?? true}
                onChange={(checked) => void persist({ scanHeroic: checked })}
              />
            )}
            <ToggleRow
              label="Show all installed games"
              description="Include every installed title — tools, SDKs, redistributables, and non-game apps."
              checked={settings?.showUnmoddableGames ?? false}
              onChange={(checked) => void persist({ showUnmoddableGames: checked })}
            />
          </Section>

          <Section title="Collections">
            <ToggleRow
              label="Auto-enable mods from collections"
              description="Enable mods after installing from a .collection file."
              checked={settings?.collectionsAutoEnable ?? true}
              onChange={(checked) => void persist({ collectionsAutoEnable: checked })}
            />
            <ToggleRow
              label="Skip optional collection mods"
              description="Do not install mods marked optional in collection manifests."
              checked={settings?.collectionsSkipOptional ?? false}
              onChange={(checked) => void persist({ collectionsSkipOptional: checked })}
            />
          </Section>

          <Section title="App data">
            <dl className="grid gap-2 text-sm">
              <div className="flex justify-between gap-4">
                <dt className="text-text-muted">Version</dt>
                <dd className="text-text-primary">{appVersion ?? "—"}</dd>
              </div>
              <div className="flex justify-between gap-4">
                <dt className="text-text-muted">Platform</dt>
                <dd className="text-text-primary">{platform}</dd>
              </div>
            </dl>
            <PathRow
              label="App data"
              path={paths?.appDataDir}
              onOpen={paths ? () => void openPath(paths.appDataDir) : undefined}
            />
            <PathRow
              label="Staging root"
              path={paths?.stagingRoot}
              onOpen={paths ? () => void openPath(paths.stagingRoot) : undefined}
            />
            <PathRow
              label="Downloads"
              path={paths?.downloadsDir}
              onOpen={paths ? () => void openPath(paths.downloadsDir) : undefined}
            />
            <PathRow
              label="Themes"
              path={paths?.themesDir}
              onOpen={paths ? () => void handleOpenThemesFolder() : undefined}
            />
            {selectedGame && gameStaging && (
              <PathRow
                label={`Staging · ${selectedGame.name}`}
                path={gameStaging}
                onOpen={() => void openPath(gameStaging)}
              />
            )}
            <TextFieldRow
              label="Staging root override"
              description="Use a custom folder for all game staging (same drive as games recommended)."
              value={stagingOverrideDraft}
              onChange={setStagingOverrideDraft}
              placeholder="D:\\SupervisorStaging"
              onSave={saveStagingOverride}
              saving={saving}
            />
          </Section>

          <Section title="Advanced">
            <TextFieldRow
              label="ModEngine2 launcher"
              value={modEngineDraft}
              onChange={setModEngineDraft}
              placeholder="C:\\ModEngine2\\launchmod_eldenring.bat"
              onSave={saveModEnginePath}
              saving={saving}
            />
            <ToggleRow
              label="Debug logging"
              description="Write verbose logs to the app data folder."
              checked={settings?.debugLogging ?? false}
              onChange={(checked) => void persist({ debugLogging: checked })}
            />
          </Section>

          <Section title={`Supported games (${profiles.length})`}>
            <p className="text-sm text-text-muted">
              Built-in profiles for top Nexus Mods titles. Unknown games fall back to a generic
              Data/ deploy target.
            </p>
            <input
              type="search"
              value={profileFilter}
              onChange={(e) => setProfileFilter(e.target.value)}
              placeholder="Filter by name, id, or mod path…"
              className="w-full rounded-xl bg-input-bg px-3 py-2 text-sm text-text-primary outline-none focus:ring-1 focus:ring-primary"
            />
            <div className="max-h-80 overflow-y-auto rounded-xl bg-panel-secondary">
              <table className="w-full text-left text-sm">
                <thead className="sticky top-0 bg-panel-secondary text-text-muted">
                  <tr>
                    <th className="px-3 py-2 font-medium">Game</th>
                    <th className="px-3 py-2 font-medium">Mod path</th>
                    <th className="px-3 py-2 font-medium">Plugins</th>
                  </tr>
                </thead>
                <tbody>
                  {filteredProfiles.map((p) => (
                    <tr key={p.id} className="border-t border-white/5">
                      <td className="px-3 py-2 text-text-primary">{p.name}</td>
                      <td className="px-3 py-2 font-mono text-xs text-text-secondary">
                        {p.primaryModPath}
                      </td>
                      <td className="px-3 py-2 text-text-muted">
                        {p.supportsPlugins ? "Yes" : "—"}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
              {filteredProfiles.length === 0 && (
                <p className="p-4 text-sm text-text-muted">No profiles match your filter.</p>
              )}
            </div>
          </Section>
        </div>
      </div>
    </div>
  );
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section>
      <h2 className="mb-3 text-sm font-medium text-text-primary">{title}</h2>
      <div className="space-y-3 rounded-xl bg-panel-secondary p-3">{children}</div>
    </section>
  );
}

function ToggleRow({
  label,
  description,
  checked,
  onChange,
}: {
  label: string;
  description?: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  return (
    <label className="flex cursor-pointer gap-3 text-sm">
      <input
        type="checkbox"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
        className="mt-0.5 rounded accent-primary"
      />
      <span>
        <span className="text-text-secondary">{label}</span>
        {description && <p className="mt-0.5 text-xs text-text-muted">{description}</p>}
      </span>
    </label>
  );
}

function SelectRow({
  label,
  description,
  value,
  onChange,
  options,
}: {
  label: string;
  description?: string;
  value: string;
  onChange: (value: string) => void;
  options: { value: string; label: string }[];
}) {
  return (
    <div>
      <label className="mb-1 block text-sm text-text-muted">{label}</label>
      {description && <p className="mb-1 text-xs text-text-muted">{description}</p>}
      <Select value={value} onValueChange={onChange}>
        <SelectTrigger aria-label={label}>
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {options.map((o) => (
            <SelectItem key={o.value} value={o.value}>
              {o.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}

function NumberRow({
  label,
  description,
  value,
  min,
  max,
  onChange,
}: {
  label: string;
  description?: string;
  value: number;
  min: number;
  max: number;
  onChange: (value: number) => void;
}) {
  return (
    <div>
      <label className="mb-1 block text-sm text-text-muted">{label}</label>
      {description && <p className="mb-1 text-xs text-text-muted">{description}</p>}
      <input
        type="number"
        min={min}
        max={max}
        value={value}
        onChange={(e) => {
          const n = Number(e.target.value);
          if (!Number.isNaN(n)) onChange(Math.min(max, Math.max(min, n)));
        }}
        className="w-full rounded-xl bg-input-bg px-3 py-2 text-sm text-text-primary outline-none focus:ring-1 focus:ring-primary"
      />
    </div>
  );
}

function TextFieldRow({
  label,
  description,
  type = "text",
  value,
  onChange,
  placeholder,
  onSave,
  saving,
}: {
  label: string;
  description?: string;
  type?: string;
  value: string;
  onChange: (v: string) => void;
  placeholder?: string;
  onSave: () => void;
  saving: boolean;
}) {
  return (
    <div>
      <label className="mb-1 block text-sm text-text-muted">{label}</label>
      {description && <p className="mb-1 text-xs text-text-muted">{description}</p>}
      <div className="flex gap-2">
        <input
          type={type}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder={placeholder}
          className="min-w-0 flex-1 rounded-xl bg-input-bg px-3 py-2 text-sm text-text-primary outline-none focus:ring-1 focus:ring-primary"
        />
        <Button variant="secondary" size="sm" disabled={saving} onClick={onSave}>
          Save
        </Button>
      </div>
    </div>
  );
}

function PathRow({
  label,
  path,
  onOpen,
}: {
  label: string;
  path?: string;
  onOpen?: () => void;
}) {
  return (
    <div className="flex items-center justify-between gap-3 rounded-xl bg-input-bg/40 p-2.5">
      <div className="min-w-0">
        <p className="text-md text-text-muted">{label}</p>
        <p className="truncate text-sm text-text-secondary">{path ?? "…"}</p>
      </div>
      {path && onOpen && (
        <Button variant="ghost" size="sm" className="shrink-0 text-sm" onClick={onOpen}>
          Open
        </Button>
      )}
    </div>
  );
}
