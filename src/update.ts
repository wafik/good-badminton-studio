// In-app update check — offline-first: any failure is a silent no-op besides a
// single console.warn. Notifies once per newer GitHub release via a top banner.
import { getVersion } from "@tauri-apps/api/app";
import { openUrl } from "@tauri-apps/plugin-opener";

const RELEASES_API = "https://api.github.com/repos/wafik/good-badminton-studio/releases/latest";
const RELEASES_PAGE = "https://github.com/wafik/good-badminton-studio/releases/latest";
const DISMISS_KEY = "gb-update-dismissed";

/** Semver-ish compare: strip leading v/V, numeric per segment, missing = 0. */
export function compareVersions(a: string, b: string): number {
  const parts = (s: string) => s.trim().replace(/^[vV]/, "").split(".").map((p) => parseInt(p, 10) || 0);
  const pa = parts(a);
  const pb = parts(b);
  const n = Math.max(pa.length, pb.length);
  for (let i = 0; i < n; i++) {
    const x = pa[i] || 0;
    const y = pb[i] || 0;
    if (x !== y) return x > y ? 1 : -1;
  }
  return 0;
}

function dismissedVersion(): string {
  try {
    return localStorage.getItem(DISMISS_KEY) ?? "";
  } catch {
    return "";
  }
}

async function fetchLatest(): Promise<{ tag: string; url: string } | null> {
  const res = await fetch(RELEASES_API, {
    headers: { Accept: "application/vnd.github+json" },
    signal: AbortSignal.timeout(8000),
  });
  if (!res.ok) {
    console.warn(`[update] GitHub returned ${res.status}`);
    return null;
  }
  const data = (await res.json()) as { tag_name?: unknown; html_url?: unknown };
  const tag = typeof data.tag_name === "string" ? data.tag_name.trim() : "";
  if (!tag) {
    console.warn("[update] release has no tag_name");
    return null;
  }
  const url = typeof data.html_url === "string" && data.html_url ? data.html_url : RELEASES_PAGE;
  return { tag, url };
}

function renderBanner(tag: string, url: string): void {
  if (document.querySelector(".update-banner")) return;
  const banner = document.createElement("div");
  banner.className = "update-banner";
  banner.setAttribute("role", "status");

  const text = document.createElement("span");
  text.append("Versi ");
  const ver = document.createElement("span");
  ver.className = "update-ver";
  ver.textContent = `v${tag.replace(/^[vV]/, "")}`;
  text.append(ver, " tersedia");

  const dl = document.createElement("button");
  dl.type = "button";
  dl.className = "update-dl";
  dl.textContent = "Unduh";
  dl.addEventListener("click", () => {
    void openUrl(url).catch((e) => console.warn("[update] open failed:", e));
  });

  const close = document.createElement("button");
  close.type = "button";
  close.className = "update-close";
  close.setAttribute("aria-label", "Tutup");
  close.textContent = "×";
  close.addEventListener("click", () => {
    try {
      localStorage.setItem(DISMISS_KEY, tag);
    } catch {
      /* storage unavailable — banner still closes for this session */
    }
    banner.remove();
  });

  banner.append(text, dl, close);
  document.querySelector("nav.topbar")?.insertAdjacentElement("afterend", banner);
}

/** Fetch latest release and show the banner when a newer, non-dismissed one exists. */
export async function checkForUpdate(): Promise<void> {
  try {
    const current = await getVersion();
    const latest = await fetchLatest();
    if (!latest) return;
    if (compareVersions(latest.tag, current) <= 0) return;
    if (dismissedVersion() === latest.tag) return;
    renderBanner(latest.tag, latest.url);
  } catch (e) {
    console.warn("[update] check failed:", e);
  }
}

if (import.meta.env.DEV) {
  console.assert(compareVersions("v1.5.0", "1.4.0") > 0, "v1.5.0 > 1.4.0");
  console.assert(compareVersions("1.4.0", "v1.4") === 0, "1.4.0 == v1.4");
  console.assert(compareVersions("1.10.0", "1.9.0") > 0, "1.10.0 > 1.9.0");
  console.assert(compareVersions("2.0.0", "1.99.99") > 0, "2.0.0 > 1.99.99");
}