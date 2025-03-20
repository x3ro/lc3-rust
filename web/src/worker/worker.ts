import * as Msg from "@/shared/messages";
import { UiMessage } from "@/shared/messages";
import * as assembler from "wasm-assembler";
import * as wasm_vm from "wasm-vm";
import { Registers } from "wasm-vm";
import { memory as raw_wasm_memory } from "wasm-vm/lc3vm_bg.wasm";

let vm: wasm_vm.Wat;
let vm_memory: Uint16Array;

function new_vm(): [wasm_vm.Wat, Uint16Array] {
    let vm = wasm_vm.Wat.new();
    let vm_memory = new Uint16Array(
        raw_wasm_memory.buffer,
        vm.memory_ptr(),
        vm.memory_len()
    );
    return [vm, vm_memory];
}

function log(...args: any[]) {
    postMessage({ kind: 'log', args })
}

let source_map: Record<number, number> = {};

let input_buffer: number[] = [];

function compile(asm: string) {
    try {
        let [data, map] = assembler.assemble_js(asm);
        source_map = map;
        let origin = vm.load(data);
        vm.set_pc(origin);

        postMessage({ kind: 'loaded' });
        postStatus();
        
        // for (let index = 0; index < cells.length; index++) {
        //     if(cells[index] != 0) {
        //         console.log(index, cells[index])
        //     }    
        // }
    } catch (e) {
        postMessage({ kind: 'error', msg: e.toString() })
    }
}

onmessage = function(e) {
    const msg: UiMessage = e.data;
    log(msg);

    switch(msg.kind) {
        case 'load':
            [vm, vm_memory] = new_vm();
            compile(msg.source);
            return;
        
        case 'pause':
            pause = true;
            return;

        case 'step':
            pause = false;
            step(msg.ticks);
            return;

        case 'input':
            input_buffer.push(msg.character);
            return;

        default:
            log("Worker received unknown message", e.data);
            return;
        
    }
}

function registers(): Record<string, number> {
    let register_values = vm.registers();

    let registers: Record<string, number> = {};
    for(const reg in Registers) {
        if(isNaN(Number(reg))) {
            continue;
        }
        registers[Registers[reg]] = register_values[reg];
    }

    return registers;
}

function postStatus(speed_hz: number = 1) {
    let register_values = vm.registers();
    
    postMessage({
        kind: 'status',
        registers: registers(),
        source_line: source_map[register_values[Registers.PC]],
        speed_hz
    })
}

// We want to execute ticks in a tight loop, but if we do this we can't receive
// messages from the UI thread, because `onmessage` will never execute if the
// loop doesn't yield (compare https://stackoverflow.com/a/49237238/124257).
// TODO: What value should we use here?
let YIELD_AFTER_TICKS = 1000000;
let DELAY_AFTER_YIELD_MS = 0;
let pause = false;

// Display status and display data register
const OS_DSR = 0xFE04;
const OS_DDR = 0xFE06;

// Keyboard status and keyboard data register
const OS_KBSR = 0xFE00;
const OS_KBDR = 0xFE02;

function step(ticks_remaining: number) {
    if(ticks_remaining < 1 || vm_memory[0xFFFE] == 0) {
        postMessage({kind: 'paused', halted: vm_memory[0xFFFE] == 0 });
        return;
    }

    if(pause) {
        postMessage({kind: 'paused', halted: vm_memory[0xFFFE] == 0 });
        return
    };

    let ticks = 0;
    let startTime = performance.now();
    while(vm_memory[0xFFFE] > 0 && ticks < ticks_remaining && ticks < YIELD_AFTER_TICKS) {
        // Setting bit[15] on the DSR indicates the display is ready
        // We can always set this, since we're running in sync with the VM
        // (that is, before a new VM instruction we're always done printing)
        vm_memory[OS_DSR] = 0b1000_0000_0000_0000;

        if(input_buffer.length > 0) {
            if(vm.accessed(OS_KBSR) && vm_memory[OS_KBSR] == 0) {
                vm_memory[OS_KBSR] = 0b1000_0000_0000_0000;
                vm_memory[OS_KBDR] = input_buffer.shift();
            }
        }

        if(vm.accessed(OS_KBDR)) {
            vm_memory[OS_KBSR] = 0;
            vm_memory[OS_KBDR] = 0;
        }

        vm.tick();
        ticks += 1;

        let character = vm_memory[OS_DDR] & 0xFF;
        if(character > 0) {
            postMessage({ kind: 'output', character })
            vm_memory[OS_DDR] = 0;
        }
    }

    let elapsedTimeMs = (performance.now() - startTime);
    let hz = (ticks * 1000) / (elapsedTimeMs || 1);
    postStatus(hz);

    setTimeout(() => step(ticks_remaining - ticks), DELAY_AFTER_YIELD_MS);
}
