use std::collections::HashMap;
use anyhow::{bail};

use crate::AstNode;
use crate::emittable::Emittable;
use crate::errors::{ErrorWithPosition, PositionContext};

pub fn to_emittable(node: &Box<AstNode>) -> anyhow::Result<Emittable> {
    match node.as_ref() {
        AstNode::Instruction { opcode, operands } => {
            Emittable::from(opcode.clone(), operands.clone())
        },
        x => unreachable!("{:?}", x)
    }
}

pub fn push_to_pending_labels(labels: &mut Vec<String>, node: &AstNode) {
    match node {
        AstNode::Label(name) => labels.push(name.clone()),
        x => unreachable!("{:?}", x),
    };
}

type SourceLocation = usize;
type MemoryLocation = u16;

pub struct Assembly {
    data: Vec<u16>,
    labels: HashMap<String, MemoryLocation>,
    source_map: HashMap<MemoryLocation, SourceLocation>,
}

impl Assembly {
    pub fn new() -> Self {
        Assembly {
            data: vec![],
            labels: Default::default(),
            source_map: Default::default()
        }
    }

    pub fn data(&self) -> &Vec<u16> {
        &self.data
    }

    pub fn source_map(&self) -> &HashMap<MemoryLocation, SourceLocation> {
        &self.source_map
    }

    pub fn record_labels(&mut self, labels: &mut Vec<String>, offset: MemoryLocation) -> anyhow::Result<()> {
        for label in labels.drain(..) {
            if let Some(_) = self.labels.insert(label.clone(), offset) {
                bail!("Re-defined label with name '{}'. Labels must only be used once.", label)
            }
        }

        Ok(())
    }

    pub fn record_offset(&mut self, offset: MemoryLocation, loc: SourceLocation) -> anyhow::Result<()> {
        let insert = self.source_map.insert(offset, loc);
        if let Some(old_loc) = insert {
            bail!("Duplicate key for memory location 0x{:x}. Previous value was {}, new value is {}. This is likely an assembler bug.", offset, old_loc, loc);
        }
        Ok(())
    }

    pub fn record_bytecode(&mut self, mut words: Vec<u16>) {
        self.data.append(&mut words);
    }
}



pub fn emit_section(origin: u16, content: Vec<AstNode>) -> Result<Assembly, ErrorWithPosition> {
    let mut offset = origin;
    let mut emittables = vec![];
    let mut assembly = Assembly::new();

    // These are the labels defined on empty lines which will be associated with
    // the next instruction that occurs.
    let mut pending_labels = vec![];

    // Pass 1: Collect emittables and labels
    for line in &content {
        match line {
            AstNode::Line {
                label,
                instruction: Some(x),
                position,
                ..
            } => {
                if let Some(label) = label {
                    push_to_pending_labels(&mut pending_labels, label);
                }

                if ! pending_labels.is_empty() {
                    assembly.record_labels(&mut pending_labels, offset).position(position.clone())?;
                }

                let emittable = to_emittable(x).position(position.clone())?;

                // Associate not only the beginning of this emittable with the source location, but
                // also all following memory locations in case the emittable's size is > 1
                for x in offset..(offset + emittable.size() as u16) {
                    assembly.record_offset(x, position.pos()).position(position.clone())?;
                }

                offset += emittable.size() as u16;
                emittables.push((position, emittable));
            }

            AstNode::Line {
                label: Some(label),
                instruction: None,
                ..
            } => {
                push_to_pending_labels(&mut pending_labels, label);
            }

            AstNode::Line {
                label: None,
                instruction: None,
                ..
            } => {
                // We can safely ignore this case (no executable information or label)
            }

            x => unreachable!("{:?}", x)
        }
    }

    // Pass 2 - Emit the bytecode now that we have label information
    let mut offset = origin;
    assembly.record_bytecode(vec![origin]);
    for (position, e) in emittables {
        let bytecode = e.emit(offset, &assembly.labels).position(position.clone())?;
        assembly.record_bytecode(bytecode);
        offset += e.size() as u16;
    }

    Ok(assembly)
}
