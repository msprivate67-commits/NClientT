// Generates all icon formats required by tauri.conf.json from
// src-tauri/icons/icon.png using only Node built-ins (no deps):
//   32x32.png, 128x128.png, 128x128@2x.png (256x256),
//   icon.ico (Windows), icon.icns (macOS placeholder via PNG container).
//
// PNGs are decoded/encoded by hand; the source PNG is downsampled with a
// simple box filter. The .ico embeds one PNG. The .icns is a minimal PNG-based
// ("ic09" 256x256) container which Tauri accepts on macOS for development.

import { readFileSync, writeFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import zlib from "node:zlib";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ICONS = join(__dirname, "..", "src-tauri", "icons");

const src = readFileSync(join(ICONS, "icon.png"));
const { width, height, rgba } = decodePng(src);

function decodePng(buf) {
  // Minimal PNG decoder for 8-bit RGBA / RGB.
  if (buf[0] !== 0x89 || buf[1] !== 0x50) throw new Error("not a png");
  let pos = 8;
  let width = 0, height = 0, bitDepth = 8, colorType = 6;
  const idat = [];
  while (pos < buf.length) {
    const len = buf.readUInt32BE(pos); pos += 4;
    const type = buf.toString("ascii", pos, pos + 4); pos += 4;
    const data = buf.subarray(pos, pos + len); pos += len + 4; // skip CRC
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
  return { width, height, rgba, channels, bpp, bitDepth };
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
      // Box filter: average the source area.
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

function writeIco(pngs) {
  // pngs: [{png: Buffer, size: number, w, h}]
  const count = pngs.length;
  const header = Buffer.alloc(6);
  header.writeUInt16LE(0, 0); // reserved
  header.writeUInt16LE(1, 2); // type = icon
  header.writeUInt16LE(count, 4);
  const dirSize = 16 * count;
  let offset = 6 + dirSize;
  const dir = Buffer.alloc(dirSize);
  pngs.forEach((p, i) => {
    const base = i * 16;
    dir[base] = p.w >= 256 ? 0 : p.w;
    dir[base + 1] = p.h >= 256 ? 0 : p.h;
    dir[base + 2] = 0;
    dir[base + 3] = 0;
    dir.writeUInt16LE(1, base + 4);
    dir.writeUInt16LE(32, base + 6);
    dir.writeUInt32LE(p.png.length, base + 8);
    dir.writeUInt32LE(offset, base + 12);
    offset += p.png.length;
  });
  return Buffer.concat([header, dir, ...pngs.map((p) => p.png)]);
}

function writeIcns(png256) {
  // Minimal icns with one ic09 (256x256) PNG entry.
  const type = Buffer.from("ic09", "ascii");
  const inner = Buffer.alloc(4 + png256.length);
  inner.writeUInt32BE(png256.length + 4, 0);
  png256.copy(inner, 4);
  const body = Buffer.concat([type, inner]);
  const magic = Buffer.from("icns", "ascii");
  const totalLen = Buffer.alloc(4);
  totalLen.writeUInt32BE(8 + body.length, 0);
  return Buffer.concat([magic, totalLen, body]);
}

const targets = [
  { w: 32, h: 32, name: "32x32.png" },
  { w: 128, h: 128, name: "128x128.png" },
  { w: 256, h: 256, name: "128x128@2x.png" },
];
const pngs = [];
for (const t of targets) {
  const data = resize(rgba, width, height, t.w, t.h);
  const png = encodePng(t.w, t.h, data);
  writeFileSync(join(ICONS, t.name), png);
  pngs.push({ png, size: png.length, w: t.w, h: t.h });
  console.log(`wrote ${t.name} (${t.w}x${t.h})`);
}

// Windows .ico: include 32, 128, 256.
const ico32 = pngs[0];
const ico128 = pngs[1];
const png256 = pngs[2].png;
const icoData = writeIco([ico32, ico128, { png: png256, w: 256, h: 256 }]);
writeFileSync(join(ICONS, "icon.ico"), icoData);
console.log(`wrote icon.ico`);

// macOS .icns
writeFileSync(join(ICONS, "icon.icns"), writeIcns(png256));
console.log(`wrote icon.icns`);
