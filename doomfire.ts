// Simple 1D Perlin noise implementation
function fade(t: number) {
  return t * t * t * (t * (t * 6 - 15) + 10);
}

function lerp(a: number, b: number, t: number) {
  return a + t * (b - a);
}

function grad(hash: number, x: number) {
  const h = hash & 15;
  let grad = 1 + (h & 7); // Gradient value 1-8
  if ((h & 8) !== 0) grad = -grad;
  return grad * x;
}

const permutation = [
  151,160,137,91,90,15, // ... fill with values 0-255 in any permutation
  131,13,201,95,96,53,194,233,7,225,140,36,103,30,69,142,
  // Repeat the array to avoid overflow:
  151,160,137,91,90,15,131,13,201,95,96,53,194,233,7,225,140,36,103,30,69,142
];

function perlinNoise1D(x: number): number {
  const xi = Math.floor(x) & 255;
  const xf = x - Math.floor(x);
  const u = fade(xf);

  const a = permutation[xi];
  const b = permutation[xi + 1];

  const gradA = grad(a, xf);
  const gradB = grad(b, xf - 1);

  return lerp(gradA, gradB, u);
}

export default function initDoomFire(width: number, height: number) {
  const size = width * height;
  const pixelBuffer = new Uint8Array(size).fill(0);
  const palette = generatePalette();

  // Initialize bottom row as fire source
  for (let x = 0; x < width; x++) {
    pixelBuffer[(height - 1) * width + x] = palette.length - 1;
  }

  let t = 0;   // <== Declare here
  let wind = 0;


  function update() {
    t += 0.01;
    const noiseVal = perlinNoise1D(t);
    wind = Math.round(noiseVal * 2); // -2 to +2

    for (let y = height - 1; y > 1; y--) {
      for (let x = 0; x < width; x++) {
        const src = y * width + x;
        const decay = Math.floor(Math.random() * 2); // Less decay = taller fire

        const xOffset = Math.floor(Math.random() * 3) - 1 + wind;
        const dstX = x + xOffset;
        const dstY = y - (Math.random() > 0.7 ? 2 : 1); // sometimes spread 2 rows up

        if (dstX >= 0 && dstX < width && dstY >= 0) {
          const dst = Math.floor(dstY) * width + dstX;
          const value = Math.max(0, pixelBuffer[src] - decay);
          pixelBuffer[dst] = value;
        }
      }
    }
  }
  return { width, height, pixelBuffer, palette, update };
}


function generatePalette() {
  // basic gradient from black to bright red/yellow
  const p = [];
  for (let i = 0; i <= 36; i++) {
    const t = i / 36;
    const r = Math.min(255, Math.floor(255 * Math.pow(t, 0.5)));
    const g = Math.min(255, Math.floor(255 * Math.pow(t, 3)));
    const b = 0;
    p.push([r, g, b]);
  }
  return p as [number, number, number][];
}
