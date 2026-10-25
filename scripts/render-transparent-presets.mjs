/**
 * Run after: mise exec rust -- cargo run --locked --example transparent_presets
 * Native host text is approximated; SVG components and rectangles use production output.
 */
import sharp from "sharp";
import { readFile, writeFile, mkdir } from "node:fs/promises";
const dir = "target/transparent-presets";
await mkdir(dir, { recursive: true });
const cases = JSON.parse(await readFile(dir + "/feedback.json", "utf8"));
const layouts = await Promise.all(["gain", "combo"].map(async n => JSON.parse(await readFile("assets/layouts/" + n + ".json", "utf8"))));
const esc = s => String(s).replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll('"', "&quot;");
const svg = (w,h,body) => Buffer.from('<svg xmlns="http://www.w3.org/2000/svg" width="'+w+'" height="'+h+'">'+body+'</svg>');
const text = (x,y,s,size=18,color="#e6eaf0") => '<text x="'+x+'" y="'+y+'" fill="'+color+'" font-family="Segoe UI, Yu Gothic UI, sans-serif" font-size="'+size+'">'+esc(s)+'</text>';
function wallpaper(type) {
 if(type===0) return '<rect width="200" height="100" fill="#18263b"/><path d="M0 80L85 12L200 100" fill="#26354c"/>';
 if(type===1) return '<defs><linearGradient id="wall"><stop stop-color="#ffe7a0"/><stop offset="1" stop-color="#f57838"/></linearGradient></defs><rect width="200" height="100" fill="url(#wall)"/>';
 if(type===3) return '<defs><linearGradient id="mixed"><stop stop-color="#eee9de"/><stop offset=".5" stop-color="#bb7660"/><stop offset="1" stop-color="#101a3b"/></linearGradient></defs><rect width="200" height="100" fill="url(#mixed)"/><path d="M0 92L45 45L80 78L145 24L200 70V100H0Z" fill="#213148"/><path d="M65 100L115 45L165 100" fill="#4c6374"/>';
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

const names=["A: Standard","B: Outlined / dark inactive","C: Bright inactive","D: Dark text"];
const rows=["Dark geometry","Bright warm gradient","Dense stripes (stress test)","Mixed bright/dark landscape"];
for(const kind of [0,1]){
 let body='<rect width="100%" height="100%" fill="#10151c"/>'+text(16,26,(kind?"Combo":"Single")+" / fully transparent / monochrome base / native 1x",20);
 const layers=[];
 for(let col=0;col<4;col++) body+=text(16+col*220,58,names[col],14);
 for(let row=0;row<4;row++){
  const y=88+row*142;
  body+=text(16,y,rows[row],13,"#a9b5c2");
  for(let col=0;col<4;col++){
   const png=await render(cases[col],kind,row);
   await writeFile(dir+"/candidate-"+kind+"-"+row+"-"+col+".png",png);
   layers.push({input:png,left:16+col*220,top:y+12});
  }
 }
 body+=text(16,680,"Production renderer + layouts; native text approximated. Synthetic backgrounds. Presets are not installed.",12,"#a9b5c2");
 await sharp(svg(896,705,body)).composite(layers).png().toFile(dir+"/comparison-"+(kind?"combo":"single")+"-1x.png");
}
// Export complete candidate palettes, ready for later review/implementation.
await writeFile(dir+"/palettes.json",JSON.stringify(cases.map((c,i)=>({name:names[i],palette:c.settings.dial.styles[0].palette})),null,2));
console.log("Rendered wallpaper presets for single and Combo; exported palettes.json");
