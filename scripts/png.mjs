import { deflateSync } from "node:zlib";

function chunk(type, data) {
  const bytes = Buffer.concat([Buffer.from(type), data]);
  let crc = 0xffffffff;
  for (const byte of bytes) {
    crc ^= byte;
    for (let bit = 0; bit < 8; bit++) crc = (crc >>> 1) ^ (crc & 1 ? 0xedb88320 : 0);
  }
  const header = Buffer.alloc(4), trailer = Buffer.alloc(4);
  header.writeUInt32BE(data.length); trailer.writeUInt32BE((crc ^ 0xffffffff) >>> 0);
  return Buffer.concat([header, bytes, trailer]);
}

// Original waveform artwork; no external graphics or conversion runtime.
export function pluginPng(size) {
  const pixels = Buffer.alloc(size * (size * 4 + 1));
  const segments = [[12,36,20,36],[20,36,25,17],[25,17,35,55],[35,55,45,17],[45,17,50,36],[50,36,60,36]];
  for (let y = 0; y < size; y++) for (let x = 0; x < size; x++) {
    const px = (x + 0.5) * 72 / size, py = (y + 0.5) * 72 / size;
    const line = segments.some(([ax, ay, bx, by]) => {
      const t = Math.max(0, Math.min(1, ((px-ax)*(bx-ax)+(py-ay)*(by-ay))/((bx-ax)**2+(by-ay)**2)));
      return Math.hypot(px-ax-t*(bx-ax), py-ay-t*(by-ay)) <= 2;
    });
    pixels.set(line ? [96,211,148,255] : [32,32,32,255], y*(size*4+1)+1+x*4);
  }
  const header = Buffer.alloc(13);
  header.writeUInt32BE(size,0); header.writeUInt32BE(size,4); header[8]=8; header[9]=6;
  return Buffer.concat([Buffer.from([137,80,78,71,13,10,26,10]), chunk("IHDR",header), chunk("IDAT",deflateSync(pixels)), chunk("IEND",Buffer.alloc(0))]);
}
