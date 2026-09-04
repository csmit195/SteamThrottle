export type BandwidthAction =
  { kind: "unlimited" } | { kind: "pause" } | { kind: "limit"; bytesPerSecond: number };
export interface Observation {
  clientPhase:
    | "idle"
    | "lobby"
    | "matchmaking"
    | "readyCheck"
    | "champSelect"
    | "loading"
    | "inGame"
    | "endOfGame";
  mode: "arena" | "other" | "unknown";
  arenaPhase: "preparation" | "combat" | "ambiguous" | null;
  life: "alive" | "dead" | "unknown";
  confidence: number;
}
export interface Rule {
  id: string;
  name: string;
  enabled: boolean;
  priority: number;
  condition: string;
  action: BandwidthAction;
}
export interface Settings {
  schemaVersion: number;
  automationEnabled: boolean;
  combatLimitBytesPerSecond: number;
  restoreOnExit: boolean;
  startWithWindows: boolean;
  closeToTray: boolean;
  rules: Rule[];
}
export interface ActivityEntry {
  timestampMs: number;
  state: string;
  action: BandwidthAction;
  reason: string;
}
export interface RuntimeSnapshot {
  observation: Observation;
  matchedRule: { ruleId: string; ruleName: string; action: BandwidthAction };
  desiredAction: BandwidthAction;
  appliedAction: BandwidthAction | null;
  baselineAction: BandwidthAction | null;
  adapterHealth: string;
  lastTransition: number | null;
  automationEnabled: boolean;
  steamPath: string | null;
  settings: Settings;
  activity: ActivityEntry[];
}
