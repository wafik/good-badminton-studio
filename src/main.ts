import "./styles.css";
import "@fontsource/ibm-plex-sans/400.css";
import "@fontsource/ibm-plex-sans/500.css";
import "@fontsource/ibm-plex-sans/600.css";
import "@fontsource/ibm-plex-mono/400.css";
import "@fontsource/ibm-plex-mono/500.css";
import { renderSetup } from "./setup";
import { renderCorners } from "./corners";
// renderHistory menyusul di Task 8 (import bertahap)

const views: Record<string, (el: HTMLElement) => void> = {
  setup: renderSetup,
  corners: renderCorners,
  // history ditambahkan di task berikutnya
};
function show(name: string) {
  document.querySelectorAll("main section").forEach((s) => ((s as HTMLElement).hidden = s.id !== `view-${name}`));
  document.querySelectorAll("nav button").forEach((b) =>
    (b as HTMLButtonElement).classList.toggle("active", (b as HTMLButtonElement).dataset.view === name));
  const el = document.getElementById(`view-${name}`);
  if (el && views[name]) views[name](el);
}
document.querySelectorAll("nav button").forEach((b) =>
  (b as HTMLButtonElement).addEventListener("click", () => show((b as HTMLButtonElement).dataset.view!)));
show("setup");
