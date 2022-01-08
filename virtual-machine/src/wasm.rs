use wasm_bindgen::JsValue;
use crate::{load_words, tick, VmState};

use wasm_bindgen::prelude::*;


#[wasm_bindgen]
pub struct Test {
    data: Vec<u16>,
}

#[wasm_bindgen]
impl Test {
    pub fn as_slice(&self) -> *const u16 {
        self.data.as_ptr()
    }
}




#[wasm_bindgen]
pub struct Wat {
    state: VmState<'static>,
}


#[wasm_bindgen]
impl Wat {
    pub fn new() -> Self {
        Wat { state: VmState::new() }
    }

    pub fn load(&mut self, data: Vec<u16>) -> Result<u16, JsValue> {
        load_words(&data, &mut self.state).map_err(|e| e.to_string().into())

    }

    pub fn memory_ptr(&self) -> *const u16 {
        self.state.memory.raw().as_ptr()
    }

    pub fn memory_len(&self) -> usize {
        self.state.memory.raw().len()
    }

    pub fn registers(&self) -> Vec<u16> {
        Vec::from(self.state.registers.raw())
    }

    pub fn set_pc(&mut self, pc: u16) {
        self.state.set_pc(pc);
    }

    pub fn tick(&mut self) {
        tick(&mut self.state);
    }


}