"use strict";

import "./style.scss";

import { WorkerMessage, WorkerStatusMessage } from "@/shared/messages";
//import * as monaco from 'monaco-editor';
import * as monaco from 'monaco-editor/esm/vs/editor/editor.api';
import { IS_DIVISIBLE, LC3_OS } from "./examples";
import { Registers } from "@/../../virtual-machine/pkg/lc3vm";
import { LANGUAGE, THEME } from "./editor";

function decimalToHex(d: number, padding: number) {
    var hex = d.toString(16);
    while (hex.length < padding) {
        hex = "0" + hex;
    }

    return `0x${hex}`;
}

const debug_logging = false;
function log(...args: any[]) {
    if(debug_logging) {
        console.log(...args)
    }
}

monaco.languages.register({ id: 'lc3-assembly' });
monaco.languages.setMonarchTokensProvider('lc3-assembly', LANGUAGE as any);
monaco.editor.defineTheme('the-theme', THEME as any);

let editor = monaco.editor.create(document.getElementById('editor'), {
	value: LC3_OS,
	language: 'lc3-assembly',
    theme: 'the-theme'
});

// make editor read only



let decorations: string[] = [];
let previousRegisters: Record<Registers, number> = null;
let latestStatus: WorkerStatusMessage;

// TODO: This is not perfect, as it continuously requests animation
//       frames, even if the VM is not running. Could be optimized
//       at a later point in time.
// renderStatus();

function renderStatus() {
    if (latestStatus == null) {
        window.requestAnimationFrame(renderStatus);
        return;
    }

    let msg = latestStatus;

    if(previousRegisters == null) {
        previousRegisters = msg.registers;
    }

    for(const reg in msg.registers) {
        let el = document.getElementById(`${reg}-value`);
        el.textContent = decimalToHex(msg.registers[reg], 4);
        if(previousRegisters[reg] != msg.registers[reg]) {
            el.classList.add("is-danger");
        } else {
            el.classList.remove("is-danger");
        }
    }

    previousRegisters = msg.registers;

    decorations = editor.deltaDecorations(
        decorations,
        [
            {
                range: new monaco.Range(msg.source_line, 1, msg.source_line, 1),
                options: {
                    isWholeLine: true,
                    linesDecorationsClassName: 'myLineDecoration',
    
                }
            },
            {
                range: new monaco.Range(msg.source_line, 1, msg.source_line, 1),
                options: {
                    inlineClassName: 'myInlineDecoration',
                    isWholeLine: true,
                }
            }
        ]
    );

    editor.revealPositionInCenter({ lineNumber: msg.source_line, column: 0 });
}


const myWorker = new Worker("./worker.js");
myWorker.onmessage = function(e) {
    let msg: WorkerMessage = e.data;
    log(msg);

    switch(msg.kind) {
        case 'log':
            console.log(...msg.args);
            return

        case 'error':
            let out = document.querySelector("#output");
            out.textContent = msg.msg;
            return

        case 'status':
            latestStatus = msg;
            return;

        case 'paused':
            renderStatus();
            ui.eventPaused(msg.halted);
            return;

        case 'loaded':
            renderStatus();
            ui.eventLoaded();
            return;

        case 'output':
            ui.eventOutput(msg.character);
            return;

        default:
            console.log('Unknown message received from worker', e.data);
            return;

    }
}

class Ui {
    public state = {
        running: false,
    };

    buttons: Record<string, Element> = {};
    output: Element;

    constructor() {
        this.buttons = {
            run: document.querySelector("#run"),
            load: document.querySelector("#load"),
            step: document.querySelector("#step"),
            pause: document.querySelector("#pause"),
            reset: document.querySelector("#reset"),
        };
    
        this.buttons['run'].addEventListener("click", () => this.actionRun());
        this.buttons['load'].addEventListener("click", () => this.actionLoad());
        this.buttons['step'].addEventListener("click", () => this.actionStep());
        this.buttons['pause'].addEventListener("click", () => this.actionPause());
        this.buttons['reset'].addEventListener("click", () => this.actionReset());

        this.output = document.getElementById("character-output");
        this.output.addEventListener('keypress', (e: any) => this.eventInput(e.key));

        this.enableButtons("load");
    }

    eventPaused(halted: boolean) {
        this.state.running = false;
        if(halted) {
            this.enableButtons("reset");
        } else {
            this.enableButtons("reset", "run", "step");
        }
    }

    eventInput(character: string) {
        myWorker.postMessage({ kind: 'input', character: character.charCodeAt(0) });
    }

    eventOutput(character: number) {
        this.output.textContent += String.fromCharCode(character);
        this.output.scrollTop = this.output.scrollHeight;
    }
    
    eventLoaded() {
        this.enableButtons("reset", "run", "step");

        editor.updateOptions({
            readOnly: true
        });
    }

    // TODO: not sure if this is a good idea
    updateUiWhileRunning() {
        if(!this.state.running) {
            return;
        }

        renderStatus();

        window.requestAnimationFrame(
            () => this.updateUiWhileRunning()
        );
    }

    enableButtons(...enable: string[]) {
        let all = Object.keys(this.buttons);
        all.forEach(name => {
            this.buttons[name].classList.remove("is-loading");
            if(enable.includes(name)) {
                this.buttons[name].removeAttribute("disabled")
            } else {
                this.buttons[name].setAttribute("disabled", "");
            }
        })
    }

    actionRun() {
        myWorker.postMessage({ kind: 'step', ticks: Number.MAX_SAFE_INTEGER });   
        this.enableButtons("pause", "run");
        this.buttons['run'].classList.add('is-loading');
        this.state.running = true;

        //this.updateUiWhileRunning();
    }

    actionLoad() {
        myWorker.postMessage({ kind: 'load', 'source': editor.getValue() });
    }

    actionStep() {
        myWorker.postMessage({ kind: 'step', ticks: 1 });
    }

    actionPause() {
        myWorker.postMessage({ kind: 'pause' });
    }

    actionReset() {
        this.enableButtons("load");
    
        latestStatus = null;
        decorations = editor.deltaDecorations(decorations, []);
    
        editor.updateOptions({
            readOnly: false
        });
    }
};

let ui = new Ui();
