import {BaseControl} from '../vendor/taet-ui.js';
import {spectrumSegments,spectrumInteger,spectrumRowCounts} from './spectrumGeometry.js';

const colorKeys={background:'background',idleColor:'idle',baselineColor:'baseline',peakColor:'peak',lineColor:'line'};
/** Display only: host supplies ordered band amplitudes; no audio capture or FFT. */
export class SpectrumAnalyzer extends BaseControl {
  constructor(containerOrOptions={},maybeOptions) {
    const mounted=typeof containerOrOptions==='string'||(typeof HTMLElement!=='undefined'&&containerOrOptions instanceof HTMLElement);
    const options=mounted?maybeOptions??{}:containerOrOptions;
    super({...options,height:options.height??120});
    this.bands=spectrumInteger(options.bands??5,'bands',1,2048);
    this.mode=mode(options.mode??'segments');
    this.maxFps=spectrumInteger(options.maxFps??60,'maxFps',1,120);
    this.maxLevel=options.maxLevel??1;
    if(!Number.isFinite(this.maxLevel)||this.maxLevel<=0)throw new RangeError('maxLevel must be positive and finite.');
    if(options.rows!=null&&options.rowCounts!=null)throw new TypeError('Choose rows or rowCounts, not both.');
    this.rowCounts=Object.freeze([...(options.rowCounts??spectrumRowCounts(options.rows??13))]);
    if(this.rowCounts.length!==3)throw new TypeError('rowCounts must contain three counts.');
    this.rowCounts.forEach(n=>spectrumInteger(n,'rowCounts',1,32));
    if(options.heightWeights!=null&&(!Array.isArray(options.heightWeights)||options.heightWeights.length!==4||options.heightWeights.some(n=>!Number.isFinite(n)||n<=0||n>100)))throw new RangeError('heightWeights requires four positive weights up to 100: baseline, low, middle, high.');
    this.heightWeights=options.heightWeights==null?null:Object.freeze([...options.heightWeights]);
    this.paused=options.paused===true;
    this._levels=new Float64Array(this.bands);
    this._colors=colors(options);
    this._frame=null;this._timer=null;this._lastDraw=-Infinity;
    this._dirty=true;this._cacheDirty=true;this._intersecting=true;
  
    if(mounted)this.mount(containerOrOptions);
  }
  createElement() {
    const root=document.createElement('div');root.className=['dui-control','dui-spectrum',this.options.css].filter(Boolean).join(' ');
    root.dataset.duiControlId=this.id;root.setAttribute('role','img');
    root.setAttribute('aria-label',this.options.label??'Audio spectrum');
    this._canvas=document.createElement('canvas');this._canvas.setAttribute('aria-hidden','true');root.append(this._canvas);
    this._context=this._canvas.getContext('2d');
    this._backdrop=document.createElement('canvas');this._backContext=this._backdrop.getContext('2d');
    const resize=new ResizeObserver(()=>this._invalidateCache());resize.observe(root);this.disposables.add(()=>resize.disconnect());
    const intersection=new IntersectionObserver(entries=>{
      this._intersecting=entries[entries.length-1].isIntersecting;
      if(this._intersecting)this._schedule();else this._cancel();
    });intersection.observe(root);this.disposables.add(()=>intersection.disconnect());
    this.disposables.addEventListener(document,'visibilitychange',()=>{if(document.hidden)this._cancel();else this._invalidateCache();});
    this.disposables.addEventListener(window,'resize',()=>this._invalidateCache());
    this._applyColors(root);return root;
  }
  mount(container) {
    const already=this.mounted;super.mount(container);
    if(!this.destroyed&&!already) {
      // Theme changes normally mutate class/style on the scope or one of its ancestors.
      const theme=new MutationObserver(()=>this._invalidateCache());
      for(let node=this.element;node;node=node.parentElement)theme.observe(node,{attributes:true,attributeFilter:['class','style']});
      this.disposables.add(()=>theme.disconnect());
      this._draw();this._dirty=false;this._lastDraw=performance.now();
    }
    return this;
  }
  setLevels(values) {
    this._assertMutable('setLevels');
    if(!values||values.length!==this.bands)throw new RangeError(`Expected ${this.bands} spectrum levels.`);
    let changed=false;
    for(let i=0;i<this.bands;i++) {
      const v=typeof values[i]==='number'&&Number.isFinite(values[i])?Math.max(0,Math.min(this.maxLevel,values[i])):0;
      if(v!==this._levels[i]){this._levels[i]=v;changed=true;}
    }
    if(changed){this._dirty=true;this._schedule();}return this;
  }
  getLevels() {return Array.from(this._levels);}
  clear() {this._assertMutable('clear');this._levels.fill(0);this._dirty=true;this._schedule();return this;}
  setBandCount(bands) {
    this._assertMutable('setBandCount');this.bands=spectrumInteger(bands,'bands',1,2048);
    this._levels=new Float64Array(this.bands);this._invalidateCache();return this;
  }
  setRowCount(rows) {this._assertMutable('setRowCount');this.rowCounts=Object.freeze(spectrumRowCounts(rows));this._invalidateCache();return this;}
  setMode(value) {this._assertMutable('setMode');this.mode=mode(value);this._invalidateCache();return this;}
  setPaused(value) {
    this._assertMutable('setPaused');this.paused=value===true;
    if(this.paused)this._cancel();else this._schedule();return this;
  }
  setColors(value) {
    this._assertMutable('setColors');this._colors={...this._colors,...colors(value)};
    if(this.element)this._applyColors(this.element);this._invalidateCache();return this;
  }
  refresh() {super.refresh();this._invalidateCache();return this;}
  _applyColors(root) {for(const [key,value] of Object.entries(this._colors))root.style.setProperty(`--dui-spectrum-${key}`,value);}
  _invalidateCache() {this._cacheDirty=true;this._dirty=true;this._schedule();}
  _canDraw() {return this.mounted&&!this.destroyed&&!this.paused&&!this.disabled&&this.visible&&!document.hidden&&this._intersecting;}
  _schedule() {
    if(!this._dirty||!this._canDraw()||this._frame!==null||this._timer!==null)return;
    this._frame=requestAnimationFrame(timestamp=>{
      this._frame=null;if(!this._dirty||!this._canDraw())return;
      // Use frame timestamps, not paint completion: otherwise a 60 Hz input can
      // lose every second frame merely because drawing itself took some time.
      const interval=1000/this.maxFps, elapsed=timestamp-this._lastDraw;
      const remaining=interval-elapsed;
      if(remaining>.5){this._timer=setTimeout(()=>{this._timer=null;this._schedule();},remaining);return;}
      this._draw();
      // Retain sub-frame phase after a skipped frame (not an audio-frame queue).
      // Coarsely rounded browser timestamps must not force permanent 30 Hz output.
      this._lastDraw=timestamp-(Math.max(0,elapsed-interval)%interval);this._dirty=false;
    });
  }
  _cancel() {if(this._frame!==null)cancelAnimationFrame(this._frame);if(this._timer!==null)clearTimeout(this._timer);this._frame=this._timer=null;}
  _applyVisibilityState() {super._applyVisibilityState();if(!this.visible)this._cancel();else this._schedule();}
  _applyDisabledState() {super._applyDisabledState();if(this.disabled)this._cancel();else this._schedule();}
  _prepare() {
    const width=this.element.clientWidth,height=this.element.clientHeight;
    if(!width||!height)return false;
    // Bound both backing stores even when the host accidentally creates an enormous panel.
    const ratio=Math.min(window.devicePixelRatio||1,2,2048/width,2048/height);
    const pw=Math.max(1,Math.round(width*ratio)),ph=Math.max(1,Math.round(height*ratio));
    this._canvas.width=this._backdrop.width=pw;this._canvas.height=this._backdrop.height=ph;
    this._width=width;this._height=height;
    this._context.setTransform(pw/width,0,0,ph/height,0,0);
    this._backContext.setTransform(pw/width,0,0,ph/height,0,0);
    const style=getComputedStyle(this.element);
    const read=(key,fallback)=>style.getPropertyValue(`--dui-spectrum-${key}`).trim()||fallback;
    this._palette={background:read('background','#081e28'),idle:read('idle','#163b46'),baseline:read('baseline',read('low','#00a99d')),
      active:[read('low','#00a99d'),read('mid','#34cbbd'),read('high','#8ae1d4')],peak:read('peak','#f35c65'),line:read('line','#34cbbd')};
    this._geometry=spectrumSegments(height,this.rowCounts,this.heightWeights);
    this._step=width/this.bands;this._barWidth=this._step*.66;
    // Cache rounded geometry once per row, shared by every band. No path
    // construction or per-band object allocation in the audio update loop.
    this._segmentPaths=this._geometry.segments.map(segment=>{
      const path=new Path2D();
      path.roundRect(0,segment.y,this._barWidth,segment.height,Math.min(2,this._barWidth/2,segment.height/2));
      return path;
    });
    const basePath=new Path2D();
    basePath.roundRect(0,height-this._geometry.base,this._barWidth,this._geometry.base,Math.min(2,this._barWidth/2,this._geometry.base/2));
    const context=this._backContext;
    context.clearRect(0,0,width,height);context.fillStyle=this._palette.background;context.fillRect(0,0,width,height);
    if(this.mode==='segments') {
      context.fillStyle=this._palette.idle;
      for(let band=0;band<this.bands;band++) {
        const x=this._step*(band+.17);
        context.save();context.translate(x,0);
        context.fillStyle=this._palette.idle;
        for(const path of this._segmentPaths)context.fill(path);
        context.fillStyle=this._palette.baseline;context.fill(basePath);context.restore();
      }
    } else {context.fillStyle=this._palette.idle;context.fillRect(0,height-1,width,1);}
    this._cacheDirty=false;return true;
  }
  _draw() {
    if(this._cacheDirty&&!this._prepare())return;
    const ctx=this._context,w=this._width,h=this._height;
    if(!w||!h)return;
    ctx.clearRect(0,0,w,h);ctx.drawImage(this._backdrop,0,0,w,h);
    if(this.mode==='line') {
      ctx.strokeStyle=this._palette.line;ctx.lineWidth=1.5;ctx.beginPath();
      for(let i=0;i<this.bands;i++) {
        const x=this.bands===1?w/2:1+(w-2)*i/(this.bands-1),y=1+(h-2)*(1-this._levels[i]/this.maxLevel);
        if(i===0)ctx.moveTo(x,y);else ctx.lineTo(x,y);
      }
      if(this.bands===1){ctx.lineTo(w/2+1,1+(h-2)*(1-this._levels[0]/this.maxLevel));}
      ctx.stroke();return;
    }
    const segments=this._geometry.segments;
    ctx.save();ctx.translate(this._step*.17,0);
    for(let band=0;band<this.bands;band++) {
      const level=this._levels[band]/this.maxLevel;
      for(let row=0;row<segments.length;row++) {
        const s=segments[row];if(level<s.threshold)break;
        ctx.fillStyle=row===segments.length-1?this._palette.peak:this._palette.active[s.zone];
        ctx.fill(this._segmentPaths[row]);
      }
      ctx.translate(this._step,0);
    }
    ctx.restore();
  }
  destroy() {
    if(this.destroyed||this._destroying)return;this._cancel();super.destroy();
    if(this._canvas){this._canvas.width=this._canvas.height=1;this._backdrop.width=this._backdrop.height=1;}
    this._canvas=this._context=this._backdrop=this._backContext=null;this._levels=new Float64Array(0);this._segmentPaths=null;
  }
}
function mode(value){if(!['segments','line'].includes(value))throw new TypeError('Spectrum mode must be segments or line.');return value;}
function colors(options) {
  const result={};
  for(const [key,token] of Object.entries(colorKeys))if(options[key]!=null){if(typeof options[key]!=='string'||!options[key].trim())throw new TypeError(`${key} must be a CSS color.`);result[token]=options[key];}
  if(options.activeColors!=null){if(!Array.isArray(options.activeColors)||options.activeColors.length!==3||options.activeColors.some(c=>typeof c!=='string'||!c.trim()))throw new TypeError('activeColors requires three CSS colors.');['low','mid','high'].forEach((key,i)=>result[key]=options.activeColors[i]);}
  return result;
}
