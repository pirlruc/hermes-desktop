import { batchCheckPath } from "./generated/schema";

export interface Identity {
  kind: "doi" | "content_hash";
  value: string;
}

export interface BatchRow {
  identity: Identity;
  known: boolean;
  work_id: string | null;
}

export const BATCH_LIMIT = 500;

export function chunk<T>(items: T[], size = BATCH_LIMIT): T[][] {
  const width = Math.max(1, size);
  const chunks: T[][] = [];
  for (let index = 0; index < items.length; index += width) {
    chunks.push(items.slice(index, index + width));
  }
  return chunks;
}

type FetchLike = (input: string, init?: RequestInit) => Promise<Response>;

export async function batchCheck(
  baseUrl: string,
  apiKey: string,
  identities: Identity[],
  fetchImpl: FetchLike = fetch,
): Promise<BatchRow[]> {
  const rows: BatchRow[] = [];
  for (const group of chunk(identities)) {
    const response = await fetchImpl(`${baseUrl.replace(/\/$/, "")}${batchCheckPath}`, {
      method: "POST",
      headers: {
        "content-type": "application/json",
        "x-api-key": apiKey,
      },
      body: JSON.stringify({ identities: group }),
    });
    if (!response.ok) {
      throw new Error(`batch-check failed: ${response.status}`);
    }
    const body = (await response.json()) as { rows: BatchRow[] };
    rows.push(...body.rows);
  }
  return rows;
}
