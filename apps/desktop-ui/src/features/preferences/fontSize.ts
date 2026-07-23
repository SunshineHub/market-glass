export const FONT_SIZE_STORAGE_KEY = "market-glass-font-size";

export const fontSizeOptions = [
  { value: "small", label: "小", description: "紧凑显示" },
  { value: "standard", label: "标准", description: "清晰均衡" },
  { value: "large", label: "大", description: "阅读优先" },
] as const;

export type FontSizePreference = (typeof fontSizeOptions)[number]["value"];

function isFontSizePreference(value: string | null): value is FontSizePreference {
  return fontSizeOptions.some((option) => option.value === value);
}

export function readFontSize(): FontSizePreference {
  try {
    const value = window.localStorage.getItem(FONT_SIZE_STORAGE_KEY);
    return isFontSizePreference(value) ? value : "standard";
  } catch {
    return "standard";
  }
}

export function applyFontSize(value: FontSizePreference) {
  document.documentElement.dataset.fontSize = value;
}

export function setFontSize(value: FontSizePreference) {
  applyFontSize(value);
  try {
    window.localStorage.setItem(FONT_SIZE_STORAGE_KEY, value);
  } catch {
    // The active window still receives the preference when storage is unavailable.
  }
}

export function initializeFontSize() {
  applyFontSize(readFontSize());
  window.addEventListener("storage", (event) => {
    if (event.key !== FONT_SIZE_STORAGE_KEY) return;
    applyFontSize(isFontSizePreference(event.newValue) ? event.newValue : "standard");
  });
}
