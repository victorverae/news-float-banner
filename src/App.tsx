import {
  useCallback,
  useEffect,
  useMemo,
  useState,
  type CSSProperties,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { PhysicalPosition, PhysicalSize } from "@tauri-apps/api/dpi";
import { currentMonitor, getCurrentWindow } from "@tauri-apps/api/window";
import { openUrl } from "@tauri-apps/plugin-opener";
import "./App.css";

const TICKER_HEIGHT_PX = 200;
const POLL_MS = 5 * 60 * 1000;

type NewsState = "breaking" | "alert" | "economy" | "conflict" | "normal";

interface NewsItem {
  title: string;
  link: string;
  state: NewsState;
  imageUrl?: string | null;
}

function NewsThumb({ url }: { url: string | null | undefined }) {
  const [ok, setOk] = useState(!!url);
  if (!url || !ok) {
    return <div className="pill__thumb pill__thumb--placeholder" aria-hidden />;
  }
  return (
    <img
      className="pill__thumb"
      src={url}
      alt=""
      loading="lazy"
      decoding="async"
      referrerPolicy="no-referrer"
      onError={() => setOk(false)}
    />
  );
}

function stateClass(s: NewsState): string {
  switch (s) {
    case "breaking":
      return "pill--breaking";
    case "alert":
      return "pill--alert";
    case "economy":
      return "pill--economy";
    case "conflict":
      return "pill--conflict";
    default:
      return "pill--normal";
  }
}

function stateLabel(s: NewsState): string {
  switch (s) {
    case "breaking":
      return "Última hora";
    case "alert":
      return "Alerta";
    case "economy":
      return "Economía";
    case "conflict":
      return "Conflicto";
    default:
      return "Mundo";
  }
}

async function applyTopBarGeometry(): Promise<void> {
  const win = getCurrentWindow();
  const monitor = await currentMonitor();
  if (!monitor) return;
  const { position, size } = monitor.workArea;
  await win.setPosition(new PhysicalPosition(position.x, position.y));
  await win.setSize(new PhysicalSize(size.width, TICKER_HEIGHT_PX));
}

export default function App() {
  const [items, setItems] = useState<NewsItem[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const load = useCallback(async () => {
    try {
      setError(null);
      const data = await invoke<NewsItem[]>("fetch_world_news");
      setItems(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setItems([]);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void applyTopBarGeometry();
    void load();
    const id = window.setInterval(() => void load(), POLL_MS);
    return () => window.clearInterval(id);
  }, [load]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    getCurrentWindow()
      .onScaleChanged(() => {
        void applyTopBarGeometry();
      })
      .then((fn) => {
        unlisten = fn;
      });
    return () => {
      unlisten?.();
    };
  }, []);

  const doubled = useMemo(() => [...items, ...items], [items]);

  const onOpen = async (link: string) => {
    if (!link || link === "#") return;
    try {
      await openUrl(link);
    } catch {
      /* ignore */
    }
  };

  const onMinimize = () => {
    void getCurrentWindow().minimize();
  };

  const onClose = () => {
    void getCurrentWindow().close();
  };

  return (
    <div className="shell">
      <header className="chrome">
        <div className="chrome__drag" data-tauri-drag-region>
          <div className="status">
            {loading && <span className="status__dot" aria-hidden />}
            {error && <span className="status__err">{error}</span>}
          </div>
        </div>
        <div className="chrome__actions">
          <button
            type="button"
            className="chrome__minimize"
            onClick={onMinimize}
            aria-label="Minimizar"
            title="Minimizar"
          >
            <span className="chrome__minimize-icon" aria-hidden />
          </button>
          <button
            type="button"
            className="chrome__close"
            onClick={onClose}
            aria-label="Cerrar"
            title="Cerrar"
          >
            <span className="chrome__close-icon" aria-hidden />
          </button>
        </div>
      </header>

      <div className="viewport" role="marquee" aria-live="polite">
        {loading && items.length === 0 ? (
          <p className="loading-msg">Cargando noticias…</p>
        ) : (
          <div
            className={`track ${items.length === 0 && !loading ? "track--empty" : ""}`}
            style={
              {
                "--segments": Math.max(1, items.length),
              } as CSSProperties
            }
          >
            {doubled.map((item, i) => (
              <button
                type="button"
                key={`${item.link}-${item.title}-${i}`}
                className={`pill ${stateClass(item.state)} ${
                  item.imageUrl ? "pill--has-image" : ""
                }`}
                onClick={() => void onOpen(item.link)}
                title={item.link}
              >
                <NewsThumb url={item.imageUrl} />
                <span className="pill__main">
                  <span className="pill__badge">{stateLabel(item.state)}</span>
                  <span className="pill__title">{item.title}</span>
                </span>
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
