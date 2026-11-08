import { inspectTypography } from './font-audit.js';
/** Programmatic native IPC smoke. This is not native pointer/clipboard acceptance. */
export async function runNativeSmoke(ui, bridge) {
 const checks=[], initial=await bridge.getSnapshot();
 const layoutFrames=[], originalSetLayout=bridge.setWindowLayout.bind(bridge);
 bridge.setWindowLayout=async (...args)=>{
  const sample=()=>{const bar=document.querySelector('.player-strip'),owner=document.querySelector('.radio-owner');if(!bar||!owner)return;const r=bar.getBoundingClientRect();layoutFrames.push({time:performance.now(),above:owner.classList.contains('panels-above'),height:window.innerHeight,top:r.top,bottom:r.bottom});};
  let active=true;const frame=()=>{if(active){sample();requestAnimationFrame(frame);}};
  sample();requestAnimationFrame(frame);
  try{return await originalSetLayout(...args);}finally{sample();active=false;}
 };
 const check=(name,pass,details)=>checks.push({name,pass:Boolean(pass),details});
 try {
  check('Æ Strøm document identity',document.title==='Æ Strøm',{});
  check('startup reveal handshake completed',window.__AE_STROM_READY__===true,{});
  check('standard bundled font loaded',document.fonts.check('12px "Noto Sans"','Æ Strøm Жж')&&[...document.fonts].some(f=>f.family.includes('Noto Sans')&&f.status==='loaded')&&[...document.querySelectorAll('.station-row input,.station-row select,.radio-ticker,.player-strip button')].every(e=>getComputedStyle(e).fontFamily.includes('Noto Sans')&&getComputedStyle(e).fontSize==='12px'),{});
  const typography=inspectTypography();
  check('complete app typography and bundled face',typography.standard&&typography.loaded&&typography.nativeSelectCount===11,{typography});
  check('synthetic backend',initial.mode==='fake',{mode:initial.mode});
  if(initial.mode!=='fake')throw Error('Smoke refuses live backend');
  check('startup geometry',document.querySelector('.player-strip').getBoundingClientRect().height===36,{});
  const idleTicker=document.querySelector('.radio-ticker'),idleText=idleTicker.querySelector('.dui-scrolling-label__text'),idleRect=idleTicker.getBoundingClientRect(),idleTextRect=idleText.getBoundingClientRect();
  check('static centered idle brand',idleText.textContent==='-= Æ Strøm =-'&&idleTicker.classList.contains('radio-ticker--idle')&&Math.abs((idleTextRect.left+idleTextRect.right-idleRect.left-idleRect.right)/2)<.75&&!idleTicker.getAnimations({subtree:true}).some(a=>a.playState==='running'),{centerError:Math.abs((idleTextRect.left+idleTextRect.right-idleRect.left-idleRect.right)/2)});
  const playing=await bridge.dispatch({type:'setPlaying',playing:true});
  check('play snapshot',playing.requestedPlaying,{generation:playing.generation});
  const selected=await bridge.dispatch({type:'selectStation',bank:25,slot:9});
  check('physical bank DTO',selected.activeBank===25&&selected.activeSlot===9,{});
  await bridge.dispatch({type:'setVolume',volume:65});await bridge.dispatch({type:'setMuted',muted:true});
  const button = name => document.querySelector(`[data-control="${name}"]`);
  const pressed = name => button(name).getAttribute('aria-pressed') === 'true';
  check('smaller toolbar SVGs', [...document.querySelectorAll('.player-strip button')].every(e => { const svg=e.querySelector('svg');return svg && svg.getBoundingClientRect().width >= (e.dataset.control==='collapse'?13:18) && getComputedStyle(svg).color !== 'rgb(32, 32, 32)'; }), {});
  check('shared blue display',getComputedStyle(document.querySelector('.radio-display')).backgroundColor==='rgb(0, 61, 102)'&&getComputedStyle(document.querySelector('.channel-indicator')).borderTopColor==='rgb(255, 255, 255)',{});
  const channel=document.querySelector('.channel-indicator'),channelBank=channel.firstElementChild,channelCircle=channel.querySelector('.channel-slot'),channelRect=channel.getBoundingClientRect(),bankRect=channelBank.getBoundingClientRect(),circleRect=channelCircle.getBoundingClientRect();
  check('compact channel geometry',Math.abs(channelRect.width-40)<.75&&Math.abs(channelRect.height-26)<.75&&Math.abs(bankRect.width-14)<.75&&Math.abs(circleRect.width-17)<.75&&Math.abs(circleRect.height-17)<.75&&Math.abs(channelRect.right-circleRect.right-3)<.75&&Math.abs(circleRect.left-bankRect.right-2)<.75,{frame:[channelRect.width,channelRect.height],bankWidth:bankRect.width,circle:[circleRect.width,circleRect.height],rightInsideFrame:channelRect.right-circleRect.right,bankCircleGap:circleRect.left-bankRect.right});
  check('public volume grip variant',document.querySelector('.radio-volume').dataset.thumbShape==='grip',{});
  check('narrow five-band spectrum',document.querySelector('.radio-spectrum').getBoundingClientRect().width===42&&document.querySelector('.radio-spectrum').getBoundingClientRect().height===24,{});
  await ui.showPanel('settings');
  check('settings pressed matches visibility',pressed('settings')&&!pressed('equalizer')&&!document.querySelector('.settings-panel').hidden,{});
  { const bar=document.querySelector('.player-strip').getBoundingClientRect(),panel=document.querySelector('.settings-panel').getBoundingClientRect(),above=document.querySelector('.radio-owner').classList.contains('panels-above');check('Settings follows native opening direction',above?panel.bottom<=bar.top+.75:panel.top>=bar.bottom-.75,{above}); }
  check('settings native inputs',document.querySelectorAll('[data-station-slot]').length===10,{});
  check('paired Settings column labels',document.querySelectorAll('.station-header').length===2&&[...document.querySelectorAll('.station-header .column-label')].map(e=>e.textContent).join('|')==='Key|Stream URL|EQ Preset|Key|Stream URL|EQ Preset',{});
  const bankRow=document.querySelector('.bank-strip').getBoundingClientRect(),columns=document.querySelector('.station-columns').getBoundingClientRect();
  const settingsHeight=document.querySelector('.radio-owner').getBoundingClientRect().height;
  check('banks below compact Settings content',bankRow.top>=columns.bottom-.75&&Math.abs(settingsHeight-258)<=.75&&!document.querySelector('.bank-info'),{settingsHeight,bankTop:bankRow.top,columnsBottom:columns.bottom,cssTolerance:.75});
  check('equal framed bank squares',[...document.querySelectorAll('.bank-strip button')].every(e=>e.getBoundingClientRect().width===24&&e.getBoundingClientRect().height===24&&getComputedStyle(e).borderTopColor==='rgb(114, 119, 128)'),{});
  check('native Settings select workaround', [...document.querySelectorAll('.station-row select')].every(e=>Math.abs(e.getBoundingClientRect().width-110)<.75&&getComputedStyle(e).paddingInlineEnd==='25px'&&getComputedStyle(e).appearance==='none'&&Math.abs(parseFloat(getComputedStyle(e.parentElement,'::after').width)-e.getBoundingClientRect().height)<.75&&getComputedStyle(e.parentElement,'::after').height==='24px'),{columnWidth:110,paddingEnd:25,squareArea:24,textGap:2,appearance:"none",nativeSelect:true});
  document.querySelector(`[data-control="bank${ui.getState().snapshot.activeBank}"]`).click();
  const selectedBank=document.querySelector('.bank-strip button[aria-pressed="true"]'),selectedSlot=document.querySelector('.station-row button[aria-checked="true"]');
  check('selected bank and slot ticker blue', [selectedBank,selectedSlot].every(e=>e&&getComputedStyle(e).backgroundColor==='rgb(0, 61, 102)'&&getComputedStyle(e).color==='rgb(255, 255, 255)'&&getComputedStyle(e).borderTopColor==='rgb(114, 119, 128)'),{});
  document.querySelector('[data-control="bank0"]').click();

  // Native DOM routing only: do not pretend synthetic pointers are an OS-held
  // mouse button. Stub the host boundary so no unsafe fake OS move is issued.
  const originalDrag=bridge.startDrag;let routed=0;
  try {
    bridge.startDrag=()=>{routed++;return Promise.resolve();};
    const gesture=target=>{target.dispatchEvent(new PointerEvent('pointerdown',{bubbles:true,button:0,buttons:1,pointerId:51,clientX:100,clientY:18}));for(const x of [108,116])document.dispatchEvent(new PointerEvent('pointermove',{bubbles:true,buttons:1,pointerId:51,clientX:x,clientY:18}));document.dispatchEvent(new PointerEvent('pointerup',{bubbles:true,pointerId:51}));};
    gesture(document.querySelector('.radio-ticker'));gesture(document.querySelector('.channel-slot'));gesture(button('play'));gesture(document.querySelector('.radio-volume input'));gesture(document.querySelector('[data-station-slot]'));
    check('native drag routing with host stub',routed===2,{actualOSDrag:false,hostStub:true});
  } finally {bridge.startDrag=originalDrag;}

  const field=document.querySelector('[data-station-slot="5"]');field.focus();field.value='https://synthetic.invalid/'+ 'long-path/'.repeat(70);field.dispatchEvent(new Event('input',{bubbles:true}));field.setSelectionRange(12,90);
  const draft=field.value;
  await ui.setCompact(true);
  check('compact clears hidden panel pressed states',!pressed('settings')&&!pressed('equalizer'),{});
  check('compact exactly three elements',[...document.querySelector('.player-strip').children].filter(e=>getComputedStyle(e).display!=='none').length===3,{});
  check('compact content width',document.querySelector('.player-strip').getBoundingClientRect().width===102,{width:document.querySelector('.player-strip').getBoundingClientRect().width});
  await ui.setCompact(false);
  check('expand restores settings pressed state',pressed('settings')&&!pressed('equalizer'),{});
  check('draft persists',field.value===draft,{length:field.value.length});
  await ui.showPanel('eq');
  check('panel switch synchronizes pressed states',!pressed('settings')&&pressed('equalizer'),{});
  { const bar=document.querySelector('.player-strip').getBoundingClientRect(),panel=document.querySelector('.eq-panel').getBoundingClientRect(),above=document.querySelector('.radio-owner').classList.contains('panels-above');check('EQ follows native opening direction',above?panel.bottom<=bar.top+.75:panel.top>=bar.bottom-.75,{above}); }
  check('11 sliders',document.querySelectorAll('.eq-sliders input[type="range"]').length===11,{});
  const eqControls=[...document.querySelectorAll('.eq-sliders .dui-trackbar')],eqInputs=eqControls.map(e=>e.querySelector('input'));
  const eqHeight=document.querySelector('.radio-owner').getBoundingClientRect().height;
  check('EQ reduced height and bottom labels',Math.abs(eqHeight-257)<=.75&&!document.querySelector('.eq-panel .panel-hint')&&eqControls.every(e=>e.querySelector('.dui-trackbar__label').getBoundingClientRect().top>=e.querySelector('input').getBoundingClientRect().bottom+1.5),{eqHeight,labels:eqControls.map(e=>e.querySelector('.dui-trackbar__label').textContent)});
  const eqInputRect=eqInputs[0].getBoundingClientRect(),grid=[...document.querySelectorAll('.eq-grid-line')];
  check('shared EQ grid matches native thumb travel',grid.length===7&&grid.filter(e=>e.classList.contains('eq-grid-line--major')).length===3&&grid.every(e=>{const r=e.getBoundingClientRect(),db=Number(e.dataset.eqDb);return Math.abs(r.y+r.height/2-(eqInputRect.y+12+(15-db)/30*(eqInputRect.height-24)))<.75;}),{thumbHeight:22,trackBorder:1,inputHeight:eqInputRect.height,values:grid.map(e=>Number(e.dataset.eqDb)),realOSInput:false});
  check('stock dark EQ grips and no individual ticks',eqControls.every(e=>e.dataset.thumbShape==='grip'&&getComputedStyle(e).getPropertyValue('--dui-thumb-background').includes('#33363c'))&&document.querySelectorAll('.eq-sliders .dui-trackbar__tick').length===0&&getComputedStyle(document.querySelector('.radio-volume')).getPropertyValue('--dui-thumb-background').includes('#33363c'),{});
  eqInputs[0].focus();
  check('EQ focus does not show value hints',eqControls.every(e=>!e.classList.contains('dui-trackbar--show-value')&&getComputedStyle(e.querySelector('.dui-trackbar__value')).display==='none'),{programmaticFocus:true,realOSInput:false});
  button('equalizer').focus();
  ui.showPanel(null);
  const originalDispatch=bridge.dispatch;
  try {
    const status=document.querySelector('.radio-status');
    // Await the actual rejected-action rendering, rather than assuming one RAF
    // drains promises/native IPC left by earlier focus/blur interactions.
    const errorShown=new Promise((resolve,reject)=>{
      const observer=new MutationObserver(()=>{
        if(status.textContent.includes('Synthetic native error')){observer.disconnect();clearTimeout(deadline);resolve();}
      });
      const deadline=setTimeout(()=>{observer.disconnect();reject(Error('Synthetic error did not render before the test deadline'));},3000);
      observer.observe(status,{subtree:true,childList:true,characterData:true});
    });
    bridge.dispatch=async()=>{throw Error('Synthetic native error');};button('play').click();await errorShown;
    const ownerHeight=document.querySelector('.radio-owner').getBoundingClientRect().height,clip=getComputedStyle(status).clipPath,tickerError=document.querySelector('.radio-ticker').textContent.includes('Synthetic native error');
    check('errors never create a protruding row',Math.abs(ownerHeight-36)<.75&&clip==='inset(50%)'&&tickerError,{ownerHeight,clip,tickerError,statusWidth:status.getBoundingClientRect().width,statusHeight:status.getBoundingClientRect().height});
  } finally {bridge.dispatch=originalDispatch;}
  button('play').click();await new Promise(requestAnimationFrame);
  const final=await bridge.dispatch({type:'setPlaying',playing:false});
  const renderDeadline=performance.now()+3000;
  while(ui.getState().snapshot.revision<final.revision&&performance.now()<renderDeadline)await new Promise(requestAnimationFrame);
  check('Stop restores idle brand',document.querySelector('.radio-ticker .dui-scrolling-label__text').textContent==='-= Æ Strøm =-'&&document.querySelector('.radio-ticker').classList.contains('radio-ticker--idle'),{});
  check('stop snapshot',!final.requestedPlaying,{generation:final.generation});
  check('hover hints stay absent after panel and status changes',document.querySelectorAll('.radio-owner [title]:not([title=""])').length===0,{});
  const upwardFrames=layoutFrames.filter(frame=>frame.above);
  check('upward strip remains on viewport edge during layout',upwardFrames.every(frame=>Math.abs(frame.height-frame.bottom)<.75),{samples:upwardFrames.length});
  await window.__TAURI__.core.invoke('save_ui_evidence',{body:JSON.stringify({kind:'programmatic native fake-backend smoke',userInput:false,audio:false,typography,layoutFrames,checks,pass:checks.every(c=>c.pass),snapshot:{mode:final.mode,generation:final.generation,status:final.status}},null,2)});
 } catch(error) { await window.__TAURI__.core.invoke('save_ui_evidence',{body:JSON.stringify({pass:false,error:String(error),checks},null,2)}); }
}
