import { open, save } from "@tauri-apps/plugin-dialog";
import { convertFileSrc } from "@tauri-apps/api/core";
import { saveAnnotations } from "./api";

export function renderCorners(root: HTMLElement) {
  root.innerHTML = `
    <div class="court-bar">
      <button id="c-open" class="btn btn-ghost" type="button">Buka template…</button>
      <span id="c-pts" class="chip">Titik: 0/4</span>
      <label class="field-inline">mid_height <input id="c-mid" class="num" type="number" value="625" /></label>
      <button id="c-save" class="btn btn-primary" type="button" disabled>Simpan annotations</button>
    </div>
    <div class="canvas-wrap card"><canvas id="c-canvas" width="478" height="850"></canvas></div>`;
  const canvas = root.querySelector("#c-canvas") as HTMLCanvasElement;
  const ctx = canvas.getContext("2d")!;
  const pts: [number, number][] = [];
  let img: HTMLImageElement | null = null;
  let imgPath = "";

  const redraw = () => {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    if (img) ctx.drawImage(img, 0, 0, canvas.width, canvas.height);
    // connector antar titik (aksen teal; dashed sampai 4 titik lengkap)
    if (pts.length >= 2) {
      ctx.strokeStyle = "rgba(67, 220, 201, 0.9)";
      ctx.lineWidth = 2;
      ctx.setLineDash(pts.length === 4 ? [] : [6, 4]);
      ctx.beginPath();
      pts.forEach(([x, y], i) => (i ? ctx.lineTo(x, y) : ctx.moveTo(x, y)));
      if (pts.length === 4) ctx.closePath();
      ctx.stroke();
      ctx.setLineDash([]);
    }
    // marker bernomor (urutan klik)
    ctx.font = "600 11px 'IBM Plex Mono', monospace";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    pts.forEach(([x, y], i) => {
      ctx.beginPath();
      ctx.arc(x, y, 9, 0, Math.PI * 2);
      ctx.fillStyle = "#43dcc9";
      ctx.fill();
      ctx.strokeStyle = "#06231f";
      ctx.lineWidth = 2;
      ctx.stroke();
      ctx.fillStyle = "#06231f";
      ctx.fillText(String(i + 1), x, y + 0.5);
    });
    (root.querySelector("#c-pts") as HTMLElement).textContent = `Titik: ${pts.length}/4`;
    (root.querySelector("#c-save") as HTMLButtonElement).disabled = pts.length !== 4;
  };

  root.querySelector("#c-open")!.addEventListener("click", async () => {
    const f = (await open({ multiple: false, filters: [{ name: "PNG", extensions: ["png"] }] })) as string | null;
    if (!f) return;
    imgPath = f;
    img = new Image();
    img.onload = () => {
      canvas.width = img!.width;
      canvas.height = img!.height;
      redraw();
    };
    img.src = convertFileSrc(f);
  });

  canvas.addEventListener("click", (ev) => {
    if (pts.length >= 4) return;
    const r = canvas.getBoundingClientRect();
    const x = Math.round((ev.clientX - r.left) * (canvas.width / r.width));
    const y = Math.round((ev.clientY - r.top) * (canvas.height / r.height));
    pts.push([x, y]);
    redraw();
  });

  root.querySelector("#c-save")!.addEventListener("click", async () => {
    const path = (await save({ defaultPath: "court_annotations.txt", filters: [{ name: "txt", extensions: ["txt"] }] })) as string | null;
    if (!path) return;
    const mid = Number((root.querySelector("#c-mid") as HTMLInputElement).value);
    try {
      await saveAnnotations(path, pts, mid);
      alert("Tersimpan: " + path);
    } catch (e) {
      alert(String(e));
    }
  });
  void imgPath;
}
