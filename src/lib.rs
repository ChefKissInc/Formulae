/*
 * Copyright (c) VisualDevelopment 2021-2022.
 * This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.
 */

#![no_std]
#![deny(warnings, unused_extern_crates, clippy::cargo, rust_2021_compatibility)]

extern crate alloc;

use alloc::{string::{String, ToString}, vec::Vec, format};

use hashbrown::HashMap;
use raw::*;

pub mod raw;

#[derive(Debug, PartialEq)]
pub enum Node {
    Bool(bool),
    Int32(u32),
    Int64(u64),
    String(String),
    Array(Vec<Node>),
    Dictionary(HashMap<String, Node>),
}

impl Node {
    pub fn to_node_type(&self) -> NodeType {
        match self {
            Self::Bool(_) => NodeType::Bool,
            Self::Int32(_) => NodeType::Int32,
            Self::Int64(_) => NodeType::Int64,
            Self::String(_) => NodeType::String,
            Self::Array(_) => NodeType::Array,
            Self::Dictionary(_) => NodeType::Dictionary,
        }
    }

    pub fn from_bytes(
        node_type: &NodeType,
        next: &mut *const u8,
        data_end: *const u8,
    ) -> Result<Option<Self>, String> {
        match node_type {
            NodeType::Bool => unsafe {
                let node = &*(*next as *const BoolNode);
                *next = next.add(1);

                if *next >= data_end {
                    Err("Data unexpectedly ended before parsing Bool node".to_string())
                } else {
                    Ok(Some(Self::Bool(node.value)))
                }
            },
            NodeType::Int32 => unsafe {
                let node = &*(*next as *const Int32Node);
                *next = next.add(4);

                if *next >= data_end {
                    Err("Data unexpectedly ended before parsing Int32 node".to_string())
                } else {
                    Ok(Some(Self::Int32(node.value)))
                }
            },
            NodeType::Int64 => unsafe {
                let node = &*(*next as *const Int64Node);
                *next = next.add(8);

                if *next >= data_end {
                    Err("Data unexpectedly ended before parsing Int64 node".to_string())
                } else {
                    Ok(Some(Self::Int64(node.value)))
                }
            },
            NodeType::String => unsafe {
                let node = &*(*next as *const StringNode);
                *next = next.add(8);

                if *next > data_end {
                    Err("Data unexpectedly ended before parsing String node".to_string())
                } else if next.add(node.len as usize) >= data_end {
                    Err("Data unexpectedly ended before parsing String node contents".to_string())
                } else {
                    let value = String::from_utf8_unchecked(
                        core::slice::from_raw_parts(*next, node.len as usize).to_vec(),
                    );
                    *next = next.add(node.len as usize);

                    Ok(Some(Self::String(value)))
                }
            },
            NodeType::Array => unsafe {
                if *next >= data_end {
                    Err("Data unexpectedly ended before parsing Array node".to_string())
                } else {
                    let mut nodes = Vec::new();
                    loop {
                        let node_type = &*(*next as *const NodeType);
                        *next = next.add(1);

                        if let Some(node) = Self::from_bytes(node_type, next, data_end)? {
                            nodes.push(node);
                        } else {
                            break Ok(Some(Self::Array(nodes)));
                        }
                    }
                }
            },
            NodeType::End => Ok(None),
            _ => Err(format!("Unimplemented Node type: {:#X?}", node_type)),
        }
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            Node::Bool(value) => {
                bytes.extend_from_slice(unsafe { BoolNode::new(*value).to_bytes() })
            }
            Node::Int32(value) => {
                bytes.extend_from_slice(unsafe { Int32Node::new(*value).to_bytes() })
            }
            Node::Int64(value) => {
                bytes.extend_from_slice(unsafe { Int64Node::new(*value).to_bytes() })
            }
            Node::String(value) => {
                bytes.extend_from_slice(unsafe { StringNode::new(value.len() as u64).to_bytes() });
                bytes.extend_from_slice(value.as_bytes())
            }
            Node::Array(nodes) => {
                for node in nodes {
                    bytes.push(node.to_node_type() as u8);
                    bytes.extend_from_slice(&node.into_bytes())
                }
            }
            _ => unimplemented!(),
        }

        bytes
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct Root {
    pub nodes: HashMap<String, Node>,
}

impl Root {
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() < 2 {
            Err("Data too small".to_string())
        } else {
            let data_end = unsafe { data.as_ptr().add(data.len()) };
            let mut next = data.as_ptr();
            let mut nodes = HashMap::new();

            let magic = unsafe { core::slice::from_raw_parts(next, 8) };
            next = unsafe { next.add(8) };

            if magic == FORMULAE_MAGIC {
                loop {
                    if next >= data_end {
                        break Err("Missing end node".to_string());
                    }

                    let header = unsafe { &*(next as *const NodeHeader) };

                    next = unsafe { next.add(3) };

                    let key = unsafe {
                        String::from_utf8_unchecked(
                            core::slice::from_raw_parts(next, header.key_len as usize).to_vec(),
                        )
                    };
                    next = unsafe { next.add(header.key_len as usize) };

                    if let Some(node) = Node::from_bytes(&header.node_type, &mut next, data_end)? {
                        nodes
                            .try_insert(key, node)
                            .map_err(|_| "Tried to insert already existing value".to_string())?;
                    } else {
                        break Ok(Self { nodes });
                    }
                }
            } else {
                Err("Invalid magic".to_string())
            }
        }
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&FORMULAE_MAGIC);

        for (key, node) in &self.nodes {
            bytes.extend_from_slice(unsafe {
                NodeHeader::new(node.to_node_type(), key.len().try_into().unwrap()).to_bytes()
            });
            bytes.extend_from_slice(key.as_bytes());
            bytes.extend(node.into_bytes())
        }

        bytes.extend(unsafe { NodeHeader::new(NodeType::End, 0).to_bytes() });

        bytes
    }
}
