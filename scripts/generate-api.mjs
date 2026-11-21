import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";

const pin = JSON.parse(readFileSync("vendor/delphi/pin.json", "utf8"));
const url = `https://github.com/${pin.repository}/releases/download/${pin.tag}/${pin.asset}`;
const response = await fetch(url);
if (!response.ok) {
  throw new Error(`GET ${url} returned ${response.status}`);
}
const bytes = Buffer.from(await response.arrayBuffer());
const digest = createHash("sha256").update(bytes).digest("hex");
if (digest !== pin.sha256) {
  throw new Error(`${pin.asset} sha256 ${digest} does not match the pin`);
}
const document = JSON.parse(bytes.toString("utf8"));
const path = "/v1/works:batch-check";
if (!document.paths?.[path]) {
  throw new Error(`${pin.asset} has no ${path}`);
}
mkdirSync("src/generated", { recursive: true });
writeFileSync(
  "src/generated/schema.ts",
  `/* Generated from ${pin.repository}@${pin.tag} ${pin.asset}. Do not edit. */
export const batchCheckPath = ${JSON.stringify(path)} as const;
`,
);
console.log(`wrote src/generated/schema.ts from ${pin.repository}@${pin.tag}`);
