import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import WeixinQrModal from "./WeixinQrModal";

interface WeixinConfig {
  pluginInstalled: boolean;
  enabled: boolean;
  connected: boolean;
  accountId: string | null;
}

export default function Channels() {
  const [config, setConfig] = useState<WeixinConfig | null>(null);
  const [installing, setInstalling] = useState(false);
  const [disconnecting, setDisconnecting] = useState(false);
  const [showQr, setShowQr] = useState(false);
  const [showBindCode, setShowBindCode] = useState(false);
  const [bindCode, setBindCode] = useState<string | null>(null);
  const [toast, setToast] = useState<{ ok: boolean; msg: string } | null>(null);
  const configRef = useRef<WeixinConfig | null>(null);
  configRef.current = config;

  useEffect(() => {
    let cancelled = false;
    async function fetchConfig() {
      try {
        const data = await invoke<WeixinConfig>("get_weixin_config");
        if (!cancelled) {
          setConfig(data);
          // Auto-show QR if plugin is installed but not connected
          if (data.pluginInstalled && !data.connected) {
            setShowQr(true);
          }
        }
      } catch (e) {
        console.error("Failed to fetch weixin config:", e);
      }
    }
    fetchConfig();
    return () => { cancelled = true; };
  }, []);

  async function handleInstall() {
    setInstalling(true);
    try {
      const result = await invoke<{ success: boolean }>("install_weixin_plugin");
      if (result.success) {
        setToast({ ok: true, msg: "微信插件安装成功" });
        setTimeout(() => setToast(null), 3000);
        // Trust the install result - force pluginInstalled to true
        setConfig(prev => prev ? { ...prev, pluginInstalled: true } : {
          pluginInstalled: true,
          enabled: false,
          connected: false,
          accountId: null,
        });
        // Auto-show QR after install
        setShowQr(true);
      } else {
        setToast({ ok: false, msg: "安装失败" });
      }
    } catch (e) {
      setToast({ ok: false, msg: `安装失败: ${e instanceof Error ? e.message : String(e)}` });
      setTimeout(() => setToast(null), 5000);
    } finally {
      setInstalling(false);
    }
  }

  async function handleDisconnect() {
    setDisconnecting(true);
    try {
      await invoke("disconnect_weixin");
      setToast({ ok: true, msg: "微信连接已断开" });
      setTimeout(() => setToast(null), 3000);
      const data = await invoke<WeixinConfig>("get_weixin_config");
      setConfig(data);
    } catch (e) {
      setToast({ ok: false, msg: `断开失败: ${e instanceof Error ? e.message : String(e)}` });
      setTimeout(() => setToast(null), 3000);
    } finally {
      setDisconnecting(false);
    }
  }

  async function handleConnected() {
    setShowQr(false);
    invoke<WeixinConfig>("get_weixin_config")
      .then((data) => setConfig(data))
      .catch(console.error);
  }

  async function handleViewBindCode() {
    try {
      const result = await invoke<{ qrcode: string; qrcodeImgContent: string; message: string }>(
        "start_weixin_qr_login"
      );
      setBindCode(result.qrcodeImgContent);
      setShowBindCode(true);
    } catch (e) {
      setToast({ ok: false, msg: `获取绑定码失败: ${e instanceof Error ? e.message : String(e)}` });
      setTimeout(() => setToast(null), 5000);
    }
  }

  if (!config) {
    return (
      <div className="skills-loading">
        <div className="spinner" />
        <span>加载 Channels 配置...</span>
      </div>
    );
  }

  return (
    <div>
      <h2>Channels</h2>
      <p className="settings-desc">管理外部消息渠道连接。</p>

      {toast && (
        <div className={`channel-toast ${toast.ok ? "success" : "error"}`}>
          {toast.msg}
        </div>
      )}

      <div className="channel-card">
        <div className="channel-card-header">
          <div className="channel-card-info">
            <span className="channel-icon">💬</span>
            <div className="channel-details">
              <div className="channel-name-row">
                <span className="channel-name">微信</span>
                <span className={`channel-status-badge ${
                  config.connected ? "connected" : config.pluginInstalled ? "disconnected" : "not-installed"
                }`}>
                  {config.connected ? "已连接" : config.pluginInstalled ? "未连接" : "未安装"}
                </span>
              </div>
              <p className="channel-desc">
                通过微信插件连接微信消息平台，支持私聊对话。
              </p>
              {config.connected && config.accountId && (
                <span className="channel-account-id">账号: {config.accountId}</span>
              )}
            </div>
          </div>
          <div className="channel-actions">
            {!config.pluginInstalled && (
              <button
                className="btn btn-small btn-primary"
                onClick={handleInstall}
                disabled={installing}
              >
                {installing ? "安装中..." : "安装插件"}
              </button>
            )}
            {config.pluginInstalled && !config.connected && (
              <button
                className="btn btn-small btn-primary"
                onClick={() => setShowQr(true)}
              >
                连接
              </button>
            )}
            {config.pluginInstalled && config.connected && (
              <>
                <button
                  className="btn btn-small"
                  onClick={handleViewBindCode}
                >
                  查看绑定码
                </button>
                <button
                  className="btn btn-small btn-danger"
                  onClick={handleDisconnect}
                  disabled={disconnecting}
                >
                  {disconnecting ? "断开中..." : "断开"}
                </button>
              </>
            )}
          </div>
        </div>
      </div>

      {showQr && (
        <WeixinQrModal
          onConnected={handleConnected}
          onClose={() => setShowQr(false)}
        />
      )}

      {showBindCode && bindCode && (
        <div className="qr-modal-overlay" onClick={() => setShowBindCode(false)}>
          <div className="qr-modal-content" onClick={(e) => e.stopPropagation()}>
            <div className="qr-modal-header">
              <h2>微信绑定码</h2>
              <button className="qr-modal-close" onClick={() => setShowBindCode(false)}>✕</button>
            </div>
            <div className="qr-modal-body">
              <div className="qr-code-wrapper">
                <img src={bindCode} alt="绑定码" style={{ width: 200, height: 200 }} />
              </div>
              <p className="qr-status-text">使用微信扫描此码以重新绑定</p>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
