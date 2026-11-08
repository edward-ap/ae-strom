/** Explicit synthetic preview: no requests, audio, user config, or local storage. */
export function createPreviewBridge() {
 const stateListeners = new Set(), spectrumListeners = new Set(), actions = [], layouts = [];
 let timer, sequence = 0, destroyed = false;
 let state = { revision: 1, generation: 1, mode: 'fake', status: 'stopped', requestedPlaying: false, fullTitle: 'Synthetic preview · no audio · Æ Strøm', displayTitle: 'Synthetic preview · no audio · Æ Strøm', activeBank: 0, activeSlot: 0, volume: 80, muted: false, stations: Array.from({length:260}, (_, i) => ({name:`Synthetic station ${i+1}`,url:`https://synthetic.invalid/station/${i+1}`,eq:'Flat',metadataType:'',metadataUrl:''})), eqPresets: [{name:'Flat',preamp:0,bands:Array(10).fill(0),builtin:true},{name:'Rock',preamp:0,bands:[3,2,1,0,-1,-1,0,1,2,3],builtin:true}], eq: {name:'Flat',preamp:0,bands:Array(10).fill(0),dirty:false,canSave:false,canDelete:false}, error:null };
 const clone = value => structuredClone(value);
 const emit = () => { state.revision++; stateListeners.forEach(fn => fn(clone(state))); return clone(state); };
 const eqState = preset => ({...clone(preset),dirty:false,canSave:!preset.builtin,canDelete:!preset.builtin});
 const bridge = {
   actions, layouts, readySignals: [], copies: [], dragCount: 0, closed: false,
   getSnapshot: async () => clone(state),
   uiReady: async info => { bridge.readySignals.push(clone(info)); },
   subscribeState: fn => { stateListeners.add(fn); return () => stateListeners.delete(fn); },
   subscribeSpectrum: fn => { spectrumListeners.add(fn); return () => spectrumListeners.delete(fn); },
   async dispatch(action) {
     actions.push(clone(action));
     switch(action.type) {
       case 'setPlaying': state.requestedPlaying=action.playing;state.status=action.playing?'playing':'stopped';state.generation++;break;
       case 'setVolume':state.volume=Math.max(0,Math.min(100,action.volume));break;
       case 'setMuted':state.muted=action.muted;break;
       case 'selectStation':state.activeBank=action.bank;state.activeSlot=action.slot;state.generation++;break;
       case 'setStationUrl':Object.assign(state.stations[action.bank*10+action.slot],{url:action.url.trim(),name:'',metadataType:'',metadataUrl:''});break;
       case 'assignStationEq':state.stations[action.bank*10+action.slot].eq=action.name;break;
       case 'selectEqPreset': {const selected=state.eqPresets.find(p=>p.name===action.name);if(selected) state.eq=eqState(selected);break;}
       case 'setEq':state.eq={...state.eq,preamp:action.preamp,bands:clone(action.bands),dirty:true,canSave:true};break;
       case 'saveEqPreset': {if(!action.name || state.eqPresets.some(p=>p.name===action.name&&p.builtin)) throw Error('Enter a custom preset name');const preset={name:action.name,preamp:state.eq.preamp,bands:clone(state.eq.bands),builtin:false};state.eqPresets=state.eqPresets.filter(p=>p.name!==action.name).concat(preset);state.eq=eqState(preset);break;}
       case 'deleteEqPreset':state.eqPresets=state.eqPresets.filter(p=>p.name!==action.name||p.builtin);state.eq=eqState(state.eqPresets[0]);break;
       default:throw Error(`Unknown preview action: ${action.type}`);
     }
     return emit();
   },
   prepareWindowLayout:async layout=>({panelsAbove:!layout.compact&&layout.height>36&&bridge.panelsAbove===true}),
   setWindowLayout:async layout=>{layouts.push(clone(layout));return {panelsAbove:!layout.compact&&layout.height>36&&bridge.panelsAbove===true};},
   startDrag:async()=>{bridge.dragCount++;},
   copyTitle:async()=>{bridge.copies.push(state.fullTitle);},
   shutdown:async()=>{bridge.closed=true;},
   update: patch=>{Object.assign(state,clone(patch));return emit();},
   deliverSpectrum: event=>spectrumListeners.forEach(fn=>fn(clone(event))),
   spectrum: levels=>bridge.deliverSpectrum({generation:state.generation,levels:clone(levels),powered:true}),
   destroy:()=>{destroyed=true;clearInterval(timer);stateListeners.clear();spectrumListeners.clear();},
 };
 timer=setInterval(()=>{if(!destroyed&&!bridge.freezeSpectrum&&state.requestedPlaying){sequence++;bridge.spectrum(Array.from({length:5},(_,i)=>(Math.sin(sequence*.12+i)+1)*.45));}},50);
 return bridge;
}
