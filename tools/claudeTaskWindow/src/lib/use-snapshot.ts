import { useEffect, useState } from "react";
import { fetchSnapshot, type Snapshot } from "./api";

const EMPTY_SNAPSHOT: Snapshot = {
  counts: {
    total: 0,
    needsUser: 0,
    completed: 0,
    running: 0,
  },
  tasks: [],
};

export function useSnapshot() {
  const [snapshot, setSnapshot] = useState<Snapshot>(EMPTY_SNAPSHOT);

  useEffect(() => {
    let active = true;

    const poll = async () => {
      try {
        const nextSnapshot = await fetchSnapshot();
        if (active) {
          setSnapshot(nextSnapshot);
        }
      } catch {}
    };

    void poll();
    const timer = window.setInterval(() => {
      void poll();
    }, 2000);

    return () => {
      active = false;
      window.clearInterval(timer);
    };
  }, []);

  return snapshot;
}
