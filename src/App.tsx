import { useState, useCallback, useEffect } from "react";
import EnvironmentCheck from "./components/EnvironmentCheck";
import ActionButtons from "./components/ActionButtons";
import GatewayStatus from "./components/GatewayStatus";
import LogViewer from "./components/LogViewer";
import Settings from "./components/Settings";
import { useLogListener } from "./hooks/useLogListener";
import "./App.css";

function AppLoading() {
  return (
    <div className="app-loading">
      <div className="app-loading__content">
        <img src="/favicon.svg" alt="" className="app-loading__logo" />
        <div className="app-loading__dots">
          <span className="dot dot-1" />
          <span className="dot dot-2" />
          <span className="dot dot-3" />
        </div>
        <p className="app-loading__text">正在启动...</p>
      </div>
    </div>
  );
}

function App() {
  const { logs, isDeploying, setIsDeploying, isUninstalling, setIsUninstalling } =
    useLogListener();
  const [showSettings, setShowSettings] = useState(false);
  const [refreshKey, setRefreshKey] = useState(0);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const timer = setTimeout(() => setLoading(false), 1200);
    return () => clearTimeout(timer);
  }, []);

  const refreshEnv = useCallback(() => {
    setTimeout(() => setRefreshKey((k) => k + 1), 800);
  }, []);

  if (loading) {
    return <AppLoading />;
  }

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
