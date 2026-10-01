import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";

interface Entry {
  id: string;
  video: string;
  template: string;
  status: string;
  started_at: string;
  elapsed_sec: number;
  output_dir: string;
  rally_count: number | null;
}

export async function renderHistory(root: HTMLElement) {
  const items = await invoke<Entry[]>("list_history");
  root.innerHTML = `
    <div class="hist-grid">
      <ul id="h-list" class="hist-list">${
        items.length
          ? items
              .map(
                (e) => `<li class="hist-item" data-out="${e.output_dir}" data-video="${e.video}">
          <span class="hist-status s-${e.status}">${e.status}</span>
          <span class="hist-name">${e.video.split(/[\\/]/).pop()}</span>
          <span class="hist-meta mono">${e.started_at ? new Date(Number(e.started_at) * 1000).toLocaleString() : "—"} · ${e.elapsed_sec.toFixed(0)}s · rally ${e.rally_count ?? "—"}</span>
        </li>`,
              )
              .join("")
          : `<li class="hist-empty">Belum ada run — jalankan analisis di tab Run</li>`
      }</ul>
      <div id="h-detail" class="hist-detail card"></div>
    </div>`;
  root.querySelectorAll("#h-list li.hist-item").forEach((li) => {
    li.addEventListener("click", async () => {
      const out = (li as HTMLElement).dataset.out!;
      const name = (li as HTMLElement).dataset.video!.split(/[\\/]/).pop()!.replace(/\.[^.]+$/, "");
      const detail = root.querySelector("#h-detail") as HTMLElement;
      if (!out) return;
      const src = convertFileSrc(`${out}/detect_${name}.mp4`);
      detail.innerHTML = `<video class="hist-video" controls src="${src}"></video>`;
    });
  });
}
