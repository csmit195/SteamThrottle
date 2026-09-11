<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { actionLabel, automationStatusLabel, ruleNameLabel, stateLabel } from "$lib/format";
  import { reconcileSettingsDraft } from "$lib/settings-draft";
  import BandwidthControl from "$lib/BandwidthControl.svelte";
  import type { RuntimeSnapshot, Settings } from "$lib/types";

  type Tab = "overview" | "behavior" | "settings";
  let tab = $state<Tab>("overview");
  let snapshot = $state<RuntimeSnapshot | null>(null);
  let settingsDraft = $state<Settings | null>(null);
  let settingsDirty = $state(false);
  let busy = $state(false);
  let updateStatus = $state<"idle" | "checking" | "current" | "available" | "error">("idle");
  let automationFailed = $state(false);
  let connectionFailed = $state(false);
  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  let settingsRevision = 0;
  let settingsSaving = false;
  let bandwidthInteracting = false;

  const titles: Record<Tab, [string, string]> = {
    overview: ["Overview", "Live game and download state"],
    behavior: ["Behavior", "League download preferences"],
    settings: ["Settings", "Steam and application preferences"],
  };

  async function call<T>(command: string, args: Record<string, unknown> = {}): Promise<T> {
    return invoke<T>(command, args);
  }

  async function toggleAutomation() {
    if (!snapshot || busy) return;
    busy = true;
    automationFailed = false;
    try {
      acceptSnapshot(await call("set_automation", { enabled: !snapshot.automationEnabled }));
    } catch {
      automationFailed = true;
    } finally {
      busy = false;
      await getCurrentWindow()
        .setFocus()
        .catch(() => {});
    }
  }

  async function saveSettings() {
    if (!snapshot || !settingsDraft || settingsSaving || bandwidthInteracting) return;
    settingsSaving = true;
    const revision = settingsRevision;
    const settings = {
      ...settingsDraft,
      automationEnabled: snapshot.automationEnabled,
    };
    try {
      const saved = await call<RuntimeSnapshot>("save_policy", { settings });
      if (revision === settingsRevision) {
        settingsDirty = false;
        acceptSnapshot(saved);
      } else {
        acceptSnapshot(saved);
      }
    } catch {
      // Keep the draft dirty so a later edit can retry the save.
    } finally {
      settingsSaving = false;
      if (revision !== settingsRevision && !bandwidthInteracting) queueSettingsSave(0);
    }
  }

  async function checkForUpdates() {
    busy = true;
    updateStatus = "checking";
    try {
      const result = await call<string>("check_for_update");
      updateStatus = result.toLowerCase().includes("up to date") ? "current" : "available";
    } catch {
      updateStatus = "error";
    } finally {
      busy = false;
    }
  }

  function minimizeWindow() {
    void getCurrentWindow().minimize();
  }

  function closeWindow() {
    void getCurrentWindow()
      .close()
      .catch(() => {});
  }

  function acceptSnapshot(value: RuntimeSnapshot) {
    snapshot = value;
    settingsDraft = reconcileSettingsDraft(settingsDraft, settingsDirty, value.settings);
  }

  function markSettingsDirty() {
    settingsDirty = true;
    settingsRevision += 1;
    queueSettingsSave();
  }

  function beginBandwidthInteraction() {
    bandwidthInteracting = true;
    if (saveTimer === null) return;
    clearTimeout(saveTimer);
    saveTimer = null;
  }

  function markBandwidthDirty() {
    settingsDirty = true;
    settingsRevision += 1;
  }

  function commitBandwidth() {
    bandwidthInteracting = false;
    if (settingsDirty) queueSettingsSave(0);
  }

  function queueSettingsSave(delay = 350) {
    if (saveTimer !== null) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      saveTimer = null;
      void saveSettings();
    }, delay);
  }

  function updateButtonLabel() {
    if (updateStatus === "checking") return "Checking…";
    if (updateStatus === "current") return "Up to date";
    if (updateStatus === "available") return "Update available";
    if (updateStatus === "error") return "Check failed";
    return "Check now";
  }

  function openExternal(event: MouseEvent, url: string) {
    event.preventDefault();
    void openUrl(url);
  }

  onMount(() => {
    const cleanups: Array<() => void> = [];
    call<RuntimeSnapshot>("get_snapshot")
      .then(acceptSnapshot)
      .catch(() => (connectionFailed = true));
    listen<RuntimeSnapshot>("state-changed", (event) => acceptSnapshot(event.payload)).then(
      (stop) => cleanups.push(stop),
    );
    listen<string>("update-available", () => (updateStatus = "available")).then((stop) =>
      cleanups.push(stop),
    );
    return () => {
      cleanups.forEach((stop) => stop());
      if (saveTimer !== null) clearTimeout(saveTimer);
    };
  });
</script>

<svelte:head><title>Steam Throttle</title></svelte:head>

