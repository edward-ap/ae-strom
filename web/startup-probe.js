/** Native startup/save/restart acceptance only. Never requests playback. */
export async function runStartupProbe(ui,bridge){
 const checks=[];const check=(name,pass,details={})=>checks.push({name,pass:Boolean(pass),details});
 try{
  const state=await bridge.getSnapshot();
  check('default live mode',state.mode==='live');
  check('starts stopped without autoplay',state.status==='stopped'&&!state.requestedPlaying);
  check('native reveal ready',window.__AE_STROM_READY__===true);
  check('idle title in default live mode',document.querySelector('.radio-ticker .dui-scrolling-label__text').textContent==='-= Æ Strøm =-'&&document.querySelector('.radio-ticker').classList.contains('radio-ticker--idle'));
  check('embedded standard font loaded',[...document.fonts].some(f=>f.family.includes('Noto Sans')&&f.status==='loaded'));
  check('persisted synthetic stations and custom EQ present',state.stations.filter(s=>s.url).length===2&&state.eqPresets.some(p=>p.name==='ProbeCurve'));
  const volume=(state.volume+1)%100;await bridge.dispatch({type:'setVolume',volume});await bridge.dispatch({type:'setMuted',muted:!state.muted});
  const after=await bridge.getSnapshot();
  check('settings action does not start audio',after.status==='stopped'&&!after.requestedPlaying&&after.volume===volume&&after.muted===!state.muted);
  const record={pass:checks.every(c=>c.pass),kind:'single executable default live startup/save probe',checks,initialVolume:state.volume,initialMuted:state.muted,savedVolume:after.volume,savedMuted:after.muted,mode:state.mode,autoplay:false,audio:false,userInput:false};
  await window.__TAURI__.core.invoke('save_ui_evidence',{body:JSON.stringify(record)});
 }catch(error){console.error(error);await bridge.shutdown();}
}
