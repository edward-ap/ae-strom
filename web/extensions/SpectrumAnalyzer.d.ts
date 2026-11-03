import {BaseControl} from '../vendor/taet-ui.js';
import type {BaseControlOptions} from '../vendor/taet-ui.js';
export interface SpectrumColors {
 background?:string;idleColor?:string;baselineColor?:string;activeColors?:readonly [string,string,string];peakColor?:string;lineColor?:string;
}
export interface SpectrumAnalyzerOptions extends BaseControlOptions<SpectrumAnalyzer>,SpectrumColors {
 bands?:number;mode?:'segments'|'line';maxLevel?:number;maxFps?:number;
 /** Relative heights: baseline, low, middle, high. Positive values up to 100. */
 heightWeights?:readonly [number,number,number,number];
 /** Active rows, 3-96; mutually exclusive with rowCounts. Baseline is extra. */
 rows?:number;rowCounts?:readonly [number,number,number];paused?:boolean;label?:string;css?:string;
}
export declare class SpectrumAnalyzer extends BaseControl<{},HTMLElement> {
 constructor(options?:SpectrumAnalyzerOptions);
 constructor(container:HTMLElement|string,options?:SpectrumAnalyzerOptions);
 readonly bands:number;readonly mode:'segments'|'line';readonly maxLevel:number;readonly maxFps:number;
 readonly heightWeights:readonly number[]|null;readonly paused:boolean;readonly rowCounts:readonly number[];
 setLevels(values:ArrayLike<number>):this;getLevels():number[];clear():this;
 setRowCount(rows:number):this;setBandCount(bands:number):this;setMode(mode:'segments'|'line'):this;setPaused(paused:boolean):this;
 setColors(colors:SpectrumColors):this;refresh():this;
}
