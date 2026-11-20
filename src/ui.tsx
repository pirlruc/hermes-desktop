import { useState } from "react";

import { batchCheck, type BatchRow, type Identity } from "./api";

export interface WorkView {
  key: string;
  title: string;
  doi: string | null;
  queued: boolean;
}

interface Bridge {
  library: () => Promise<WorkView[]>;
  grantConsent: (key: string) => Promise<void>;
  batchCheck: (identities: Identity[]) => Promise<BatchRow[]>;
}

const browserBridge: Bridge = {
  library: async () => [],
  grantConsent: async () => undefined,
  batchCheck: (identities) => batchCheck("http://127.0.0.1:8080", "", identities),
};

export function App({ bridge = browserBridge }: { bridge?: Bridge }) {
  const [works, setWorks] = useState<WorkView[]>([]);
  const [pending, setPending] = useState<string | null>(null);
  const [message, setMessage] = useState("Library not loaded");

  async function load() {
    const rows = await bridge.library();
    setWorks(rows);
    setMessage(`${rows.length} works`);
  }

  async function consent(key: string) {
    await bridge.grantConsent(key);
    setPending(null);
    setMessage(`Consent recorded for ${key}`);
  }

  return (
    <main>
      <h1>Hermes</h1>
      <p>{message}</p>
      <button type="button" onClick={() => void load()}>
        Load library
      </button>
      <ul>
        {works.map((work) => (
          <li key={work.key}>
            {work.title}
            {work.queued ? (
              <button type="button" onClick={() => setPending(work.key)}>
                Upload
              </button>
            ) : null}
          </li>
        ))}
      </ul>
      {pending ? (
        <dialog open>
          <p>Upload {pending} to Delphi?</p>
          <button type="button" onClick={() => void consent(pending)}>
            Allow
          </button>
          <button type="button" onClick={() => setPending(null)}>
            Cancel
          </button>
        </dialog>
      ) : null}
    </main>
  );
}
