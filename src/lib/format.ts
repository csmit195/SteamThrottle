import type { BandwidthAction, Observation } from "./types";

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
  if (state.arenaPhase === "combat") return state.life === "dead" ? "Arena combat · dead" : "Arena combat";
  return "Arena phase uncertain";
}
