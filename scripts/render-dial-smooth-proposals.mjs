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
   if(variant && item.key.startsWith("status")) source=treatment===0?statusProposal(source):treatment===1?crispStatus(source,true):smoothStatus(source,treatment===2);
   if(variant && item.key.startsWith("levelMeter")) {
    source=meterProposal(source,w,variant);
    source=Buffer.from(source.toString().replace('fill="#2c3d4d" fill-opacity=".65"','fill="black" fill-opacity="0.623529"'));
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

function smoothStatus(source,border) {
 let s=source.toString();
 // Inset 1px strokes by 0.5px: straight edges land on integer pixel boundaries.
 s=s.replace(/<circle cx="([^"]+)" cy="([^"]+)" r="2.5" fill="([^"]+)"\/>/g,(_,x,y,c)=>{
  if(c!=="#2c3d4d") return '<circle cx="'+x+'" cy="'+y+'" r="2.5" fill="'+(c==="#70c399"?"#8bdfb1":c)+'"/>';
  return '<circle cx="'+x+'" cy="'+y+'" r="'+(border?'2':'2.5')+'" fill="#2c3d4d"'+(border?' stroke="#91a4b5" stroke-width="1"':'')+'/>';
 });
 s=s.replace(/<rect x="([^"]+)" y="15" width="7" height="7" rx="1.5" fill="([^"]+)"\/>/g,(_,x,c)=>{
  if(c!=="#2c3d4d") return '<rect x="'+x+'" y="15" width="7" height="7" rx="1.5" fill="'+(c==="#296ffd"?"#72a9ff":c)+'"/>';
  return border?'<rect x="'+(Number(x)+.5)+'" y="15.5" width="6" height="6" rx="1" fill="#2c3d4d" stroke="#91a4b5" stroke-width="1"/>':'<rect x="'+x+'" y="15" width="7" height="7" rx="1.5" fill="#2c3d4d"/>';
 });
 return Buffer.from(s);
}
const names=["Previous thin outline","Pixel outline","1px outline + smooth curves","Smooth / no outline"];
for(const kind of [0,1]){
 let body='<rect width="100%" height="100%" fill="#10151c"/>'+text(16,26,(kind?"Combo":"Single")+" / aligned strokes, antialiased curves / 1x",19);
 const layers=[];
 for(let col=0;col<4;col++) body+=text(16+col*220,56,names[col],12);
 for(let row=0;row<2;row++){
  const y=86+row*150;
  body+=text(16,y,row?"VoiceMeeter / meter A":"Black / white / meter A",13,"#a9b5c2");
  for(let col=0;col<4;col++){
   const png=await render(cases[0],kind,0,1,1,row?"vm":"standard",col);
   await writeFile(dir+"/smooth-"+kind+"-"+row+"-"+col+".png",png);
   layers.push({input:png,left:16+col*220,top:y+12});
  }
 }
 await sharp(svg(896,390,body)).composite(layers).png().toFile(dir+"/smooth-indicators-"+(kind?"combo":"single")+"-1x.png");
}
let body='<rect width="100%" height="100%" fill="#10151c"/>'+text(16,28,"Indicator contours / 8x native pixels (no added interpolation)",20);
const layers=[];
for(let col=0;col<4;col++){
 body+=text(16+col*310,62,names[col],14);
 const input=await sharp(await readFile(dir+"/smooth-1-1-"+col+".png")).extract({left:8,top:40,width:36,height:24}).resize(288,192,{kernel:"nearest"}).png().toBuffer();
 layers.push({input,left:16+col*310,top:85});
}
await sharp(svg(1256,300,body)).composite(layers).png().toFile(dir+"/smooth-indicators-detail.png");
console.log("Rendered smooth indicator comparisons");
