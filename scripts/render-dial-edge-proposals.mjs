/**
 * Run after: mise exec rust -- cargo run --locked --example dial_readability
 * Native host text is approximated; SVG components and rectangles use production output.
 */
import sharp from "sharp";
import { readFile, writeFile, mkdir } from "node:fs/promises";
const dir = "target/dial-readability";
await mkdir(dir, { recursive: true });
const cases = JSON.parse(await readFile(dir + "/feedback.json", "utf8"));
const layouts = await Promise.all(["gain", "combo"].map(async n => JSON.parse(await readFile("assets/layouts/" + n + ".json", "utf8"))));
const esc = s => String(s).replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll('"', "&quot;");
const svg = (w,h,body) => Buffer.from('<svg xmlns="http://www.w3.org/2000/svg" width="'+w+'" height="'+h+'">'+body+'</svg>');
const text = (x,y,s,size=18,color="#e6eaf0") => '<text x="'+x+'" y="'+y+'" fill="'+color+'" font-family="Segoe UI, Yu Gothic UI, sans-serif" font-size="'+size+'">'+esc(s)+'</text>';
function wallpaper(type) {
 if(type===0) return '<rect width="200" height="100" fill="#18263b"/><path d="M0 80L85 12L200 100" fill="#26354c"/>';
 if(type===1) return '<defs><linearGradient id="wall"><stop stop-color="#ffe7a0"/><stop offset="1" stop-color="#f57838"/></linearGradient></defs><rect width="200" height="100" fill="url(#wall)"/>';
 return '<rect width="200" height="100" fill="#18374e"/>'+Array.from({length:12},(_,i)=>'<path d="M'+(i*24-100)+' 0l100 100h12L'+(i*24-88)+' 0z" fill="'+(i%2?"#efaf51":"#cbe7ee")+'"/>').join("");
}
async function render(c,kind,bg,opacity,variant,palette,treatment) {
 const frame=c.frames[kind], layers=[];
 for(const item of [...layouts[kind].items].sort((a,b)=>a.zOrder-b.zOrder)) {
  const p=frame[item.key]; if(!p) continue;
  const background=item.key.startsWith("background");
  if(!(background&&opacity!==undefined) && !p.enabled) continue;
  const [x,y,w,h]=item.rect;
  if(variant===3 && item.key==="icon") {
   for(const x0 of kind?[6,106]:[74]) {
    layers.push({input:await sharp(svg(kind?88:112,34,'<rect width="100%" height="34" rx="5" fill="#101c29" fill-opacity=".78"/>')).png().toBuffer(),left:x0,top:38});
   }
  }
  let input;
  if(item.type==="pixmap") {
   let source=Buffer.from(p.value.split(",")[1],"base64");
   if(variant && item.key.startsWith("status")) source=treatment<2?statusProposal(source):crispStatus(source,treatment===2);
   if(variant && item.key.startsWith("levelMeter")) {
    source=meterProposal(source,w,variant);
    if(treatment>0) source=Buffer.from(source.toString().replace('fill="#2c3d4d" fill-opacity=".65"','fill="black" fill-opacity="0.623529"'));
   }
   if(item.key.startsWith("background")) source=svg(w,h,'<rect width="'+w+'" height="'+h+'" fill="'+(palette==="vm"?"#2c3d4d":"#000000")+'"/>');
   if(palette==="standard" && item.key.startsWith("gainSlider")) source=Buffer.from(source.toString().replaceAll("#70c399","#ffffff").replaceAll("#f8634d","#ffffff"));

   const alpha=background?(opacity??p.opacity??1):1;
   if(alpha===0) continue;
   const raw=await sharp(source).resize(w,h).ensureAlpha().raw().toBuffer();
   for(let i=3;i<raw.length;i+=4) raw[i]=Math.round(raw[i]*alpha);
   input=await sharp(raw,{raw:{width:w,height:h,channels:4}}).png().toBuffer();
  } else {
   const right=item.alignment==="right";
   const fade=item["text-overflow"]==="fade";
   const mask=fade?'<defs><linearGradient id="fade"><stop offset="0.75" stop-color="white"/><stop offset="1" stop-color="black"/></linearGradient><mask id="m"><rect width="'+w+'" height="'+h+'" fill="url(#fade)"/></mask></defs>':"";
   input=await sharp(svg(w,h,mask+'<text x="'+(right?w:0)+'" y="18" text-anchor="'+(right?"end":"start")+'" fill="'+p.color+'" font-family="Segoe UI, Yu Gothic UI, sans-serif" font-size="'+item.font.size+'" font-weight="'+item.font.weight+'"'+(fade?' mask="url(#m)"':"")+'>'+esc(p.value)+'</text>')).png().toBuffer();
  }
  layers.push({input,left:x,top:y});
 }
 return sharp(svg(200,100,wallpaper(bg))).composite(layers).png().toBuffer();
}

