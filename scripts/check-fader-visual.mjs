import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import sharp from 'sharp';

// SVGs are exported by gain_button_fixtures from the production Rust renderer.
// Rasterize at the screenshot's native 72 px resolution, without resampling.
const results = [];
for (const name of ['left', 'right', 'upper', 'lower']) {
  const svg = await readFile(`target/fader-visual/${name}.svg`);
  const reference = await sharp(`tests/fixtures/fader/${name}.png`).removeAlpha().greyscale().raw().toBuffer();
  const {data: actual, info} = await sharp(svg, {density:36}).removeAlpha().greyscale().raw().toBuffer({resolveWithObject:true});
  assert.equal(info.width,72); assert.equal(info.height,72); assert.equal(info.channels,1);
  let squared = 0, count = 0, mismatch = 0;
  const diff = Buffer.alloc(72*72*3);
  for(let y=0;y<72;y++) for(let x=0;x<72;x++) {
    // Rounded host key-frame corners are not plugin pixels. Exclude only
    // those corners, never the rail, thumb, or their surrounding clearances.
    if((x<14 || x>57) && (y<14 || y>57)) continue;
    const i=y*72+x, d=actual[i]-reference[i];
    squared+=d*d; count++;
    if((actual[i]>128)!==(reference[i]>128)) mismatch++;
    diff[i*3]=Math.max(0,-d); diff[i*3+1]=Math.max(0,d);
  }
  const rmse=Math.sqrt(squared/count);
  const result={name, rmse, silhouetteMismatchPixels:mismatch, comparedPixels:count};
  results.push(result);
  await sharp(svg,{density:36}).png().toFile(`target/fader-visual/${name}.png`);
  await sharp(diff,{raw:{width:72,height:72,channels:3}}).png().toFile(`target/fader-visual/${name}-diff.png`);
  console.log(result);
  assert.ok(rmse<=2, `${name}: grayscale RMSE ${rmse} exceeds 2/255`);
  assert.ok(mismatch<=4, `${name}: ${mismatch} silhouette pixels differ (maximum 4)`);
}
await writeFile('target/fader-visual/results.json',JSON.stringify(results,null,2)+'\n');
