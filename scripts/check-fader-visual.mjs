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

// Check continuity at the pair seam and separation of the two tracks using
// actual raster pixels, for both orientations and both placement options.
for (const vertical of [false,true]) for (const swap of [false,true]) {
  const names=vertical ? ['upper','lower'] : ['left','right'];
  const buffers=await Promise.all(names.map(async name => sharp(await readFile(`target/fader-visual/${name}-meter-${swap}.svg`)).removeAlpha().raw().toBuffer()));
  const pixel=(b,x,y)=>[...b.subarray((y*144+x)*3,(y*144+x)*3+3)];
  for(let cross=0;cross<144;cross++) {
    assert.deepEqual(vertical ? pixel(buffers[0],cross,143) : pixel(buffers[0],143,cross),
      vertical ? pixel(buffers[1],cross,0) : pixel(buffers[1],0,cross), 'pair seam must be continuous');
  }
  const meterMin=swap?30:98, faderCenter=swap?96:48;
  let green=0, white=0;
  const railExtents=[], meterExtents=[];
  for(const [half,b] of buffers.entries()) for(let y=0;y<144;y++) for(let x=0;x<144;x++) {
    const [r,g,blue]=pixel(b,x,y), cross=vertical?x:y;
    const along=vertical ? y+half*144 : x+half*144;
    if(cross===meterMin+3 && r+g+blue>50) meterExtents.push(along);
    if(cross===faderCenter && r>128) railExtents.push(along);
    if(g>r+20 && g>blue+20 && g>80) {
      green++;
      assert.ok(cross>=meterMin && cross<meterMin+16,'meter must stay in its track');
    }
    if(r>245 && g>245 && blue>245 && !(cross>=meterMin && cross<meterMin+16)) {
      white++;
      assert.ok(Math.abs(cross-faderCenter)<=21,'fader must stay in its separate track');
    }
  }
  assert.ok(green>100 && white>100,'both meter and fader must be visible');
  assert.ok(Math.abs(Math.min(...railExtents)-Math.min(...meterExtents))<=1, 'rail and meter start must align within antialiasing');
  assert.ok(Math.abs(Math.max(...railExtents)-Math.max(...meterExtents))<=1, 'rail and meter end must align within antialiasing');
  const width=vertical?144:288,height=vertical?288:144;
  await sharp({create:{width,height,channels:3,background:'#000'}}).composite(buffers.map((input,i)=>({input,raw:{width:144,height:144,channels:3},left:vertical?0:i*144,top:vertical?i*144:0}))).png().toFile(`target/fader-visual/pair-${vertical?'vertical':'horizontal'}-${swap}.png`);
}
console.log('Paired meter continuity and track separation passed for both orientations and placements.');
