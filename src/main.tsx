import { StrictMode } from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";

import App from "./App";
import { isTauri } from "./lib/events";
import "./styles.css";

const windowLabel = isTauri
  ? getCurrentWindow().label
  : (new URLSearchParams(window.location.search).get("window") ?? "popup");

document.documentElement.dataset.window = windowLabel;

if (!isTauri) {
  // Browser-only preview framing, so the windows appear at their real Tauri sizes.
  document.documentElement.dataset.preview = "true";
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <StrictMode>
    <App windowLabel={windowLabel} />
  </StrictMode>,
);
