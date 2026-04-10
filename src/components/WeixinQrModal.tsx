import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { QRCodeCanvas } from "qrcode.react";

interface WeixinQrModalProps {
  onConnected: () => void;
  onClose: () => void;
}

export default function WeixinQrModal({ onConnected, onClose }: WeixinQrModalProps) {
  const [qrText, setQrText] = useState("");
  const [qrcode, setQrcode] = useState("");
  const [status, setStatus] = useState<"wait" | "scaned" | "confirmed" | "expired" | "success" | "error">("wait");
  const [message, setMessage] = useState("正在获取二维码...");
  const countdownRef = useRef(0);
  const [countdownDisplay, setCountdownDisplay] = useState("5:00");
  const pollRef = useRef<number | null>(null);
  const timerRef = useRef<number | null>(null);
  const abortRef = useRef(false);
  const onConnectedRef = useRef(onConnected);
  onConnectedRef.current = onConnected;

  const startLogin = async () => {
    abortRef.current = true;
    if (pollRef.current) clearTimeout(pollRef.current);
    if (timerRef.current) clearInterval(timerRef.current);
    abortRef.current = false;
    countdownRef.current = 300;
    setCountdownDisplay("5:00");

    try {
      const result = await invoke<{ qrcode: string; qrcodeImgContent: string; message: string }>(
        "start_weixin_qr_login"
      );
      setQrcode(result.qrcode);
      setQrText(result.qrcodeImgContent);
      setMessage(result.message);
      setStatus("wait");
    } catch (e) {
      setStatus("error");
      setMessage(`获取二维码失败: ${e instanceof Error ? e.message : String(e)}`);
    }
  };

  useEffect(() => {
    startLogin();
    return () => {
      abortRef.current = true;
      if (pollRef.current) clearTimeout(pollRef.current);
      if (timerRef.current) clearInterval(timerRef.current);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Countdown timer (only updates display state)
  useEffect(() => {
    if (status !== "wait") return;
    timerRef.current = window.setInterval(() => {
      countdownRef.current -= 1;
      if (countdownRef.current <= 0) {
        if (timerRef.current) clearInterval(timerRef.current);
        setStatus("expired");
        setMessage("二维码已过期，请点击重试");
        setCountdownDisplay("0:00");
        return;
      }
      const m = Math.floor(countdownRef.current / 60);
      const s = countdownRef.current % 60;
      setCountdownDisplay(`${m}:${s.toString().padStart(2, "0")}`);
    }, 1000);
    return () => {
      if (timerRef.current) clearInterval(timerRef.current);
    };
  }, [status]);

  // Poll QR status
  useEffect(() => {
    if (status !== "wait" || !qrcode) return;

    async function poll() {
      if (abortRef.current) return;
      try {
        const result = await invoke<{
          status: string;
          botToken?: string;
          accountId?: string;
          baseUrl?: string;
          userId?: string;
        }>("poll_weixin_qr_status", { qrcode });

        if (abortRef.current) return;

        if (result.status === "scaned") {
          setStatus("scaned");
          setMessage("已扫描，请在微信中确认...");
        } else if (result.status === "confirmed" && result.accountId && result.botToken) {
          setStatus("success");
          setMessage("连接成功！正在保存...");
          try {
            await invoke("save_weixin_login_result", {
              result: {
                status: result.status,
                botToken: result.botToken,
                accountId: result.accountId,
                baseUrl: result.baseUrl,
                userId: result.userId,
              },
            });
            onConnectedRef.current();
          } catch (e) {
            setStatus("error");
            setMessage(`保存凭据失败: ${e instanceof Error ? e.message : String(e)}`);
          }
          return;
        } else if (result.status === "expired") {
          setStatus("expired");
          setMessage("二维码已过期，请点击重试");
          if (timerRef.current) clearInterval(timerRef.current);
          return;
        }
      } catch {
        // ignore polling errors
      }

      if (!abortRef.current) {
        pollRef.current = window.setTimeout(poll, 5000);
      }
    }

    pollRef.current = window.setTimeout(poll, 3000);
    return () => {
      if (pollRef.current) clearTimeout(pollRef.current);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [status, qrcode]);

  return (
    <div className="qr-modal-overlay" onClick={onClose}>
      <div className="qr-modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="qr-modal-header">
          <h2>微信绑定</h2>
          <button className="qr-modal-close" onClick={onClose}>✕</button>
        </div>
        <div className="qr-modal-body">
          {status === "error" || status === "expired" ? (
            <div className="qr-error">
              <span className="qr-error-icon">⚠</span>
              <p>{message}</p>
              <button className="btn btn-primary" onClick={startLogin}>
                重新获取二维码
              </button>
            </div>
          ) : qrText ? (
            <>
              <div className="qr-code-wrapper">
                <QRCodeCanvas value={qrText} size={200} level="M" />
              </div>
              <p className="qr-status-text">{message}</p>
              <p className="qr-countdown">{countdownDisplay}</p>
            </>
          ) : (
            <div className="qr-loading-qr">
              <div className="spinner" />
              <span>正在获取二维码...</span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
