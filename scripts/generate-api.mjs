import { readFileSync, writeFileSync, mkdirSync } from "node:fs";

const document = JSON.parse(readFileSync("vendor/delphi/openapi.json", "utf8"));
const path = "/v1/works:batch-check";
if (!document.paths?.[path]) {
  throw new Error(`openapi.json has no ${path}`);
}
mkdirSync("src/generated", { recursive: true });
writeFileSync(
  "src/generated/schema.ts",
  `/* Generated from vendor/delphi/openapi.json. Do not edit. */
export const batchCheckPath = ${JSON.stringify(path)} as const;
`,
);
console.log(`wrote src/generated/schema.ts for ${path}`);
