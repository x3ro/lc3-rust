import { Registers } from "wasm-vm";

export type UiMessage
  = { kind: 'load', source: string }
  | { kind: 'step', ticks: number }
  | { kind: 'pause' }
  | { kind: 'input', character: number }
;

export type WorkerStatusMessage = 
{ kind: 'status', registers: Record<Registers, number>, source_line: number, speed_hz: number }

export type WorkerMessage
  = { kind: 'log', args: any[] }
  | { kind: 'loaded' }
  | { kind: 'paused', halted: boolean }
  | { kind: 'error', msg: string }
  | { kind: 'output', character: number }
  | WorkerStatusMessage;
;
