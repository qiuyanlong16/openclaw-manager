import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface ActionButtonsProps {
  isDeploying: boolean;
  isUninstalling: boolean;
  onDeployStart: () => void;
  onDeployEnd: (success: boolean) => void;
  onUninstallStart: () => void;
  onUninstallEnd: (success: boolean) => void;
}

export default function ActionButtons({
  isDeploying,
  isUninstalling,
  onDeployStart,
  onDeployEnd,
  onUninstallStart,
  onUninstallEnd,
}: ActionButtonsProps) {
  const [isInstalled, setIsInstalled] = useState(false);

  useEffect(() => {
    invoke<boolean>("is_openclaw_installed")
      .then(setIsInstalled)
      .catch(() => setIsInstalled(false));
  }, []);

  useEffect(() => {
    if (!isDeploying && !isUninstalling) {
      invoke<boolean>("is_openclaw_installed")
        .then(setIsInstalled)
        .catch(() => setIsInstalled(false));
    }
  }, [isDeploying, isUninstalling]);

  async function handleDeploy() {
    onDeployStart();
    try {
      const result = await invoke<{ success: boolean; error?: string }>(
        "deploy_openclaw"
      );
      setIsInstalled(result.success);
      onDeployEnd(result.success);
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      console.error("Deploy failed:", msg);
      onDeployEnd(false);
    }
  }

  async function handleUninstall() {
    if (!window.confirm("确定要卸载 OpenClaw 吗？此操作将删除所有配置和数据。")) {
      return;
    }
    onUninstallStart();
    try {
      const result = await invoke<{ success: boolean; error?: string }>(
        "uninstall_openclaw"
      );
      setIsInstalled(false);
      onUninstallEnd(result.success);
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      console.error("Uninstall failed:", msg);
      onUninstallEnd(false);
    }
  }

  return (
    <div className="card">
      <h2>操作</h2>
      <div className="action-buttons">
        <button
          className="btn btn-primary"
          onClick={handleDeploy}
          disabled={isDeploying || isUninstalling || isInstalled}
        >
          {isDeploying ? "部署中..." : "一键部署"}
        </button>
        <button
          className="btn btn-danger"
          onClick={handleUninstall}
          disabled={isDeploying || isUninstalling || !isInstalled}
        >
          {isUninstalling ? "卸载中..." : "一键卸载"}
        </button>
      </div>
    </div>
  );
}
