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
async function render(c,kind,bg,opacity,variant) {
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
   if(variant && item.key.startsWith("status")) source=statusProposal(source);
   if(variant && item.key.startsWith("levelMeter")) source=meterProposal(source,w,variant);
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
const names=["Current","A: softer cells + hollow indicators","B: continuous stereo meter","C: B + local dark backing"];
async function proposalSheet(kind) {
 const cw=424,ch=270,w=cw*4+32,h=ch*3+104;
 let body='<rect width="100%" height="100%" fill="#10151c"/>'+text(20,30,kind?"Combo: proposed indicator and meter treatments":"Single: proposed indicator and meter treatments",24)+text(20,57,"Preview only / entire background opacity 40% / production positions and states / text approximated / 2x",14);
 const layers=[];
 for(let bg=0;bg<3;bg++) for(let variant=0;variant<4;variant++){
  const x=20+variant*cw,y=88+bg*ch;
  body+=text(x,y,names[variant],16)+text(x,y+20,["Dark wallpaper","Bright wallpaper","Busy wallpaper"][bg],13,"#94a3b8");
  const png=await render(cases[0],kind,bg,.4,variant);
  await writeFile(dir+"/proposal-"+kind+"-"+bg+"-"+variant+".png",png);
  layers.push({input:await sharp(png).resize(400,200,{kernel:"nearest"}).png().toBuffer(),left:x,top:y+32});
 }
 const path=dir+"/proposals-"+(kind?"combo":"single")+".png";
 await sharp(svg(w,h,body)).composite(layers).png().toFile(path);
 console.log(path);
}
await proposalSheet(0);
await proposalSheet(1);