<div class="window-shell">
  <header class="titlebar" data-tauri-drag-region>
    <strong data-tauri-drag-region>Steam Throttle</strong>
    <div class="window-controls">
      <button onclick={minimizeWindow} aria-label="Minimize" title="Minimize">
        <svg viewBox="0 0 12 12" aria-hidden="true"><path d="M2 6.5h8" /></svg>
      </button>
      <button class="close" onclick={closeWindow} aria-label="Close" title="Close">
        <svg viewBox="0 0 12 12" aria-hidden="true"><path d="m2.5 2.5 7 7m0-7-7 7" /></svg>
      </button>
    </div>
  </header>

  <div class="app-shell">
    <aside class="sidebar">
      <nav aria-label="Sections">
        <span class="nav-label">Monitor</span>
        <button class:active={tab === "overview"} onclick={() => (tab = "overview")}>
          <svg viewBox="0 0 24 24"><path d="M4 13a8 8 0 1 1 16 0M12 13l4-4M5 19h14" /></svg><span
            >Overview</span
          >
        </button>
        <span class="nav-label">Configure</span>
        <button class:active={tab === "behavior"} onclick={() => (tab = "behavior")}>
          <svg viewBox="0 0 24 24"><path d="M4 7h10m4 0h2M4 17h2m4 0h10M14 4v6M6 14v6" /></svg><span
            >Behavior</span
          >
        </button>
        <button class:active={tab === "settings"} onclick={() => (tab = "settings")}>
          <svg viewBox="0 0 24 24"
            ><circle cx="12" cy="12" r="3" /><path
              d="M19 12a7 7 0 0 0-.1-1l2-1.5-2-3.4-2.4 1A8 8 0 0 0 15 6l-.3-2.6h-4L10.4 6a8 8 0 0 0-1.5.9l-2.4-1-2 3.5 2.1 1.5a7 7 0 0 0 0 2.1l-2 1.5 2 3.4 2.3-1A8 8 0 0 0 10.4 18l.3 2.6h4L15 18a8 8 0 0 0 1.5-.9l2.4 1 2-3.5-2.1-1.5A7 7 0 0 0 19 12Z"
            /></svg
          ><span>Settings</span>
        </button>
      </nav>

      <div class="automation-panel">
        <button
          class:enabled={snapshot?.automationEnabled}
          class="toggle"
          onclick={toggleAutomation}
          disabled={!snapshot || busy}
          aria-label="Toggle automation"><span></span></button
        >
        <span
          >{automationFailed
            ? "Couldn't update"
            : automationStatusLabel(snapshot?.automationEnabled ?? false)}</span
        >
      </div>
    </aside>

    <section class="workspace">
      <header class="toolbar">
        <div>
          <h1>{titles[tab][0]}</h1>
          <span>{titles[tab][1]}</span>
        </div>
      </header>

      <div class:overview={tab === "overview" && snapshot !== null} class="content">
        {#if !snapshot}
          <div class="loading">
            <span></span><strong>{connectionFailed ? "Connection failed" : "Connecting"}</strong
            ><small
              >{connectionFailed
                ? "Restart Steam Throttle and try again"
                : "Reading local League and Steam state"}</small
            >
          </div>
        {:else if tab === "overview"}
          <section class="live-state">
            <div class="live-copy">
              <span class="section-label">Current state</span>
              <h2>{stateLabel(snapshot.observation)}</h2>
              <p>
                {ruleNameLabel({
                  id: snapshot.matchedRule.ruleId,
                  name: snapshot.matchedRule.ruleName,
                })}
              </p>
            </div>
            <div class:restricted={snapshot.desiredAction.kind !== "unlimited"} class="action">
              <span>Steam</span><strong>{actionLabel(snapshot.desiredAction)}</strong>
            </div>
          </section>

          <section class="details">
            <div>
              <span>Status</span><strong
                >{snapshot.appliedAction ? actionLabel(snapshot.appliedAction) : "Waiting"}</strong
              >
            </div>
            <div><span>Steam connection</span><strong>{snapshot.adapterHealth}</strong></div>
          </section>

          <section class="block recent">
            <div class="block-title">
              <strong>Recent activity</strong>
            </div>
            {#if snapshot.activity.length === 0}<div class="empty">No state changes yet</div>{/if}
            {#each snapshot.activity.slice(0, 4) as entry}
              <div class="activity-row">
                <span class="event-dot"></span>
                <div><strong>{entry.state}</strong><small>{entry.reason}</small></div>
                <span>{actionLabel(entry.action)}</span><time
                  >{new Date(entry.timestampMs).toLocaleTimeString([], {
                    hour: "2-digit",
                    minute: "2-digit",
                  })}</time
                >
              </div>
            {/each}
          </section>
        {:else if tab === "behavior" && settingsDraft}
          <div class="settings-page">
            <section class="settings-section">
              <div class="settings-section-title">Bandwidth</div>
              <div class="settings-panel">
                <div class="setting-row limit-row">
                  <div>
                    <strong>Gameplay bandwidth limit</strong>
                    <span>Used whenever downloads need to be restricted without pausing.</span>
                  </div>
                  <BandwidthControl
                    value={settingsDraft.combatLimitBytesPerSecond}
                    onchange={(bytesPerSecond) => {
                      settingsDraft!.combatLimitBytesPerSecond = bytesPerSecond;
                      markBandwidthDirty();
                    }}
                    oninteractionstart={beginBandwidthInteraction}
                    oncommit={commitBandwidth}
                  />
                </div>
              </div>
            </section>

            <section class="settings-section">
              <div class="settings-section-title">Arena</div>
              <div class="settings-panel">
                <label class="setting-row">
                  <div>
                    <strong>Throttle while alive</strong>
                    <span>Turn off to allow full download speed during combat.</span>
                  </div>
                  <input
                    class="native-toggle"
                    type="checkbox"
                    bind:checked={settingsDraft.throttleWhileAlive}
                    onchange={markSettingsDirty}
                  />
                </label>
                <label class="setting-row">
                  <div>
                    <strong>Download while dead</strong>
                    <span>Turn off to use the gameplay limit while dead.</span>
                  </div>
                  <input
                    class="native-toggle"
                    type="checkbox"
                    bind:checked={settingsDraft.downloadWhileDead}
                    onchange={markSettingsDirty}
                  />
                </label>
                <label class="setting-row">
                  <div>
                    <strong>Download between rounds</strong>
                    <span>Turn off to use the gameplay limit during downtime.</span>
                  </div>
                  <input
                    class="native-toggle"
                    type="checkbox"
                    bind:checked={settingsDraft.downloadBetweenRounds}
                    onchange={markSettingsDirty}
                  />
                </label>
              </div>
            </section>

            <section class="settings-section">
              <div class="settings-section-title">Other game modes</div>
              <div class="settings-panel">
                <label class="setting-row">
                  <div>
                    <strong>Pause downloads</strong>
                    <span>Turn off to throttle to the gameplay limit instead.</span>
                  </div>
                  <input
                    class="native-toggle"
                    type="checkbox"
                    bind:checked={settingsDraft.pauseDuringOtherModes}
                    onchange={markSettingsDirty}
                  />
                </label>
              </div>
            </section>
          </div>
        {:else if settingsDraft}
          <div class="settings-page">
            <section class="settings-section">
              <div class="settings-section-title">Window and startup</div>
              <div class="settings-panel">
                <label class="setting-row">
                  <div>
                    <strong>Close to tray</strong>
                    <span>Keep monitoring when the window is closed.</span>
                  </div>
                  <input
                    class="native-toggle"
                    type="checkbox"
                    bind:checked={settingsDraft.closeToTray}
                    onchange={markSettingsDirty}
                  />
                </label>
                <label class="setting-row">
                  <div>
                    <strong>Start with Windows</strong>
                    <span>Launch quietly after signing in.</span>
                  </div>
                  <input
                    class="native-toggle"
                    type="checkbox"
                    bind:checked={settingsDraft.startWithWindows}
                    onchange={markSettingsDirty}
                  />
                </label>
              </div>
            </section>

            <section class="settings-section">
              <div class="settings-section-title">Steam</div>
              <div class="settings-panel">
                <label class="setting-row">
                  <div>
                    <strong>Restore previous limit on exit</strong>
                    <span>Put Steam back the way it was before automation started.</span>
                  </div>
                  <input
                    class="native-toggle"
                    type="checkbox"
                    bind:checked={settingsDraft.restoreOnExit}
                    onchange={markSettingsDirty}
                  />
                </label>
              </div>
            </section>

            <section class="settings-section">
              <div class="settings-section-title">About</div>
              <div class="settings-panel">
                <div class="setting-row action-row">
                  <div>
                    <strong>Software updates</strong>
                    <span>Check GitHub Releases for a newer version.</span>
                  </div>
                  <button
                    class="compact-button"
                    class:status-success={updateStatus === "current"}
                    class:status-notice={updateStatus === "available"}
                    class:status-error={updateStatus === "error"}
                    onclick={checkForUpdates}
                    disabled={busy}>{updateButtonLabel()}</button
                  >
                </div>
                <div class="setting-row action-row">
                  <div>
                    <strong>Steam Throttle</strong>
                    <span>Source code, releases, and issue tracker.</span>
                  </div>
                  <button
                    class="compact-button"
                    onclick={(event) =>
                      openExternal(event, "https://github.com/csmit195/SteamThrottle")}
                    >Open GitHub</button
                  >
                </div>
              </div>
            </section>
          </div>
        {/if}
      </div>

      <footer>
        <span class:online={snapshot?.automationEnabled} class="footer-dot"></span><span
          >{snapshot ? stateLabel(snapshot.observation) : "Connecting"}</span
        ><span class="spacer"></span><span>Steam wrangled by</span><a
          href="https://csmit195.com"
          onclick={(event) => openExternal(event, "https://csmit195.com")}>csmit195</a
        ><span class="footer-divider"></span><span>v0.1.2</span>
      </footer>
    </section>
  </div>
</div>
