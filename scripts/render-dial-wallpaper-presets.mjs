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
async function render(c,kind,bg,opacity) {
 const frame=c.frames[kind], layers=[];
 for(const item of [...layouts[kind].items].sort((a,b)=>a.zOrder-b.zOrder)) {
  const p=frame[item.key]; if(!p) continue;
  const background=item.key.startsWith("background");
  if(!(background&&opacity!==undefined) && !p.enabled) continue;
  const [x,y,w,h]=item.rect;
  let input;
  if(item.type==="pixmap") {
   const source=Buffer.from(p.value.split(",")[1],"base64");
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

const layers=[];
let body='<rect width="100%" height="100%" fill="#10151c"/>'+text(16,28,"Wallpaper presets / transparent background / 1x",20);
const headings=["Single / no outline","Single / outline","Combo / no outline","Combo / outline"];
for(let col=0;col<4;col++) body+=text(16+col*220,58,headings[col],14);
for(let bg=0;bg<3;bg++){
 const y=88+bg*145;
 body+=text(16,y,["Dark wallpaper","Bright wallpaper","Busy wallpaper"][bg],13,"#a9b5c2");
 for(let col=0;col<4;col++){
  const png=await render(cases[9+col%2],col<2?0:1,bg,0);
  await writeFile(dir+"/wallpaper-preset-"+bg+"-"+col+".png",png);
  layers.push({input:png,left:16+col*220,top:y+12});
 }
}
body+=text(16,530,"White preset base / inactive indicators: black, 62% / empty fader: #808080 / native text approximated",12,"#a9b5c2");
await sharp(svg(896,550,body)).composite(layers).png().toFile(dir+"/wallpaper-presets-1x.png");
let zoom='<rect width="100%" height="100%" fill="#10151c"/>'+text(16,28,"Combo / 2x / no outline (left), outline (right)",20);
const zoomLayers=[];
for(let bg=0;bg<3;bg++) for(let col=0;col<2;col++){
 zoomLayers.push({input:await sharp(await readFile(dir+"/wallpaper-preset-"+bg+"-"+(col+2)+".png")).resize(400,200,{kernel:"nearest"}).png().toBuffer(),left:16+col*424,top:50+bg*216});
}
await sharp(svg(856,710,zoom)).composite(zoomLayers).png().toFile(dir+"/wallpaper-presets-2x.png");
console.log("Created wallpaper-presets-1x.png and wallpaper-presets-2x.png");
