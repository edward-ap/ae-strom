import { Button,IconButton,Panel,Form,TextInput,SelectInput,TrackBar,ScrollingLabel,BaseControl } from '../web/vendor/taet-ui.js';
import { createRadioUI } from '../web/radio.js';
import type { Bridge,Action } from '../web/contracts.js';
declare const bridge:Bridge;
createRadioUI(document.body,bridge);
const input=new TextInput({name:'url',inputType:'text',spellcheck:false,autocomplete:'off'});
new Form().row(new Button({text:'1'}),input,new SelectInput({valueField:'name',textField:'name'}));
const track=new TrackBar({min:-15,max:15,orientation:'vertical',value:0});track.value.set(2,{silent:true});
new ScrollingLabel({text:'Synthetic',speed:28}).setPaused(true);
const owner:BaseControl=new Panel();void owner;
const valid:Action={type:'setStationUrl',bank:0,slot:5,url:'https://synthetic.invalid/'};bridge.dispatch(valid);
// @ts-expect-error domain action fields are exact
bridge.dispatch({type:'selectStation',bank:'A',slot:5});

import { SpectrumAnalyzer } from '../web/extensions/SpectrumAnalyzer.js';
new SpectrumAnalyzer({bands:5,width:62,height:24}).setLevels([0,.2,.4,.6,1]);

// The facade mirrors the published foundation + components profile.
import { icons } from '../web/vendor/taet-ui.js';
icons.register('synthetic-test-icon', {viewBox:'0 0 24 24',nodes:[['path',{d:'M4 4 20 20',stroke:'currentColor'}]]});
new IconButton({icon:'synthetic-test-icon',pressed:false,tooltip:'Synthetic'}).setPressed(true).setIcon('synthetic-test-icon');

new TrackBar({thumbShape:'grip',min:0,max:100,value:50}).setThumbShape('grip');

new TrackBar({orientation:'vertical',scale:'none',thumbShape:'grip',showValue:true,valuePosition:'top',valueFormatter:value=>`${value} dB`});
