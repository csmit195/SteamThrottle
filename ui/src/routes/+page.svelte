<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { actionLabel, automationStatusLabel, ruleNameLabel, stateLabel } from "$lib/format";
  import type { RuntimeSnapshot } from "$lib/types";

  type Tab = "overview" | "behavior" | "settings";
  let tab = $state<Tab>("overview");
  let snapshot = $state<RuntimeSnapshot | null>(null);
  let busy = $state(false);
  let message = $state("");
  let error = $state("");

  const titles: Record<Tab, [string, string]> = {
    overview: ["Overview", "Live game and download state"],
    behavior: ["Behavior", "League download preferences"],
    settings: ["Settings", "Steam and application preferences"],
  };

  async function call<T>(command: string, args: Record<string, unknown> = {}): Promise<T> {
    error = "";
    try {
      return await invoke<T>(command, args);
    } catch (reason) {
      error = String(reason);
      throw reason;
    }
  }

  async function toggleAutomation() {
    if (!snapshot || busy) return;
    busy = true;
    try {
      snapshot = await call("set_automation", { enabled: !snapshot.automationEnabled });
    } finally {
      busy = false;
      await getCurrentWindow()
        .setFocus()
        .catch(() => {});
    }
  }

  async function saveSettings() {
    if (!snapshot || busy) return;
    busy = true;
    try {
      snapshot.settings.automationEnabled = snapshot.automationEnabled;
      snapshot = await call("save_policy", { settings: snapshot.settings });
      message = "Changes saved";
    } finally {
      busy = false;
    }
  }

  async function utility(command: string) {
    busy = true;
    try {
      message = await call<string>(command);
    } finally {
      busy = false;
    }
  }

  function minimizeWindow() {
    void getCurrentWindow().minimize();
  }

  function exitApp() {
    void invoke("quit_app");
  }

  function openExternal(event: MouseEvent, url: string) {
    event.preventDefault();
    void openUrl(url);
  }

  onMount(() => {
    const cleanups: Array<() => void> = [];
    call<RuntimeSnapshot>("get_snapshot")
      .then((value) => (snapshot = value))
      .catch(() => {});
    listen<RuntimeSnapshot>("state-changed", (event) => (snapshot = event.payload)).then((stop) =>
      cleanups.push(stop),
    );
    listen<string>(
      "update-available",
      (event) => (message = `Version ${event.payload} is available`),
    ).then((stop) => cleanups.push(stop));
    return () => cleanups.forEach((stop) => stop());
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
      <button class="close" onclick={exitApp} aria-label="Exit" title="Exit">
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
        <span>{automationStatusLabel(snapshot?.automationEnabled ?? false)}</span>
      </div>
    </aside>

    <section class="workspace">
      <header class="toolbar">
        <div>
          <h1>{titles[tab][0]}</h1>
          <span>{titles[tab][1]}</span>
        </div>
        {#if (tab === "behavior" || tab === "settings") && snapshot}<button
            class="button primary"
            onclick={saveSettings}
            disabled={busy}>Save</button
          >{/if}
      </header>

      {#if error}<div class="toast danger" role="alert">
          <span>{error}</span><button onclick={() => (error = "")}>×</button>
        </div>{/if}
      {#if message}<div class="toast">
          <span>{message}</span><button onclick={() => (message = "")}>×</button>
        </div>{/if}

      <div class:overview={tab === "overview" && snapshot !== null} class="content">
        {#if !snapshot}
          <div class="loading">
            <span></span><strong>Connecting</strong><small
              >Reading local League and Steam state</small
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
        {:else if tab === "behavior"}
          <section class="block">
            <div class="setting-row limit-row">
              <strong>Throttled speed limit</strong>
              <label
                ><input
                  type="number"
                  min="0.128"
                  max="125"
                  step="0.5"
                  value={snapshot.settings.combatLimitBytesPerSecond / 1_000_000}
                  onchange={(event) =>
                    (snapshot!.settings.combatLimitBytesPerSecond =
                      Number(event.currentTarget.value) * 1_000_000)}
                /><span>MB/s</span></label
              >
            </div>
          </section>
          <div class="list-heading"><span>Arena</span></div>
          <section class="block behavior-list">
            <label class="behavior-row">
              <span>Throttle while alive</span>
              <input
                class="behavior-checkbox"
                type="checkbox"
                bind:checked={snapshot.settings.throttleWhileAlive}
              />
            </label>
            <label class="behavior-row">
              <span>Download while dead</span>
              <input
                class="behavior-checkbox"
                type="checkbox"
                bind:checked={snapshot.settings.downloadWhileDead}
              />
            </label>
            <label class="behavior-row">
              <span>Download between rounds</span>
              <input
                class="behavior-checkbox"
                type="checkbox"
                bind:checked={snapshot.settings.downloadBetweenRounds}
              />
            </label>
          </section>
          <div class="list-heading"><span>Other modes</span></div>
          <section class="block behavior-list">
            <label class="behavior-row">
              <span>Pause downloads during matches</span>
              <input
                class="behavior-checkbox"
                type="checkbox"
                bind:checked={snapshot.settings.pauseDuringOtherModes}
              />
            </label>
          </section>
        {:else}
          <div class="list-heading"><span>Application</span></div>
          <section class="block settings-list">
            <label class="setting-row"
              ><div>
                <strong>Restore on exit</strong><span>Return Steam to its captured throttle.</span>
              </div>
              <input
                class="native-toggle"
                type="checkbox"
                bind:checked={snapshot.settings.restoreOnExit}
              /></label
            >
            <label class="setting-row"
              ><div>
                <strong>Close to tray</strong><span>Keep monitoring after closing the window.</span>
              </div>
              <input
                class="native-toggle"
                type="checkbox"
                bind:checked={snapshot.settings.closeToTray}
              /></label
            >
            <label class="setting-row"
              ><div>
                <strong>Start with Windows</strong><span>Launch silently when you sign in.</span>
              </div>
              <input
                class="native-toggle"
                type="checkbox"
                bind:checked={snapshot.settings.startWithWindows}
              /></label
            >
          </section>
          <div class="settings-actions">
            <button class="button" onclick={() => utility("check_for_update")} disabled={busy}
              >Check updates</button
            >
            <a
              href="https://github.com/csmit195/SteamThrottle"
              onclick={(event) => openExternal(event, "https://github.com/csmit195/SteamThrottle")}
              >View on GitHub</a
            >
          </div>
        {/if}
      </div>

      <footer>
        <span class:online={snapshot?.automationEnabled} class="footer-dot"></span><span
          >{snapshot ? stateLabel(snapshot.observation) : "Connecting"}</span
        ><span class="spacer"></span><span>Steam wrangled by</span><a
          href="https://csmit195.com"
          onclick={(event) => openExternal(event, "https://csmit195.com")}>csmit195</a
        ><span class="footer-divider"></span><span>v0.1.1</span>
      </footer>
    </section>
  </div>
</div>
