import type { BandwidthAction, Observation } from "./types";

const ruleConditions: Record<string, string> = {
  idle: "League is outside a match",
  loading: "A match is loading",
  nonArenaMatch: "You are in a non-Arena match",
  arenaPreparation: "Arena is between fights",
  arenaCombatDead: "You are dead during Arena combat",
  arenaCombat: "You are alive or uncertain during Arena combat",
  always: "No earlier rule matches",
};

export function actionLabel(action: BandwidthAction): string {
  if (action.kind === "unlimited") return "Unlimited";
  if (action.kind === "pause") return "Downloads paused";
  return `Limited to ${(action.bytesPerSecond / 1_000_000).toFixed(1)} MB/s`;
}

export function stateLabel(state: Observation): string {
  if (state.clientPhase === "idle") return "League idle";
  if (state.clientPhase === "lobby") return "League lobby";
  if (state.clientPhase === "matchmaking") return "Finding a match";
  if (state.clientPhase === "readyCheck") return "Ready check";
  if (state.clientPhase === "champSelect") return "Champion select";
  if (state.clientPhase === "loading") return "League loading";
  if (state.clientPhase === "endOfGame") return "League post-game";
  if (state.mode !== "arena") return state.mode === "other" ? "League match" : "Match type unknown";
  if (state.arenaPhase === "preparation") return "Arena preparation";
  if (state.arenaPhase === "combat")
    return state.life === "dead" ? "Arena combat · dead" : "Arena combat";
  return "Arena phase uncertain";
}

export function ruleConditionLabel(condition: string): string {
  return ruleConditions[condition] ?? condition;
}

export function ruleNameLabel(rule: { id: string; name: string }): string {
  return rule.id === "fail-safe" ? "Fallback" : rule.name;
}
