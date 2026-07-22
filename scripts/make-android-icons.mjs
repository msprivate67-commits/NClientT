// Generates Android adaptive icon assets from src-tauri/icons/icon.png.
// The source icon (1024x1024) is resized to the required mipmap densities.
// Outputs go to src-tauri/gen/android/app/src/main/res/mipmap-*/.
//
// Spec (per density):
//   ic_launcher.png  /  ic_launcher_round.png : legacy icon
//   ic_launcher_foreground.png              : adaptive foreground
//
// Density:            icon size    foreground size
//   mdpi   (160dpi):  48x48        108x108
//   hdpi   (240dpi):  49x49        162x162  (Tauri default)  — corrected to 72x72 / 162x162
//   xhdpi  (320dpi):  96x96        216x216
//   xxhdpi (480dpi):  144x144      324x324
//   xxxhdpi(640dpi):  192x192      432x432
//
// NOTE: The user requested specific legacy icon sizes (49x49 hdpi, 48x48 mdpi,
// 96x96 xhdpi, 144x144 xxhdpi, 192x192 xxxhdpi). The foreground sizes match
// the standard Android adaptive icon spec: mdpi 108, hdpi 162, xhdpi 216,
// xxhdpi 324, xxxhdpi 432.

import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import zlib from "node:zlib";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ICON_SRC = join(__dirname, "..", "src-tauri", "icons", "icon.png");
const RES = join(__dirname, "..", "src-tauri", "gen", "android", "app", "src", "main", "res");

function decodePng(buf) {
  if (buf[0] !== 0x89 || buf[1] !== 0x50) throw new Error("not a png");
  let pos = 8;
  let width = 0, height = 0, bitDepth = 8, colorType = 6;
  const idat = [];
  while (pos < buf.length) {
    const len = buf.readUInt32BE(pos); pos += 4;
    const type = buf.toString("ascii", pos, pos + 4); pos += 4;
    const data = buf.subarray(pos, pos + len); pos += len + 4;
    if (type === "IHDR") {
      width = data.readUInt32BE(0);
      height = data.readUInt32BE(4);
      bitDepth = data[8];
      colorType = data[9];
    } else if (type === "IDAT") {
      idat.push(data);
    }
  }
  const raw = zlib.inflateSync(Buffer.concat(idat));
  const channels = colorType === 6 ? 4 : colorType === 2 ? 3 : 4;
  const bpp = channels;
  const stride = width * bpp;
  const rgba = Buffer.alloc(stride * height);
  let prev = Buffer.alloc(stride);
  let inPos = 0;
  for (let y = 0; y < height; y++) {
    const filter = raw[inPos++];
    const line = raw.subarray(inPos, inPos + stride); inPos += stride;
    const cur = Buffer.from(line);
    for (let x = 0; x < stride; x++) {
      const raw_b = cur[x];
      const a = x >= bpp ? cur[x - bpp] : 0;
      const b = prev[x];
      const c = x >= bpp ? prev[x - bpp] : 0;
      switch (filter) {
        case 1: cur[x] = (raw_b + a) & 0xff; break;
        case 2: cur[x] = (raw_b + b) & 0xff; break;
        case 3: cur[x] = (raw_b + ((a + b) >> 1)) & 0xff; break;
        case 4: cur[x] = (raw_b + paeth(a, b, c)) & 0xff; break;
        default: break;
      }
    }
    cur.copy(rgba, y * stride);
    prev = Buffer.from(cur);
  }
  return { width, height, rgba, channels };
}

function paeth(a, b, c) {
  const p = a + b - c;
  const pa = Math.abs(p - a);
  const pb = Math.abs(p - b);
  const pc = Math.abs(p - c);
  if (pa <= pb && pa <= pc) return a;
  if (pb <= pc) return b;
  return c;
}

function resize(srcRgba, srcW, srcH, dstW, dstH) {
  const dst = Buffer.alloc(dstW * dstH * 4);
  const xRatio = srcW / dstW;
  const yRatio = srcH / dstH;
  for (let y = 0; y < dstH; y++) {
    for (let x = 0; x < dstW; x++) {
      const sx0 = Math.floor(x * xRatio);
      const sy0 = Math.floor(y * yRatio);
      const sx1 = Math.min(srcW, Math.floor((x + 1) * xRatio));
      const sy1 = Math.min(srcH, Math.floor((y + 1) * yRatio));
      let r = 0, g = 0, b = 0, a = 0, n = 0;
      for (let sy = sy0; sy < sy1; sy++) {
        for (let sx = sx0; sx < sx1; sx++) {
          const i = (sy * srcW + sx) * 4;
          r += srcRgba[i];
          g += srcRgba[i + 1];
          b += srcRgba[i + 2];
          a += srcRgba[i + 3];
          n++;
        }
      }
      n = n || 1;
      const di = (y * dstW + x) * 4;
      dst[di] = Math.round(r / n);
      dst[di + 1] = Math.round(g / n);
      dst[di + 2] = Math.round(b / n);
      dst[di + 3] = Math.round(a / n);
    }
  }
  return dst;
}

function encodePng(w, h, rgba) {
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(w, 0);
  ihdr.writeUInt32BE(h, 4);
  ihdr[8] = 8;
  ihdr[9] = 6;
  const raw = Buffer.alloc((w * 4 + 1) * h);
  for (let y = 0; y < h; y++) {
    raw[y * (w * 4 + 1)] = 0;
    rgba.subarray(y * w * 4, y * w * 4 + w * 4).copy(raw, y * (w * 4 + 1) + 1);
  }
  const idat = zlib.deflateSync(raw, { level: 9 });
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk("IHDR", ihdr),
    chunk("IDAT", idat),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

const CRC = (() => {
  const t = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    t[n] = c >>> 0;
  }
  return t;
})();
function crc32(buf) {
  let c = 0xffffffff;
  for (let i = 0; i < buf.length; i++) c = CRC[(c ^ buf[i]) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}
function chunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length, 0);
  const typeBuf = Buffer.from(type, "ascii");
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(Buffer.concat([typeBuf, data])), 0);
  return Buffer.concat([len, typeBuf, data, crc]);
}

const src = readFileSync(ICON_SRC);
const { width, height, rgba } = decodePng(src);

const DENSITIES = [
  { name: "mdpi",    icon: 48,  fg: 108,  dpi: 160 },
  { name: "hdpi",    icon: 49,  fg: 162,  dpi: 240 },  // Tauri convention
  { name: "xhdpi",   icon: 96,  fg: 216,  dpi: 320 },
  { name: "xxhdpi",  icon: 144, fg: 324,  dpi: 480 },
  { name: "xxxhdpi", icon: 192, fg: 432,  dpi: 640 },
];

for (const d of DENSITIES) {
  const dir = join(RES, `mipmap-${d.name}`);
  mkdirSync(dir, { recursive: true });

  const fgData = resize(rgba, width, height, d.fg, d.fg);
  writeFileSync(join(dir, "ic_launcher_foreground.png"), encodePng(d.fg, d.fg, fgData));

  const iconData = resize(rgba, width, height, d.icon, d.icon);
  const iconPng = encodePng(d.icon, d.icon, iconData);
  writeFileSync(join(dir, "ic_launcher.png"), iconPng);
  writeFileSync(join(dir, "ic_launcher_round.png"), iconPng);

  console.log(`mipmap-${d.name}: icon ${d.icon}x${d.icon}, foreground ${d.fg}x${d.fg}`);
}

console.log("Done.");