function statusProposal(source) {
 let s=source.toString();
 // Keep production coordinates and active state; distinguish inactive slots by hollow shape.
 s=s.replaceAll('fill="#2c3d4d"','fill="#152331" fill-opacity=".35" stroke="#91a4b5" stroke-width=".65"');
 s=s.replaceAll('#70c399','#8bdfb1').replaceAll('#296ffd','#72a9ff');
 return Buffer.from(s);
}
function meterProposal(source,w,variant) {
 const raw=source.toString();
 const cells=[...raw.matchAll(/<rect x="([^"]+)" y="([^"]+)" width="1" height="1" fill="([^"]+)"\/>/g)];
 let body='<rect width="'+w+'" height="5" rx="2.5" fill="#2c3d4d" fill-opacity=".65"/>';
 if(variant===1) {
  for(const [,x,y,color] of cells) body+='<rect x="'+x+'" y="'+y+'" width="1.65" height="1" fill="'+(color==="#191b1b"?"#536575":color==="#1efe5b"?"#70c399":color)+'" fill-opacity="'+(color==="#191b1b"?".28":"1")+'"/>';
 } else {
  // Same occupied cells/levels as production. Two continuous stereo lanes.
  for(const y of ["1","3"]) {
   const active=cells.filter(c=>c[2]===y&&c[3]!=="#191b1b");
   for(const [,x,,color] of active) body+='<rect x="'+x+'" y="'+(Number(y)-.2)+'" width="2" height="1.4" fill="'+(color==="#1efe5b"?"#70c399":color)+'"/>';
  }
 }
 return svg(w,5,body);
}


// Draw whole native pixels; compare explicit stepped contours with antialiased SVG outlines.
function crispStatus(source,border) {
 const original=source.toString();
 const circles=[...original.matchAll(/<circle cx="([^"]+)" cy="([^"]+)" r="2.5" fill="([^"]+)"\/>/g)];
 const squares=[...original.matchAll(/<rect x="([^"]+)" y="15" width="7" height="7" rx="1.5" fill="([^"]+)"\/>/g)];
 let body="";
 function pixels(x,y,shape,color) {
  const off=color==="#2c3d4d";
  const fill=off?"#2c3d4d":color==="#70c399"?"#8bdfb1":color==="#296ffd"?"#72a9ff":color;
  for(let py=0;py<shape.length;py++) for(let px=0;px<shape[py].length;px++){
   if(shape[py][px]!=="1") continue;
   const edge= [[0,-1],[0,1],[-1,0],[1,0]].some(([dx,dy])=>shape[py+dy]?.[px+dx]!=="1");
   const c=off&&border&&edge?"#91a4b5":fill;
   body+='<rect x="'+(x+px)+'" y="'+(y+py)+'" width="1" height="1" fill="'+c+'"/>';
  }
 }
 for(const [,x,y,c] of circles) pixels(Number(x)-2.5,Number(y)-2.5,["01110","11111","11111","11111","01110"],c);
 for(const [,x,c] of squares) pixels(Number(x),15,["0111110","1111111","1111111","1111111","1111111","1111111","0111110"],c);
 return svg(36,24,body);
}
for(const kind of [0,1]) {
 const layers=[],w=896,h=710;
 let body='<rect width="100%" height="100%" fill="#10151c"/>'+text(16,26,(kind?"Combo":"Single")+" / meter background and indicator edges / 1x",19);
 const names=["Previous","Matched background","Matched + crisp outline","Matched + no outline"];
 for(let col=0;col<4;col++) body+=text(16+col*220,56,names[col],13);
 for(let row=0;row<4;row++){
  const palette=row<2?"standard":"vm",variant=row%2+1,y=86+row*150;
  body+=text(16,y,(palette==="vm"?"VoiceMeeter":"Black / white")+" / meter "+(variant===1?"A (cells)":"B (continuous)"),13,"#a9b5c2");
  for(let treatment=0;treatment<4;treatment++){
   const png=await render(cases[0],kind,0,1,variant,palette,treatment);
   await writeFile(dir+"/edge-"+kind+"-"+row+"-"+treatment+".png",png);
   layers.push({input:png,left:16+treatment*220,top:y+12});
  }
 }
 body+=text(16,695,"Opaque backgrounds. Matched = black at 62.35% opacity, same as fader rim. Preview only.",12,"#a9b5c2");
 await sharp(svg(w,h,body)).composite(layers).png().toFile(dir+"/indicator-edges-"+(kind?"combo":"single")+"-1x.png");
}
const zoomLayers=[];
let zoomBody='<rect width="100%" height="100%" fill="#10151c"/>'+text(16,26,"Details / VoiceMeeter + meter A (top 3x, bottom 6x)",20);
for(let col=0;col<4;col++){
 zoomBody+=text(16+col*310,58,["Previous","Matched background","Crisp outline","No outline"][col],15);
 const src=await readFile(dir+"/edge-1-2-"+col+".png");
 const input=await sharp(src).extract({left:6,top:38,width:88,height:50}).resize(264,150,{kernel:"nearest"}).png().toBuffer();
 zoomLayers.push({input,left:16+col*310,top:78});
 const detail=await sharp(src).extract({left:8,top:40,width:36,height:24}).resize(216,144,{kernel:"nearest"}).png().toBuffer();
 zoomLayers.push({input:detail,left:16+col*310,top:270});
}
zoomBody+=text(16,253,"Indicators only (6x, nearest-neighbor; no smoothing added)",14,"#a9b5c2");
await sharp(svg(1256,440,zoomBody)).composite(zoomLayers).png().toFile(dir+"/indicator-edges-detail.png");
console.log("Created indicator-edges-{single,combo}-1x.png and indicator-edges-detail.png");
