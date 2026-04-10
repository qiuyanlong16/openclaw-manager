import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface EnvStatus {
  ok: boolean;
  version: string | null;
}

interface EnvCheckData {
  node: EnvStatus;
  git: EnvStatus;
  openclaw: EnvStatus;
}

export default function EnvironmentCheck() {
  const [env, setEnv] = useState<EnvCheckData | null>(null);

  useEffect(() => {
    invoke<EnvCheckData>("check_environment").then(setEnv).catch(console.error);
  }, []);

  if (!env) return null;

  return (
    <div className="card">
      <h2>环境检测</h2>
      <div className="env-list">
        <div className="env-item">
          <span className={`status-dot ${env.node.ok ? "ok" : "error"}`} />
          <span>
            Node.js {env.node.ok ? `${env.node.version}` : "未安装 (需要 >= 22)"}
          </span>
        </div>
        <div className="env-item">
          <span className={`status-dot ${env.git.ok ? "ok" : "error"}`} />
          <span>
            Git {env.git.ok ? `${env.git.version}` : "未安装"}
          </span>
        </div>
        <div className="env-item">
          <span className={`status-dot ${env.openclaw.ok ? "ok" : "error"}`} />
          <span>
            OpenClaw {env.openclaw.ok ? `${env.openclaw.version}` : "未安装"}
          </span>
        </div>
      </div>
    </div>
  );
}
