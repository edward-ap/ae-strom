import type { Bridge,Snapshot } from './contracts.js';
export declare function createRadioUI(host:HTMLElement,bridge:Bridge):{ready:Promise<void>;getState():{snapshot:Snapshot|null;compact:boolean;panel:string|null;lastPanel:string|null;browsedBank:number;drafts:Record<string,string>};setCompact(value:boolean):void;showPanel(panel:'settings'|'eq'):void;destroy():void};
