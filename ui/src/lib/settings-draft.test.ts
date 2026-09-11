import { describe, expect, it } from "vitest";
import { reconcileSettingsDraft } from "./settings-draft";
import type { Settings } from "./types";

const liveSettings: Settings = {
  schemaVersion: 3,
  automationEnabled: true,
  combatLimitBytesPerSecond: 10_000_000,
  throttleWhileAlive: true,
  downloadWhileDead: true,
  downloadBetweenRounds: true,
  pauseDuringOtherModes: true,
  restoreOnExit: true,
  startWithWindows: false,
  closeToTray: true,
};

describe("reconcileSettingsDraft", () => {
  it("preserves unsaved edits when a live telemetry snapshot arrives", () => {
    const draft = { ...liveSettings, pauseDuringOtherModes: false };
    const refreshed = { ...liveSettings, combatLimitBytesPerSecond: 12_000_000 };

    expect(reconcileSettingsDraft(draft, true, refreshed)).toEqual(draft);
  });

  it("refreshes an untouched draft from the latest saved settings", () => {
    const refreshed = { ...liveSettings, pauseDuringOtherModes: false };

    expect(reconcileSettingsDraft(liveSettings, false, refreshed)).toEqual(refreshed);
  });
});
