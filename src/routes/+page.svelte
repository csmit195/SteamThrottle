<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { actionLabel, stateLabel } from "$lib/format";
  import type { RuntimeSnapshot } from "$lib/types";

  type Tab = "overview" | "rules" | "activity" | "settings";
  let tab = $state<Tab>("overview");
  let snapshot = $state<RuntimeSnapshot | null>(null);
  let busy = $state(false);
  let message = $state("");
  let error = $state("");
  let showAdvanced = $state(false);

  async function call<T>(command: string, args: Record<string, unknown> = {}): Promise<T> {
    error = "";
    try { return await invoke<T>(command, args); }
    catch (reason) { error = String(reason); throw reason; }
  }

  async function toggleAutomation() {
    if (!snapshot || busy) return;
    busy = true;
    try { snapshot = await call("set_automation", { enabled: !snapshot.automationEnabled }); }
    finally { busy = false; }
  }

  async function saveSettings() {
    if (!snapshot || busy) return;
    busy = true;
    try {
      snapshot.settings.automationEnabled = snapshot.automationEnabled;
      snapshot = await call("save_policy", { settings: snapshot.settings });
      message = "Settings saved";
    } finally { busy = false; }
  }

  async function utility(command: string) {
    busy = true;
    try { message = await call<string>(command); } finally { busy = false; }
  }

  function moveRule(index: number, direction: -1 | 1) {
    if (!snapshot) return;
    const target = index + direction;
    if (target < 0 || target >= snapshot.settings.rules.length) return;
    const rules = snapshot.settings.rules;
    [rules[index], rules[target]] = [rules[target], rules[index]];
    rules.forEach((rule, priority) => rule.priority = priority);
  }

  onMount(() => {
    let stop: (() => void) | undefined;
    call<RuntimeSnapshot>("get_snapshot").then((value) => snapshot = value).catch(() => {});
    listen<RuntimeSnapshot>("state-changed", (event) => snapshot = event.payload).then((unlisten) => stop = unlisten);
    listen<string>("update-available", (event) => message = `SteamThrottle ${event.payload} is available on GitHub Releases`);
    return () => stop?.();
  });
</script>

