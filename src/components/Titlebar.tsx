import { getCurrentWindow } from "@tauri-apps/api/window";

const isTauri = "__TAURI_INTERNALS__" in window;

export default function Titlebar() {
  const handleMinimize = () => {
    if (isTauri) getCurrentWindow().minimize();
  };

  const handleClose = () => {
    if (isTauri) getCurrentWindow().hide();
  };

  return (
    <div data-tauri-drag-region className="titlebar">
      <div className="titlebar-btns">
        <button onClick={handleMinimize} className="titlebar-btn">
          <svg width="10" height="2" viewBox="0 0 10 2">
            <rect width="10" height="1.5" rx="0.75" fill="currentColor" />
          </svg>
        </button>
        <button onClick={handleClose} className="titlebar-btn titlebar-btn-close">
          <svg width="10" height="10" viewBox="0 0 10 10">
            <line x1="1.5" y1="1.5" x2="8.5" y2="8.5" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" />
            <line x1="8.5" y1="1.5" x2="1.5" y2="8.5" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" />
          </svg>
        </button>
      </div>
    </div>
  );
}
