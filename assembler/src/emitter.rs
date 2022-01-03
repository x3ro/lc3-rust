use std::collections::HashMap;
use anyhow::anyhow;
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

pub fn emit_section(origin: u16, content: Vec<AstNode>) -> Result<Vec<u16>, ErrorWithPosition> {
    let mut offset = origin;
    let mut emittables = vec![];
    let mut labels = HashMap::new();

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
                    for label in &pending_labels {
                        if let Some(_) = labels.insert(label.clone(), offset) {
                            return Err(anyhow!("Re-defined label with name '{}'. Labels must only be used once.", label))
                                .position(position.clone());
                        }
                    }
                    pending_labels.clear();
                }

                let emittable = to_emittable(x).position(position.clone())?;
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
    let mut data = vec![origin];
    for (position, e) in emittables {
        let mut bytecode = e.emit(offset, &labels).position(position.clone())?;
        data.append(&mut bytecode);
        offset += e.size() as u16;
    }

    Ok(data)
}