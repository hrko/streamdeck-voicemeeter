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
async function render(c,kind,bg,opacity,matched,outline,palette) {
 const frame=c.frames[kind], layers=[];
 for(const item of [...layouts[kind].items].sort((a,b)=>a.zOrder-b.zOrder)) {
  const p=frame[item.key]; if(!p) continue;
  const background=item.key.startsWith("background");
  if(!(background&&opacity!==undefined) && !p.enabled) continue;
  const [x,y,w,h]=item.rect;
  let input;
  if(item.type==="pixmap") {
   let source=Buffer.from(p.value.split(",")[1],"base64");
   if(item.key.startsWith("status")){
    let markup=source.toString();
    if(!outline) {
     markup=markup.replace(/<rect x="([^"]+)" y="([^"]+)" width="(4|6)" height="(4|6)" rx="[^"]+" fill="#2c3d4d" stroke="#91a4b5" stroke-width="1"\/>/g,
      (_,x,y,w)=>'<rect x="'+(Number(x)-.5)+'" y="'+(Number(y)-.5)+'" width="'+(Number(w)+1)+'" height="'+(Number(w)+1)+'" rx="'+(w==="4"?2:1.5)+'" fill="#2c3d4d"/>');
    }
    if(matched) markup=markup.replaceAll('fill="#2c3d4d"','fill="black" fill-opacity="0.623529"');
    source=Buffer.from(markup);
   }
   if(background && palette==="vm") source=svg(w,h,'<rect width="'+w+'" height="'+h+'" fill="#2c3d4d"/>');
   if(item.key.startsWith("gainSlider") && palette==="white") source=Buffer.from(source.toString().replaceAll("#70c399","#ffffff"));

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

const names=["Current / outline","Matched / outline","Current / no outline","Matched / no outline"];
for(const kind of [0,1]){
 let body='<rect width="100%" height="100%" fill="#10151c"/>'+text(16,26,(kind?"Combo":"Single")+" / inactive indicator fill comparison / 1x",19);
 const layers=[];
 for(let col=0;col<4;col++) body+=text(16+col*220,56,names[col],13);
 for(let row=0;row<3;row++){
  const y=86+row*145;
  body+=text(16,y,["Black background (100%)","VoiceMeeter background (100%)","Bright wallpaper / black background (40%)"][row],13,"#a9b5c2");
  for(let col=0;col<4;col++){
   const png=await render(cases[0],kind,row===2?1:0,row===2?.4:1,col%2===1,col<2,row===1?"vm":"white");
   await writeFile(dir+"/indicator-fill-"+kind+"-"+row+"-"+col+".png",png);
   layers.push({input:png,left:16+col*220,top:y+12});
  }
 }
 await sharp(svg(896,520,body)).composite(layers).png().toFile(dir+"/indicator-fill-"+(kind?"combo":"single")+"-1x.png");
}
let body='<rect width="100%" height="100%" fill="#10151c"/>'+text(16,26,"Inactive indicator fill / VoiceMeeter background / 8x",20);
const layers=[];
for(let col=0;col<4;col++){
 body+=text(16+col*310,62,names[col],15);
 const input=await sharp(await readFile(dir+"/indicator-fill-1-1-"+col+".png")).extract({left:8,top:40,width:36,height:24}).resize(288,192,{kernel:"nearest"}).png().toBuffer();
 layers.push({input,left:16+col*310,top:85});
}
await sharp(svg(1256,300,body)).composite(layers).png().toFile(dir+"/indicator-fill-detail.png");
console.log("Created indicator-fill-{single,combo}-1x.png and indicator-fill-detail.png");
