import { describe, expect, it } from "vitest";

import { App } from "./ui";

describe("App", () => {
  it("exports the library view", () => {
    expect(typeof App).toBe("function");
  });
});
