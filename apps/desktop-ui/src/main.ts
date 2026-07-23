import { createPinia } from "pinia";
import { createApp } from "vue";
import MainWindow from "@/windows/MainWindow.vue";
import MiniWindow from "@/windows/MiniWindow.vue";
import { initializeFontSize } from "@/features/preferences/fontSize";
import "@/theme/tokens.css";
import "@/theme/global.css";

const params = new URLSearchParams(window.location.search);
const windowKind = params.get("window") === "mini" ? "mini" : "main";
const root = windowKind === "mini" ? MiniWindow : MainWindow;
const detectedPlatform = /Macintosh|Mac OS X/.test(navigator.userAgent)
  ? "macos"
  : /Windows/.test(navigator.userAgent)
    ? "windows"
    : "other";
const previewPlatform = import.meta.env.DEV ? params.get("platform") : null;

document.documentElement.dataset.window = windowKind;
initializeFontSize();
document.documentElement.dataset.platform = previewPlatform === "windows" || previewPlatform === "macos"
  ? previewPlatform
  : detectedPlatform;

createApp(root).use(createPinia()).mount("#app");
