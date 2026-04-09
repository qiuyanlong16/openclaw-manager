import EnvironmentCheck from "./components/EnvironmentCheck";
import "./App.css";

function App() {
  return (
    <div className="app">
      <header className="app-header">
        <div className="logo" />
        <h1>OpenClaw Manager</h1>
      </header>
      <div className="content">
        <EnvironmentCheck />
      </div>
    </div>
  );
}

export default App;
