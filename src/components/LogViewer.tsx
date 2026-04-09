import { useRef, useEffect } from "react";
import { LogEntry } from "../hooks/useLogListener";

interface LogViewerProps {
  logs: LogEntry[];
}

export default function LogViewer({ logs }: LogViewerProps) {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [logs]);

  return (
    <div className="card log-card">
      <h2>日志</h2>
      <div className="log-viewer" ref={containerRef}>
        {logs.length === 0 ? (
          <div className="log-entry info">等待操作...</div>
        ) : (
          logs.map((entry, i) => (
            <div key={i} className={`log-entry ${entry.level}`}>
              [{entry.timestamp}] {entry.message}
            </div>
          ))
        )}
      </div>
    </div>
  );
}
