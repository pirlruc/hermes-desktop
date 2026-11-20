import { describe, expect, it } from "vitest";

import { batchCheck, chunk, type Identity } from "./api";

const identity = (value: string): Identity => ({ kind: "doi", value });

describe("chunk", () => {
  it("splits a request at 500 identities", () => {
    const items = Array.from({ length: 501 }, (_, index) => index);
    const parts = chunk(items);
    expect(parts).toHaveLength(2);
    expect(parts[0]).toHaveLength(500);
    expect(parts[1]).toEqual([500]);
  });
});

describe("batchCheck", () => {
  it("posts the api key and returns rows", async () => {
    const fetchImpl = async (_input: string, init?: RequestInit) => {
      expect(init?.headers).toMatchObject({ "x-api-key": "secret" });
      return new Response(JSON.stringify({ rows: [{ identity: identity("10.1/a"), known: true, work_id: "w" }] }), {
        status: 200,
      });
    };
    const rows = await batchCheck("http://127.0.0.1:8080/", "secret", [identity("10.1/a")], fetchImpl);
    expect(rows[0]?.work_id).toBe("w");
  });

  it("rejects a non-success status", async () => {
    const fetchImpl = async () => new Response("no", { status: 401 });
    await expect(batchCheck("http://127.0.0.1:8080", "secret", [identity("10.1/a")], fetchImpl)).rejects.toThrow(
      "401",
    );
  });
});
