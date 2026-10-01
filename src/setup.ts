import { open, save } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import { openPath } from "@tauri-apps/plugin-opener";
import { startRun, cancelRun, getDefaults, saveConfig, type RunParams } from "./api";

let t0 = 0;
let listenersBound = false;

export function renderSetup(root: HTMLElement) {
  root.innerHTML = `
    <div class="pane-grid">
      <div class="stack">
        <section class="card">
          <header class="card-head"><h2>Source</h2></header>
          <div class="field">
            <span class="field-label">Video</span>
            <div class="picker">
              <input id="v-video" readonly placeholder="Pilih video pertandingan…" />
              <button id="p-video" class="btn btn-ghost" type="button">Browse</button>
            </div>
          </div>
          <div class="field">
            <span class="field-label">Template lapangan</span>
            <div class="picker">
              <input id="v-template" readonly placeholder="PNG template sudut lapangan…" />
              <button id="p-template" class="btn btn-ghost" type="button">Browse</button>
            </div>
          </div>
        </section>
        <section class="card">
          <header class="card-head"><h2>Output</h2></header>
          <div class="field">
            <span class="field-label">Folder output</span>
            <div class="picker">
              <input id="v-out" readonly placeholder="Folder tujuan hasil analisis…" />
              <button id="p-out" class="btn btn-ghost" type="button">Browse</button>
            </div>
          </div>
          <div class="field">
            <span class="field-label">Anotasi lapangan <em class="opt">opsional</em></span>
            <div class="picker">
              <input id="v-ann" readonly placeholder="court_annotations.txt — kosongkan untuk auto-detect" />
              <button id="p-ann" class="btn btn-ghost" type="button">Browse</button>
            </div>
          </div>
        </section>
        <section class="card">
          <header class="card-head"><h2>Parameters</h2></header>
          <div class="row">
            <label class="switch-row">
              <input type="checkbox" id="v-audio" class="switch" checked />
              <span>Simpan audio</span>
            </label>
            <div class="field field-half">
              <span class="field-label">Bahasa</span>
              <select id="v-lang"><option value="zh">zh</option><option value="en">en</option></select>
            </div>
          </div>
          <fieldset class="params">
            <legend>Display overlay</legend>
            <div class="tog-grid">
              <label class="tog"><input type="checkbox" id="p-skel" checked /><span>Skeleton</span></label>
              <label class="tog"><input type="checkbox" id="p-trail" checked /><span>Player trail</span></label>
              <label class="tog"><input type="checkbox" id="p-court" checked /><span>Court trail</span></label>
              <label class="tog"><input type="checkbox" id="p-shuttle" checked /><span>Shuttle trail</span></label>
              <label class="tog"><input type="checkbox" id="p-stats" checked /><span>Stats panel</span></label>
              <label class="tog"><input type="checkbox" id="p-roi" checked /><span>Pose ROI</span></label>
            </div>
          </fieldset>
          <div class="actions">
            <button id="b-run" class="btn btn-primary" type="button">Run analysis</button>
            <button id="b-cancel" class="btn btn-danger" type="button" disabled>Cancel</button>
            <button id="b-open" class="btn btn-ghost" type="button" hidden>Buka folder output</button>
          </div>
        </section>
      </div>
      <div class="stack">
        <section class="card progress-card">
          <header class="card-head"><h2>Progress</h2><span id="run-status" class="chip" hidden></span></header>
          <progress id="prog" value="0" max="100"></progress>
          <span id="eta" class="mono">—</span>
        </section>
        <section class="card console">
          <header class="card-head"><h2>Console</h2></header>
          <pre id="log" aria-live="polite"></pre>
        </section>
      </div>
    </div>`;

  const pick = async (opts: any) => (await open({ multiple: false, ...opts })) as string | null;
  root.querySelector("#p-video")!.addEventListener("click", async () => {
    const f = await pick({ filters: [{ name: "Video", extensions: ["mp4", "avi", "mov"] }] });
    if (f) (root.querySelector("#v-video") as HTMLInputElement).value = f;
  });
  root.querySelector("#p-template")!.addEventListener("click", async () => {
    const f = await pick({ filters: [{ name: "Image", extensions: ["png", "jpg"] }] });
    if (f) (root.querySelector("#v-template") as HTMLInputElement).value = f;
  });
  root.querySelector("#p-out")!.addEventListener("click", async () => {
    const f = await pick({ directory: true });
    if (f) (root.querySelector("#v-out") as HTMLInputElement).value = f;
  });
  root.querySelector("#p-ann")!.addEventListener("click", async () => {
    const f = await pick({ filters: [{ name: "Anotasi", extensions: ["txt"] }] });
    if (f) (root.querySelector("#v-ann") as HTMLInputElement).value = f;
  });

  // Prefill parameter dari config.json (preferensi user terakhir)
  void getDefaults().then(([cfg]) => {
    const d = cfg?.defaults;
    if (!d) return;
    (root.querySelector("#v-audio") as HTMLInputElement).checked = d.audio;
    (root.querySelector("#v-lang") as HTMLSelectElement).value = d.language;
    const pairs: [string, boolean][] = [
      ["#p-skel", d.show_skeletons],
      ["#p-trail", d.show_player_trajectories],
      ["#p-court", d.show_court_trajectory],
      ["#p-shuttle", d.show_shuttlecock_trajectory],
      ["#p-stats", d.show_player_stats],
      ["#p-roi", d.show_pose_roi],
    ];
    for (const [id, v] of pairs) (root.querySelector(id) as HTMLInputElement).checked = v;
  });

  // listen() global — cukup sekali; handler selalu query DOM via root saat event tiba
  if (!listenersBound) {
    listenersBound = true;
    void listen<{ frame: number; total: number }>("progress", (e) => {
      if (!t0) t0 = Date.now();
      const { frame, total } = e.payload;
      if (total > 0) {
        const bar = root.querySelector("#prog") as HTMLProgressElement;
        bar.value = (frame / total) * 100;
        const rate = frame / ((Date.now() - t0) / 1000);
        const eta = rate > 0 ? Math.round((total - frame) / rate) : 0;
        (root.querySelector("#eta") as HTMLElement).textContent = `${frame}/${total} · ETA ${eta}s`;
      }
    });
    void listen<{ line: string }>("log", (e) => {
      const log = root.querySelector("#log")!;
      log.textContent += e.payload.line + "\n";
      log.scrollTop = log.scrollHeight;
    });
    void listen<{ status: string; output_dir: string }>("run-finished", (e) => {
      (root.querySelector("#b-run") as HTMLButtonElement).disabled = false;
      (root.querySelector("#b-cancel") as HTMLButtonElement).disabled = true;
      const b = root.querySelector("#b-open") as HTMLButtonElement;
      b.hidden = e.payload.status !== "ok";
      b.onclick = () => openPath(e.payload.output_dir);
      // status final di chip durable — #eta hanya untuk frame/ETA
      const chip = root.querySelector("#run-status") as HTMLElement;
      chip.textContent = e.payload.status;
      chip.className = `chip s-${e.payload.status}`;
      chip.hidden = false;
    });
  }

  root.querySelector("#b-run")!.addEventListener("click", async () => {
    t0 = 0;
    const chk = (id: string) => (root.querySelector(id) as HTMLInputElement).checked;
    const params: RunParams = {
      video: (root.querySelector("#v-video") as HTMLInputElement).value,
      template: (root.querySelector("#v-template") as HTMLInputElement).value,
      annotations: (root.querySelector("#v-ann") as HTMLInputElement).value || null,
      out_dir: (root.querySelector("#v-out") as HTMLInputElement).value,
      audio: (root.querySelector("#v-audio") as HTMLInputElement).checked,
      language: (root.querySelector("#v-lang") as HTMLSelectElement).value as "en" | "zh",
      show_skeletons: chk("#p-skel"),
      show_player_trajectories: chk("#p-trail"),
      show_court_trajectory: chk("#p-court"),
      show_shuttlecock_trajectory: chk("#p-shuttle"),
      show_player_stats: chk("#p-stats"),
      show_pose_roi: chk("#p-roi"),
    };
    if (!params.video || !params.template || !params.out_dir) return alert("lengkapi video/template/output");
    (root.querySelector("#b-run") as HTMLButtonElement).disabled = true;
    (root.querySelector("#b-cancel") as HTMLButtonElement).disabled = false;
    (root.querySelector("#log") as HTMLElement).textContent = "";
    (root.querySelector("#run-status") as HTMLElement).hidden = true;
    try {
      // persist preferensi parameter (gagal simpan tidak menghalangi run)
      const [cfg] = await getDefaults();
      if (cfg) {
        cfg.defaults = {
          audio: params.audio,
          language: params.language,
          show_skeletons: params.show_skeletons,
          show_player_trajectories: params.show_player_trajectories,
          show_court_trajectory: params.show_court_trajectory,
          show_shuttlecock_trajectory: params.show_shuttlecock_trajectory,
          show_player_stats: params.show_player_stats,
          show_pose_roi: params.show_pose_roi,
        };
        await saveConfig(cfg).catch(() => {});
      }
      await startRun(params);
    } catch (e) {
      alert(String(e));
      (root.querySelector("#b-run") as HTMLButtonElement).disabled = false;
      (root.querySelector("#b-cancel") as HTMLButtonElement).disabled = true;
    }
  });
  root.querySelector("#b-cancel")!.addEventListener("click", () => {
    void cancelRun();
  });
  // Keep `save` import referenced for Task 8 reuse without changing public API.
  void save;
}
