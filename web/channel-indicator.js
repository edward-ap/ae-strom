import { BaseControl } from './vendor/taet-ui.js';
/** Application-owned composite; Tæt owns lifecycle, this class owns its markup. */
export class ChannelIndicator extends BaseControl {
 constructor(options={}) { super(options); this.bank=0;this.slot=0;this.flashTimer=0; }
 createElement() {
  const element=document.createElement('span');element.className='dui-control channel-indicator';element.setAttribute('aria-live','polite');
  this.bankNode=document.createElement('span');this.slotNode=document.createElement('span');this.slotNode.className='channel-slot';element.append(this.bankNode,this.slotNode);return element;
 }
 setChannel(bank,slot,flash=false) {
  this.bank=bank;this.slot=slot;
  if(!this.element)return this;
  const name=String.fromCharCode(65+bank),digit=String((slot+1)%10);
  this.bankNode.textContent=name;this.slotNode.textContent=digit;this.element.setAttribute('aria-label',`Bank ${name}, station ${digit}`);
  if(flash){this.element.classList.add('channel-flash');clearTimeout(this.flashTimer);this.flashTimer=setTimeout(()=>this.element?.classList.remove('channel-flash'),130);}
  return this;
 }
 destroy(){clearTimeout(this.flashTimer);super.destroy();}
}
