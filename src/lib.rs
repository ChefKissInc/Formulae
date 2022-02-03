/*
 * Copyright (c) VisualDevelopment 2021-2022.
 * This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.
 */

#![no_std]
#![deny(warnings, unused_extern_crates, clippy::cargo, rust_2021_compatibility)]

extern crate alloc;

use alloc::{string::String, vec::Vec};

use hashbrown::HashMap;
use raw::*;

pub mod raw;

#[derive(Debug, PartialEq)]
pub enum Node {
    Bool(bool),
    Int32(u32),
    Int64(u64),
    String(String),
    Dictionary(Vec<Node>),
}

#[derive(Debug, Default, PartialEq)]
pub struct Root {
    pub nodes: HashMap<String, Node>,
}

impl Root {
    pub fn from_bytes(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < 2 {
            Err("Data too small")
        } else {
            let data_end = unsafe { data.as_ptr().add(data.len()) };
            let mut next = data.as_ptr();
            let mut nodes = HashMap::new();

            let magic = unsafe { core::slice::from_raw_parts(next, 8) };
            next = unsafe { next.add(8) };

            if magic == FORMULAE_MAGIC {
                loop {
                    if next >= data_end {
                        break Err("Missing end node");
                    }

                    let header = unsafe { &*(next as *const NodeHeader) };

                    next = unsafe { next.add(3) };

                    let key = unsafe {
                        String::from_utf8_unchecked(
                            core::slice::from_raw_parts(next, header.key_len() as usize).to_vec(),
                        )
                    };
                    next = unsafe { next.add(header.key_len() as usize) };

                    match header.node_type() {
                        NodeType::Bool => unsafe {
                            let node = &*(next as *const BoolNode);
                            next = next.add(1);

                            if next >= data_end {
                                break Err("Data unexpectedly ended before parsing Bool node");
                            }

                            nodes
                                .try_insert(key, Node::Bool(node.value()))
                                .map_err(|_| "Tried to add already existing value")?;
                        },
                        NodeType::Int32 => unsafe {
                            let node = &*(next as *const Int32Node);
                            next = next.add(4);

                            if next >= data_end {
                                break Err("Data unexpectedly ended before parsing Int32 node");
                            }

                            nodes
                                .try_insert(key, Node::Int32(node.value()))
                                .map_err(|_| "Tried to add already existing value")?;
                        },
                        NodeType::Int64 => unsafe {
                            let node = &*(next as *const Int64Node);
                            next = next.add(8);

                            if next >= data_end {
                                break Err("Data unexpectedly ended before parsing Int64 node");
                            }

                            nodes
                                .try_insert(key, Node::Int64(node.value()))
                                .map_err(|_| "Tried to add already existing value")?;
                        },
                        NodeType::String => unsafe {
                            let node = &*(next as *const StringNode);
                            next = next.add(8);

                            if next > data_end {
                                break Err("Data unexpectedly ended before parsing String node");
                            }
                            if next.add(node.length() as usize) >= data_end {
                                break Err(
                                    "Data unexpectedly ended before parsing String node contents"
                                );
                            }

                            let value = String::from_utf8_unchecked(
                                core::slice::from_raw_parts(next, node.length() as usize).to_vec(),
                            );
                            next = next.add(node.length() as usize);

                            nodes
                                .try_insert(key, Node::String(value))
                                .map_err(|_| "Tried to add already existing value")?;
                        },
                        NodeType::End => break Ok(Self { nodes }),
                        _ => unimplemented!(),
                    }
                }
            } else {
                Err("Invalid magic")
            }
        }
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&FORMULAE_MAGIC);

        for (key, node) in &self.nodes {
            match node {
                Node::Bool(value) => {
                    bytes.extend_from_slice(
                        &NodeHeader::new()
                            .with_node_type(NodeType::Bool)
                            .with_key_len(key.len().try_into().unwrap())
                            .into_bytes(),
                    );
                    bytes.extend_from_slice(key.as_bytes());
                    bytes.extend_from_slice(&BoolNode::new().with_value(*value).into_bytes())
                }
                Node::Int32(value) => {
                    bytes.extend_from_slice(
                        &NodeHeader::new()
                            .with_node_type(NodeType::Int32)
                            .with_key_len(key.len().try_into().unwrap())
                            .into_bytes(),
                    );
                    bytes.extend_from_slice(key.as_bytes());
                    bytes.extend_from_slice(&Int32Node::new().with_value(*value).into_bytes())
                }
                Node::Int64(value) => {
                    bytes.extend_from_slice(
                        &NodeHeader::new()
                            .with_node_type(NodeType::Int64)
                            .with_key_len(key.len().try_into().unwrap())
                            .into_bytes(),
                    );
                    bytes.extend_from_slice(key.as_bytes());
                    bytes.extend_from_slice(&Int64Node::new().with_value(*value).into_bytes())
                }
                Node::String(value) => {
                    bytes.extend_from_slice(
                        &NodeHeader::new()
                            .with_node_type(NodeType::String)
                            .with_key_len(key.len().try_into().unwrap())
                            .into_bytes(),
                    );
                    bytes.extend_from_slice(key.as_bytes());
                    bytes.extend_from_slice(
                        &StringNode::new()
                            .with_length(value.len() as u64)
                            .into_bytes(),
                    );
                    bytes.extend_from_slice(value.as_bytes())
                }
                _ => unimplemented!(),
            }
        }
        bytes.extend(NodeHeader::new().with_node_type(NodeType::End).into_bytes());

        bytes
    }
}
