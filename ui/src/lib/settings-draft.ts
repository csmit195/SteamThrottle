import type { Settings } from "./types";

export function reconcileSettingsDraft(
  draft: Settings | null,
  dirty: boolean,
  live: Settings,
): Settings {
  return dirty && draft ? draft : { ...live };
}
