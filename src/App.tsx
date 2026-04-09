import EnvironmentCheck from "./components/EnvironmentCheck";
import ActionButtons from "./components/ActionButtons";
import GatewayStatus from "./components/GatewayStatus";
import LogViewer from "./components/LogViewer";
import { useLogListener } from "./hooks/useLogListener";
import "./App.css";

function App() {
  const { logs, isDeploying, setIsDeploying, isUninstalling, setIsUninstalling } =
    useLogListener();

  return (
    <div className="app">
      <header className="app-header">
        <div className="logo" />
        <h1>OpenClaw Manager</h1>
      </header>
      <div className="content">
        <EnvironmentCheck />
        <ActionButtons
          isDeploying={isDeploying}
          isUninstalling={isUninstalling}
          onDeployStart={() => setIsDeploying(true)}
          onDeployEnd={(success) => {
            setIsDeploying(false);
          }}
          onUninstallStart={() => setIsUninstalling(true)}
          onUninstallEnd={(success) => {
            setIsUninstalling(false);
          }}
        />
        <GatewayStatus />
        <LogViewer logs={logs} />
      </div>
    </div>
  );
}

export default App;
