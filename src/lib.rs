/*
 * Copyright (c) VisualDevelopment 2021-2022.
 * This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.
 */

#![no_std]
#![deny(warnings, unused_extern_crates, clippy::cargo, rust_2021_compatibility)]

extern crate alloc;

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use hashbrown::HashMap;

pub mod node_types;

pub const FORMULAE_MAGIC: &str = "formulae";

#[derive(Debug, PartialEq)]
pub enum Node {
    Root(HashMap<String, Node>),
    Bool(bool),
    Int32(u32),
    Int64(u64),
    String(String),
    Array(Vec<Node>),
    Dictionary(HashMap<String, Node>),
}

fn read_bytes<const N: usize>(input: &[u8]) -> Option<([u8; N], &[u8])> {
    if input.len() < N {
        return None;
    }

    let (head, rest) = input.split_at(N);

    Some((head.try_into().unwrap(), rest))
}

fn read_key(input: &[u8]) -> Option<(String, &[u8])> {
    let (len, input) = read_bytes(input)?;
    let len = u16::from_le_bytes(len) as usize;

    if input.len() < len {
        return None;
    }

    let (head, input) = input.split_at(len);

    Some((core::str::from_utf8(head).ok()?.to_string(), input))
}

fn read_string(input: &[u8]) -> Option<(String, &[u8])> {
    let (len, input) = read_bytes(input)?;
    let len = u64::from_le_bytes(len) as usize;

    if input.len() < len {
        return None;
    }

    let (head, input) = input.split_at(len);

    Some((core::str::from_utf8(head).ok()?.to_string(), input))
}

impl Node {
    pub fn to_node_type(&self) -> u8 {
        match self {
            Self::Bool(_) => node_types::BOOL,
            Self::Int32(_) => node_types::INT32,
            Self::Int64(_) => node_types::INT64,
            Self::String(_) => node_types::STR,
            Self::Array(_) => node_types::ARRAY,
            Self::Dictionary(_) => node_types::DICT,
            _ => unreachable!(),
        }
    }

    pub fn parse(node_type: u8, mut input: &[u8]) -> Result<Option<(Self, &[u8])>, String> {
        match node_type {
            node_types::BOOL => {
                if let Some(([value], input)) = read_bytes(input) {
                    if value > 1 {
                        Err(format!("Invalid value for Bool node: {}", value))
                    } else {
                        Ok(Some((Self::Bool(value == 1), input)))
                    }
                } else {
                    Err("Data unexpectedly ended while parsing Bool node".to_string())
                }
            }
            node_types::INT32 => {
                if let Some((bytes, input)) = read_bytes(input) {
                    Ok(Some((Self::Int32(u32::from_le_bytes(bytes)), input)))
                } else {
                    Err("Data unexpectedly ended while parsing Int32 node".to_string())
                }
            }
            node_types::INT64 => {
                if let Some((bytes, input)) = read_bytes(input) {
                    Ok(Some((Self::Int64(u64::from_le_bytes(bytes)), input)))
                } else {
                    Err("Data unexpectedly ended while parsing Int64 node".to_string())
                }
            }
            node_types::STR => {
                if let Some((s, input)) = read_string(input) {
                    Ok(Some((Self::String(s), input)))
                } else {
                    Err("Data unexpectedly ended while parsing String node".to_string())
                }
            }
            node_types::ARRAY => {
                let mut nodes = Vec::new();

                loop {
                    if let Some(([node_type], rest)) = read_bytes(input) {
                        input = rest;

                        if let Some((node, rest)) = Self::parse(node_type, input)? {
                            input = rest;
                            nodes.push(node);
                        } else {
                            break Ok(Some((Self::Array(nodes), input)));
                        }
                    } else {
                        break Err("Data unexpectedly ended while parsing Array node".to_string());
                    }
                }
            }
            node_types::DICT => {
                let mut map = HashMap::new();

                loop {
                    if let Some(([node_type], rest)) = read_bytes(input) {
                        input = rest;
                        if let Some((key, rest)) = read_key(input) {
                            input = rest;
                            if let Some((node, rest)) = Node::parse(node_type, input)? {
                                input = rest;
                                map.insert(key, node);
                            } else {
                                break Ok(Some((Self::Dictionary(map), input)));
                            }
                        } else {
                            break Err("Data unexpectedly ended while parsing Dictionary node \
                                       contents"
                                .to_string());
                        }
                    } else {
                        break Err(
                            "Data unexpectedly ended while parsing Dictionary node".to_string()
                        );
                    }
                }
            }
            node_types::END => Ok(None),
            _ => Err(format!("Unknown Node type: {}", node_type)),
        }
    }

    pub fn parse_root(mut input: &[u8]) -> Result<Self, String> {
        if input.len() < 2 {
            Err("Data too small".to_string())
        } else {
            let mut nodes = HashMap::new();

            let (magic, rest) = read_bytes::<8>(input).unwrap();
            input = rest;

            if core::str::from_utf8(&magic).map_err(|e| e.to_string())? == FORMULAE_MAGIC {
                while !input.is_empty() {
                    if let Some(([node_type], rest)) = read_bytes(input) {
                        input = rest;
                        if let Some((key, rest)) = read_key(input) {
                            input = rest;
                            if let Some((node, rest)) = Node::parse(node_type, input)? {
                                input = rest;
                                nodes.try_insert(key, node).map_err(|_| {
                                    "Tried to insert already existing value".to_string()
                                })?;
                            } else {
                                return Ok(Self::Root(nodes));
                            }
                        }
                    } else {
                        return Err("Data unexpectedly ended".to_string());
                    }
                }

                Err("Missing End node".to_string())
            } else {
                Err("Invalid magic".to_string())
            }
        }
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            Node::Root(map) => {
                bytes.extend_from_slice(FORMULAE_MAGIC.as_bytes());

                for (key, node) in map {
                    bytes.push(node.to_node_type());
                    bytes.extend_from_slice(&(key.len() as u16).to_le_bytes());
                    bytes.extend_from_slice(key.as_bytes());
                    bytes.extend_from_slice(&node.into_bytes())
                }

                bytes.push(node_types::END);
                bytes.extend_from_slice(&0u16.to_le_bytes());
            }
            Node::Bool(value) => bytes.extend_from_slice(&(*value as u8).to_le_bytes()),
            Node::Int32(value) => bytes.extend_from_slice(&value.to_le_bytes()),
            Node::Int64(value) => bytes.extend_from_slice(&value.to_le_bytes()),
            Node::String(value) => {
                bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
                bytes.extend_from_slice(value.as_bytes())
            }
            Node::Array(nodes) => {
                for node in nodes {
                    bytes.push(node.to_node_type() as u8);
                    bytes.extend_from_slice(&node.into_bytes())
                }
                bytes.push(node_types::END);
                bytes.extend_from_slice(&0u16.to_le_bytes());
            }
            Node::Dictionary(map) => {
                for (key, node) in map {
                    bytes.push(node.to_node_type());
                    bytes.extend_from_slice(&(key.len() as u16).to_le_bytes());
                    bytes.extend_from_slice(key.as_bytes());
                    bytes.extend_from_slice(&node.into_bytes())
                }
                bytes.push(node_types::END);
                bytes.extend_from_slice(&0u16.to_le_bytes());
            }
        }

        bytes
    }
}
