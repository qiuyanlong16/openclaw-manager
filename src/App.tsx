import { useState, useCallback } from "react";
import EnvironmentCheck from "./components/EnvironmentCheck";
import ActionButtons from "./components/ActionButtons";
import GatewayStatus from "./components/GatewayStatus";
import LogViewer from "./components/LogViewer";
import Settings from "./components/Settings";
import { useLogListener } from "./hooks/useLogListener";
import "./App.css";

function App() {
  const { logs, isDeploying, setIsDeploying, isUninstalling, setIsUninstalling } =
    useLogListener();
  const [showSettings, setShowSettings] = useState(false);
  const [refreshKey, setRefreshKey] = useState(0);

  const refreshEnv = useCallback(() => {
    // Brief delay to ensure backend file writes have settled
    setTimeout(() => setRefreshKey((k) => k + 1), 800);
  }, []);

  return (
    <div className="app">
      <header className="app-header">
        <div className="logo">
          <img src="/favicon.svg" alt="" />
        </div>
        <h1>OpenClaw Manager</h1>
        <div className="header-spacer" />
        <button className="btn-settings" onClick={() => setShowSettings(true)} title="设置">
          ⚙
        </button>
      </header>
      <div className="content">
        <EnvironmentCheck key={refreshKey} />
        <ActionButtons
          isDeploying={isDeploying}
          isUninstalling={isUninstalling}
          onDeployStart={() => setIsDeploying(true)}
          onDeployEnd={(success) => {
            setIsDeploying(false);
            if (success) refreshEnv();
          }}
          onUninstallStart={() => setIsUninstalling(true)}
          onUninstallEnd={(success) => {
            setIsUninstalling(false);
            if (success) refreshEnv();
          }}
        />
        <GatewayStatus />
        <LogViewer logs={logs} />
      </div>
      {showSettings && <Settings onClose={() => setShowSettings(false)} />}
    </div>
  );
}

export default App;
