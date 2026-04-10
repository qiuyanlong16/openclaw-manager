import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

export default function GatewayStatus() {
  const [running, setRunning] = useState<boolean | null>(null);
  const [loading, setLoading] = useState(false);

  const fetchStatus = useCallback(async () => {
    try {
      const result = await invoke<{ running: boolean }>("get_gateway_status");
      setRunning(result.running);
    } catch {
      setRunning(false);
    }
  }, []);

  useEffect(() => {
    fetchStatus();
    const interval = setInterval(fetchStatus, 5000);
    return () => clearInterval(interval);
  }, [fetchStatus]);

  async function handleStart() {
    setLoading(true);
    try {
      await invoke("start_gateway");
      setRunning(true);
    } catch (e) {
      console.error("Failed to start gateway:", e);
    } finally {
      setLoading(false);
    }
  }

  async function handleStop() {
    setLoading(true);
    try {
      await invoke("stop_gateway");
      setRunning(false);
    } catch (e) {
      console.error("Failed to stop gateway:", e);
    } finally {
      setLoading(false);
    }
  }

  async function handleOpenBrowser() {
    try {
      await invoke("open_url", { url: "http://127.0.0.1:18789/" });
    } catch (e) {
      console.error("Failed to open browser:", e);
    }
  }

  return (
    <div className="card">
      <h2>网关状态</h2>
      <div className="gateway-status">
        <div className="gateway-info">
          <span
            className={`status-dot ${
              running === null ? "" : running ? "ok" : "error"
            }`}
          />
          <span>
            {running === null
              ? "检测中..."
              : running
                ? "运行中"
                : "已停止"}
          </span>
        </div>
        <div className="gateway-actions">
          <button
            className="btn btn-small btn-primary"
            onClick={handleOpenBrowser}
            disabled={!running}
            title="在浏览器中打开 OpenClaw"
          >
            打开浏览器
          </button>
          <button
            className="btn btn-small btn-primary"
            onClick={handleStart}
            disabled={loading || running === true}
          >
            启动
          </button>
          <button
            className="btn btn-small btn-danger"
            onClick={handleStop}
            disabled={loading || running !== true}
          >
            停止
          </button>
        </div>
      </div>
    </div>
  );
}
