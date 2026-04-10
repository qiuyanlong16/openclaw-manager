import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import Skills from "./Skills";
import Channels from "./Channels";

type Section = "model" | "skills" | "channels";

const providers = [
  { id: "anthropic", label: "Anthropic", models: ["claude-sonnet-4-5-20250929", "claude-opus-4-5-20251101", "claude-haiku-4-5-20251001", "claude-sonnet-4-20250514", "claude-3-5-sonnet-20241022"] },
  { id: "kimi", label: "Kimi", models: ["moonshot-v1-8k", "moonshot-v1-32k", "moonshot-v1-128k"] },
  { id: "openai", label: "OpenAI", models: ["gpt-4o", "gpt-4o-mini", "o1", "o3-mini"] },
  { id: "google", label: "Google", models: ["gemini-2.5-pro", "gemini-2.5-flash", "gemini-2.0-flash"] },
  { id: "custom", label: "Custom", models: [] },
];

interface ModelConfig {
  provider: string;
  apiKey: string;
  model: string;
  customBaseUrl?: string;
}

interface SettingsProps {
  onClose: () => void;
}

export default function Settings({ onClose }: SettingsProps) {
  const [activeSection, setActiveSection] = useState<Section>("model");
  const [config, setConfig] = useState<ModelConfig>({
    provider: "anthropic",
    apiKey: "",
    model: "claude-sonnet-4-5-20250929",
    customBaseUrl: "",
  });
  const [saving, setSaving] = useState(false);
  const [toast, setToast] = useState<{ ok: boolean; msg: string } | null>(null);
  const [showKey, setShowKey] = useState(false);
  const [tabLoading, setTabLoading] = useState(false);

  useEffect(() => {
    invoke<Partial<ModelConfig>>("get_model_config")
      .then((data) => {
        if (data && data.provider) {
          setConfig((prev) => ({ ...prev, ...data }));
        }
      })
      .catch(console.error);
  }, []);

  function switchTab(tab: Section) {
    if (tab === activeSection) return;
    setTabLoading(tab === "skills");
    setActiveSection(tab);
    if (tab === "skills") {
      setTimeout(() => setTabLoading(false), 500);
    }
  }

  async function handleSave() {
    if (config.provider === "custom" && (!config.customBaseUrl || !config.customBaseUrl.trim())) {
      setToast({ ok: false, msg: "保存失败: Custom provider 必须填写 Base URL" });
      return;
    }
    setSaving(true);
    setToast(null);
    try {
      await invoke("set_model_config", {
        provider: config.provider,
        apiKey: config.apiKey,
        model: config.model,
        customBaseUrl: config.provider === "custom" ? config.customBaseUrl : undefined,
      });
      setToast({ ok: true, msg: "配置已保存 ✓" });
      setTimeout(() => setToast(null), 3000);
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      setToast({ ok: false, msg: `保存失败: ${msg}` });
    } finally {
      setSaving(false);
    }
  }

  const currentProvider = providers.find((p) => p.id === config.provider) ?? providers[0];

  return (
    <div className="settings-overlay" onClick={onClose}>
      <div className="settings-panel" onClick={(e) => e.stopPropagation()}>
        <div className="settings-header">
          <span className="settings-header-left">
            <span className="settings-header-icon">⚙</span>
            <h1>设置</h1>
          </span>
          <button className="settings-close" onClick={onClose}>✕</button>
        </div>
        <div className="settings-body">
          <div className="settings-sidebar">
            <button
              className={`settings-nav ${activeSection === "model" ? "active" : ""}`}
              onClick={() => switchTab("model")}
            >
              <span className="nav-icon">⚙</span> Model
            </button>
            <button
              className={`settings-nav ${activeSection === "skills" ? "active" : ""}`}
              onClick={() => switchTab("skills")}
            >
              <span className="nav-icon">🧩</span> Skills
            </button>
            <button
              className={`settings-nav ${activeSection === "channels" ? "active" : ""}`}
              onClick={() => switchTab("channels")}
            >
              <span className="nav-icon">📡</span> Channels
            </button>
          </div>
          <div className="settings-content">
            {activeSection === "model" && (
              <div>
                <h2>Model Configuration</h2>
                <p className="settings-desc">Change your LLM provider, API key, or model.</p>

                <div className="provider-tabs">
                  {providers.map((p) => (
                    <button
                      key={p.id}
                      className={`provider-tab ${config.provider === p.id ? "active" : ""}`}
                      onClick={() => {
                        setConfig((prev) => ({
                          ...prev,
                          provider: p.id,
                          model: p.models.length > 0 ? p.models[0] : "",
                        }));
                      }}
                    >
                      {p.label}
                    </button>
                  ))}
                </div>

                <div className="form-group">
                  <label>
                    API Key
                    <span className="get-key-link">
                      {config.provider === "anthropic" && (
                        <a href="https://console.anthropic.com/settings/keys" target="_blank" rel="noopener noreferrer" onClick={(e) => e.stopPropagation()}>
                          Get API Key →
                        </a>
                      )}
                      {config.provider === "openai" && (
                        <a href="https://platform.openai.com/api-keys" target="_blank" rel="noopener noreferrer" onClick={(e) => e.stopPropagation()}>
                          Get API Key →
                        </a>
                      )}
                      {config.provider === "google" && (
                        <a href="https://makersuite.google.com/app/apikey" target="_blank" rel="noopener noreferrer" onClick={(e) => e.stopPropagation()}>
                          Get API Key →
                        </a>
                      )}
                    </span>
                  </label>
                  <div className="input-row">
                    <input
                      type={showKey ? "text" : "password"}
                      value={config.apiKey}
                      placeholder={config.provider === "anthropic" ? "sk-ant-..." : `Enter your ${currentProvider.label} API key`}
                      onChange={(e) => setConfig((prev) => ({ ...prev, apiKey: e.target.value }))}
                    />
                    <button
                      className="toggle-visibility"
                      type="button"
                      onClick={() => setShowKey(!showKey)}
                      title={showKey ? "Hide" : "Show"}
                    >
                      {showKey ? "🙈" : "👁"}
                    </button>
                  </div>
                </div>

                <div className="form-group">
                  <label>Model</label>
                  {currentProvider.models.length > 0 ? (
                    <select
                      value={config.model}
                      onChange={(e) => setConfig((prev) => ({ ...prev, model: e.target.value }))}
                    >
                      {currentProvider.models.map((m) => (
                        <option key={m} value={m}>{m}</option>
                      ))}
                    </select>
                  ) : (
                    <input
                      type="text"
                      value={config.model}
                      placeholder="Enter model name (e.g. qwen-plus)"
                      onChange={(e) => setConfig((prev) => ({ ...prev, model: e.target.value }))}
                    />
                  )}
                </div>

                {config.provider === "custom" && (
                  <div className="form-group">
                    <label>Base URL</label>
                    <input
                      type="text"
                      value={config.customBaseUrl || ""}
                      placeholder="https://api.example.com/v1"
                      onChange={(e) => setConfig((prev) => ({ ...prev, customBaseUrl: e.target.value }))}
                    />
                  </div>
                )}

                <div className="form-actions">
                  {toast && <span className={`save-status ${toast.ok ? "" : "error"}`}>{toast.msg}</span>}
                  <button className="btn btn-primary" onClick={handleSave} disabled={saving}>
                    {saving ? "保存中..." : "保存配置"}
                  </button>
                </div>
              </div>
            )}
            {activeSection === "skills" && (
              <div style={{ position: "relative", width: "100%" }}>
                {tabLoading && (
                  <div className="skills-loading-overlay">
                    <div className="spinner" />
                    <span>正在加载 Skills 列表...</span>
                  </div>
                )}
                <Skills />
              </div>
            )}
            {activeSection === "channels" && <Channels />}
          </div>
        </div>
      </div>
    </div>
  );
}
