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
  // A cell edge can fall on the seam. Compare against one uninterrupted
  // production canvas instead of requiring adjacent pixels to be identical.
  let fullSvg = await readFile(`target/fader-visual/${names[0]}-meter-${swap}.svg`, 'utf8');
  fullSvg = fullSvg.replaceAll('width="144" height="144"', vertical ? 'width="144" height="288"' : 'width="288" height="144"')
    .replace('viewBox="0 0 144 144"', vertical ? 'viewBox="0 0 144 288"' : 'viewBox="0 0 288 144"');
  const full = await sharp(Buffer.from(fullSvg)).removeAlpha().raw().toBuffer();
  for (const [half,b] of buffers.entries()) for(let y=0;y<144;y++) for(let x=0;x<144;x++) {
    const i=((vertical ? y+half*144 : y)*(vertical?144:288)+(vertical?x:x+half*144))*3;
    assert.deepEqual(pixel(b,x,y), [...full.subarray(i,i+3)], 'both keys must preserve the uninterrupted cell grid');
  }
  const meterMin=swap?88:36, faderCenter=swap?48:96;
  assert.equal(swap ? faderCenter-14 : meterMin-2,34,'outer margin is 17px');
  assert.equal(swap ? 144-(meterMin+22) : 144-(faderCenter+14),34,'opposite outer margin is 17px');
  assert.equal((Math.abs(meterMin+10-faderCenter)-12-14)/2,12,'bar clearance is 12px');
  let green=0, white=0;
  const railExtents=[], meterExtents=[];
  for(const [half,b] of buffers.entries()) for(let y=0;y<144;y++) for(let x=0;x<144;x++) {
    const [r,g,blue]=pixel(b,x,y), cross=vertical?x:y;
    const along=vertical ? y+half*144 : x+half*144;
    if(cross===meterMin+3 && r+g+blue>20) meterExtents.push(along);
    if(cross===faderCenter && r>128) railExtents.push(along);
    if(g>r+20 && g>blue+20 && g>80) {
      green++;
      assert.ok(cross>=meterMin && cross<meterMin+20,'meter must stay in its track');
    }
    if(r>245 && g>245 && blue>245 && !(cross>=meterMin && cross<meterMin+20)) {
      white++;
      assert.ok(Math.abs(cross-faderCenter)<=21,'fader must stay in its separate track');
    }
  }
  assert.ok(green>100 && white>100,'both meter and fader must be visible');
  for(const [half,name] of names.entries()) {
    const labeled=await sharp(await readFile(`target/fader-visual/${name}-label-${swap}.svg`)).removeAlpha().raw().toBuffer();
    if(name==='right' || name==='upper') assert.deepEqual(labeled,buffers[half],'positive half must never render a label');
    else {
      let changed=0;
      for(let y=0;y<144;y++) for(let x=0;x<144;x++) {
        if(!pixel(labeled,x,y).every((v,i)=>v===pixel(buffers[half],x,y)[i])) {
          changed++;
          assert.ok(Math.abs((vertical?x:y)-faderCenter)<=13,'label must be clipped inside the rail');
        }
      }
      assert.ok(changed>20,'negative half must actually display its label');
    }
  }

  assert.ok(Math.abs(Math.min(...railExtents)-Math.min(...meterExtents))<=2, 'rail and meter start must align within antialiasing');
  assert.ok(Math.abs(Math.max(...railExtents)-Math.max(...meterExtents))<=2, 'rail and meter end must align within antialiasing');
  const width=vertical?144:288,height=vertical?288:144;
  await sharp({create:{width,height,channels:3,background:'#000'}}).composite(buffers.map((input,i)=>({input,raw:{width:144,height:144,channels:3},left:vertical?0:i*144,top:vertical?i*144:0}))).png().toFile(`target/fader-visual/pair-${vertical?'vertical':'horizontal'}-${swap}.png`);
}
console.log('Paired meter continuity and track separation passed for both orientations and placements.');

// Removing the text clip must not reveal any lost glyph pixels at bounds.
for(const name of ['left','right','upper','lower']) for(const gain of [-60,-24,-12.5,0,6,12]) {
  const svg=await readFile(`target/fader-visual/${name}-db-${gain}.svg`,'utf8');
  const actual=await sharp(Buffer.from(svg)).raw().toBuffer();
  const unclipped=await sharp(Buffer.from(svg.replace('clip-path="url(#gain-number)"',''))).raw().toBuffer();
  assert.deepEqual(actual,unclipped,`${name} ${gain}: digits must fit inside the circle`);
  assert.ok(!svg.includes(' dB'),'no unit outside the thumb');
  assert.ok(svg.includes('translate(0 24)'),'gain text never shifts the fader axis');
  assert.ok(svg.includes('y="34"'),'gain text never shifts the meter background');
  assert.ok(svg.includes(name==='upper'||name==='lower' ? 'rotate(90 ' : 'rotate(0 '),'numbers stay horizontal');
}
console.log('In-thumb numbers fit at gain limits, zero, positive values and the key seam.');
