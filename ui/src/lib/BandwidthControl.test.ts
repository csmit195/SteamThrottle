import { render } from "svelte/server";
import { describe, expect, it } from "vitest";
import BandwidthControl from "./BandwidthControl.svelte";

describe("BandwidthControl", () => {
  it("renders the current value in a dedicated width measurer", () => {
    const { body } = render(BandwidthControl, {
      props: {
        value: 125_000_000,
        onchange: () => {},
        oninteractionstart: () => {},
        oncommit: () => {},
      },
    });

    expect(body).toMatch(/<span class="value-sizer[^"]*" aria-hidden="true">125<\/span>/);
  });
});
