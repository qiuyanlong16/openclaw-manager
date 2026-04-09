import { useEffect, useState, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";

export interface LogEntry {
  level: "info" | "warn" | "error";
  message: string;
  timestamp: string;
}

export function useLogListener() {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [isDeploying, setIsDeploying] = useState(false);
  const [isUninstalling, setIsUninstalling] = useState(false);

  useEffect(() => {
    const unlisten = listen<{
      level: string;
      message: string;
    }>("deploy-log", (event) => {
      const entry: LogEntry = {
        level: event.payload.level as "info" | "warn" | "error",
        message: event.payload.message,
        timestamp: new Date().toLocaleTimeString(),
      };
      setLogs((prev) => [...prev, entry]);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const clearLogs = useCallback(() => setLogs([]), []);

  return { logs, clearLogs, isDeploying, setIsDeploying, isUninstalling, setIsUninstalling };
}
