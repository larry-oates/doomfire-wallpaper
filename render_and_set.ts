import { createCanvas } from "https://deno.land/x/canvas/mod.ts";
import initDoomFire from "./doomfire.ts";

const screen_width = 1940;
const screen_height = 720;
const scale = 5;
const width = screen_width/scale;
const height = screen_height/scale;
const canvas = createCanvas(width, height);
const ctx = canvas.getContext("2d");
const fire = initDoomFire(width, height);

const cacheDir = `${Deno.env.get("HOME")}/.cache/hyprpaper`;
await Deno.mkdir(cacheDir, { recursive: true });

const finalFrame = `${cacheDir}/fire_frame.png;`
const tempFrame = `${cacheDir}/fire_frame_tmp.png;`

async function getMonitorNames(): Promise<string[]> {
  const proc = Deno.run({ cmd: ["hyprctl", "monitors"], stdout: "piped" });
  const output = new TextDecoder().decode(await proc.output());
  proc.close();

  return output
    .split("\n")
    .filter((line) => line.startsWith("Monitor"))
    .map((line) => line.split(" ")[1].trim());
}

async function renderFrame(filePath: string) {
  fire.update();

  const img = ctx.createImageData(width, height);
  for (let i = 0; i < fire.pixelBuffer.length; i++) {
    const [r, g, b] = fire.palette[fire.pixelBuffer[i]];
    const pos = i * 4;
    img.data[pos] = r;
    img.data[pos + 1] = g;
    img.data[pos + 2] = b;
    img.data[pos + 3] = 255;
  }
  ctx.putImageData(img, 0, 0);

  // Save as PNG
  await Deno.writeFile(filePath, canvas.toBuffer("image/png"));
}

async function setWallpaper(monitors: string[], filePath: string) {
  for (const monitor of monitors) {
    const proc = Deno.run({
      cmd: ["hyprctl", "hyprpaper", "reload", `${monitor},${filePath}`],
    });
    await proc.status();
    proc.close();
  }
}

async function loop() {
  const monitors = await getMonitorNames();

  while (true) {
    await renderFrame(tempFrame);
    await Deno.rename(tempFrame, finalFrame);

    await setWallpaper(monitors, finalFrame);
    await new Promise((r) => setTimeout(r, 18)); // 10 FPS
  }
}

loop();