<svelte:head><title>SteamThrottle</title></svelte:head>
<main>
  <header class="app-header">
    <div class="brand-mark" aria-hidden="true">ST</div>
    <div class="brand"><h1>SteamThrottle</h1><p>Bandwidth control for League players</p></div>
    <button class:active={snapshot?.automationEnabled} class="automation" onclick={toggleAutomation} disabled={!snapshot || busy}><span class="switch"><span></span></span>{snapshot?.automationEnabled ? "Active" : "Off"}</button>
  </header>
  <nav aria-label="Application sections">
    {#each ["overview", "rules", "activity", "settings"] as item}<button class:active={tab === item} onclick={() => tab = item as Tab}>{item}</button>{/each}
  </nav>
  {#if error}<div class="notice error" role="alert">{error}</div>{/if}
  {#if message}<button class="notice success" onclick={() => message = ""}>{message}<span>Dismiss</span></button>{/if}

  {#if !snapshot}
    <section class="loading"><div class="spinner"></div><h2>Connecting to local services</h2><p>Reading League and Steam state…</p></section>
  {:else if tab === "overview"}
    <section class="hero">
      <div class="eyebrow"><span class:limited={snapshot.desiredAction.kind !== "unlimited"} class="pulse"></span>Current decision</div>
      <h2>{actionLabel(snapshot.desiredAction)}</h2><p>{stateLabel(snapshot.observation)}</p>
      <div class="reason">Matched rule <strong>{snapshot.matchedRule.ruleName}</strong></div>
    </section>
    <section class="status-grid">
      <article><span>League state</span><strong>{stateLabel(snapshot.observation)}</strong><small>{snapshot.observation.confidence}% confidence</small></article>
      <article><span>Steam adapter</span><strong>{snapshot.adapterHealth}</strong><small>{snapshot.steamPath ?? "Choose steam.exe in Settings"}</small></article>
      <article><span>Applied action</span><strong>{snapshot.appliedAction ? actionLabel(snapshot.appliedAction) : "Waiting"}</strong><small>Changes only when required</small></article>
      <article><span>Safety mode</span><strong>{snapshot.observation.clientPhase === "loading" ? "Fail-safe" : "Normal"}</strong><small>Restrictive changes apply immediately</small></article>
    </section>
    {#if !snapshot.automationEnabled}<section class="first-run"><h3>Automation is off</h3><p>Review the combat policy, then enable automation when ready. Nothing changes in Steam until you opt in.</p><button class="primary" onclick={toggleAutomation}>Enable automation</button></section>{/if}
  {:else if tab === "rules"}
    <section class="section-head"><div><h2>Bandwidth rules</h2><p>First enabled match wins. Restrictive transitions apply immediately.</p></div><button class="primary" onclick={saveSettings} disabled={busy}>Save changes</button></section>
    <section class="card friendly"><label for="combat-limit">Arena combat limit</label><div class="input-row"><input id="combat-limit" type="number" min="0.128" max="125" step="0.5" value={snapshot.settings.combatLimitBytesPerSecond / 1_000_000} onchange={(event) => snapshot!.settings.combatLimitBytesPerSecond = Number(event.currentTarget.value) * 1_000_000} /><span>MB/s</span></div><p>Alive or uncertain during combat. Dead, shopping, voting, and intermission states are unlimited.</p></section>
    <button class="advanced-toggle" onclick={() => showAdvanced = !showAdvanced}>{showAdvanced ? "Hide" : "Show"} advanced rule order</button>
    {#if showAdvanced}<section class="rule-list">{#each snapshot.settings.rules as rule, index (rule.id)}<article class:disabled={!rule.enabled}><input aria-label={`Enable ${rule.name}`} type="checkbox" bind:checked={rule.enabled} /><div><strong>{rule.name}</strong><small>{rule.condition} → {actionLabel(rule.action)}</small></div><div class="move"><button aria-label="Move up" onclick={() => moveRule(index, -1)} disabled={index === 0}>↑</button><button aria-label="Move down" onclick={() => moveRule(index, 1)} disabled={index === snapshot!.settings.rules.length - 1}>↓</button></div></article>{/each}</section>{/if}
  {:else if tab === "activity"}
    <section class="section-head"><div><h2>Recent decisions</h2><p>Local, identity-free state changes from this session.</p></div></section>
    <section class="timeline">{#if snapshot.activity.length === 0}<div class="empty">No transitions recorded yet.</div>{/if}{#each snapshot.activity as entry}<article><span class="dot"></span><div><strong>{entry.state}</strong><p>{entry.reason} · {actionLabel(entry.action)}</p></div><time>{new Date(entry.timestampMs).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" })}</time></article>{/each}</section>
  {:else}
    <section class="section-head"><div><h2>Settings</h2><p>Local-only configuration. No account data leaves this PC.</p></div><button class="primary" onclick={saveSettings} disabled={busy}>Save</button></section>
    <section class="card settings-form">
      <label>Steam executable<input value={snapshot.settings.steamPath ?? ""} placeholder="C:\Program Files (x86)\Steam\steam.exe" onchange={(event) => snapshot!.settings.steamPath = event.currentTarget.value || null} /></label>
      <label class="check"><input type="checkbox" bind:checked={snapshot.settings.restoreOnExit} /><span><strong>Restore Steam on exit</strong><small>Remove the runtime throttle when automation stops.</small></span></label>
      <label class="check"><input type="checkbox" bind:checked={snapshot.settings.closeToTray} /><span><strong>Close to notification area</strong><small>Keep monitoring when the window closes.</small></span></label>
      <label class="check"><input type="checkbox" bind:checked={snapshot.settings.startWithWindows} /><span><strong>Start with Windows</strong><small>Registration is enabled in packaged builds.</small></span></label>
      <label class="check"><input type="checkbox" bind:checked={snapshot.settings.hardPauseEnabled} /><span><strong>Use Steam hard pause</strong><small>Opt in to Steam's pause gate instead of the safe 0.128 MB/s fallback.</small></span></label>
    </section>
    <section class="utility-grid"><button onclick={() => utility("test_steam_connection")} disabled={busy}><strong>Test Steam</strong><span>Apply a temporary 10.0 MB/s limit</span></button><button onclick={() => utility("restore_steam_now")} disabled={busy}><strong>Remove limit now</strong><span>Return Steam to unlimited</span></button><button onclick={() => utility("export_diagnostics")} disabled={busy}><strong>Export diagnostics</strong><span>Write a redacted local snapshot</span></button><button onclick={() => utility("check_for_update")} disabled={busy}><strong>Check for updates</strong><span>Use GitHub Releases</span></button></section>
  {/if}
  <footer><span>v0.1.0</span><span>Local only · No Overwolf · No network blocking</span></footer>
</main>
