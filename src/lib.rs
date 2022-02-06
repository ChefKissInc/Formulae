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

pub mod obj_types;

pub const FORMULAE_MAGIC: &str = "formulae";

#[derive(Debug, PartialEq)]
pub enum Object {
    Root(HashMap<String, Object>),
    Bool(bool),
    Int32(u32),
    Int64(u64),
    String(String),
    Dictionary(HashMap<String, Object>),
    Array(Vec<Object>),
}

fn read_bytes<const N: usize>(input: &[u8]) -> Result<([u8; N], &[u8]), String> {
    if input.len() < N {
        return Err(format!(
            "Length larger than input slice ({} < {})",
            input.len(),
            N
        ));
    }

    let (head, rest) = input.split_at(N);

    Ok((head.try_into().unwrap(), rest))
}

fn read_key(input: &[u8]) -> Result<(String, &[u8]), String> {
    let (len, input) = read_bytes(input)?;
    let len = u16::from_le_bytes(len) as usize;

    if input.len() < len {
        return Err(format!(
            "Key length larger than input slice ({} < {})",
            input.len(),
            len
        ));
    }

    let (head, input) = input.split_at(len);

    Ok((
        core::str::from_utf8(head)
            .map_err(|e| e.to_string())?
            .to_string(),
        input,
    ))
}

fn read_string(input: &[u8]) -> Result<(String, &[u8]), String> {
    let (len, input) = read_bytes(input)?;
    let len = u64::from_le_bytes(len) as usize;

    if input.len() < len {
        return Err(format!(
            "String length larger than input slice ({} < {})",
            input.len(),
            len
        ));
    }

    let (head, input) = input.split_at(len);

    Ok((
        core::str::from_utf8(head)
            .map_err(|e| e.to_string())?
            .to_string(),
        input,
    ))
}

impl Object {
    pub fn to_obj_type(&self) -> u8 {
        match self {
            Self::Bool(_) => obj_types::BOOL,
            Self::Int32(_) => obj_types::INT32,
            Self::Int64(_) => obj_types::INT64,
            Self::String(_) => obj_types::STR,
            Self::Dictionary(_) => obj_types::DICT,
            Self::Array(_) => obj_types::ARRAY,
            _ => unreachable!(),
        }
    }

    pub fn parse(obj_type: u8, mut input: &[u8]) -> Result<(Option<Self>, &[u8]), String> {
        match obj_type {
            obj_types::BOOL => {
                let ([value], input) = read_bytes(input)?;
                if value > 1 {
                    Err(format!("Invalid value for Bool object: {}", value))
                } else {
                    Ok((Some(Self::Bool(value == 1)), input))
                }
            }
            obj_types::INT32 => {
                let (bytes, input) = read_bytes(input)?;
                Ok((Some(Self::Int32(u32::from_le_bytes(bytes))), input))
            }
            obj_types::INT64 => {
                let (bytes, input) = read_bytes(input)?;
                Ok((Some(Self::Int64(u64::from_le_bytes(bytes))), input))
            }
            obj_types::STR => {
                let (s, input) = read_string(input)?;
                Ok((Some(Self::String(s)), input))
            }
            obj_types::DICT => {
                let mut map = HashMap::new();

                loop {
                    let ([obj_type], rest) = read_bytes(input)?;
                    input = rest;
                    let (key, rest) = read_key(input)?;
                    input = rest;
                    let (object, rest) = Self::parse(obj_type, input)?;
                    input = rest;
                    if let Some(object) = object {
                        map.insert(key, object);
                    } else {
                        break Ok((Some(Self::Dictionary(map)), input));
                    }
                }
            }
            obj_types::ARRAY => {
                let mut items = Vec::new();

                loop {
                    let ([obj_type], rest) = read_bytes(input)?;
                    input = rest;
                    let (object, rest) = Self::parse(obj_type, input)?;
                    input = rest;
                    if let Some(object) = object {
                        items.push(object);
                    } else {
                        break Ok((Some(Self::Array(items)), input));
                    }
                }
            }
            obj_types::END => Ok((None, input)),
            _ => Err(format!("Unknown Object type: {}", obj_type)),
        }
    }

    pub fn parse_root(mut input: &[u8]) -> Result<Self, String> {
        if input.len() < 9 {
            Err("Data too small".to_string())
        } else {
            let mut data = HashMap::new();

            let (magic, rest) = read_bytes::<8>(input).unwrap();
            input = rest;

            if core::str::from_utf8(&magic).map_err(|e| e.to_string())? == FORMULAE_MAGIC {
                while !input.is_empty() {
                    let ([obj_type], rest) = read_bytes(input)?;
                    input = rest;
                    if obj_type == obj_types::END {
                        return Ok(Self::Root(data));
                    }
                    let (key, rest) = read_key(input)?;
                    input = rest;
                    let (object, rest) = Self::parse(obj_type, input)?;
                    input = rest;
                    if let Some(object) = object {
                        data.try_insert(key, object)
                            .map_err(|_| "Tried to insert already existing value".to_string())?;
                    } else {
                        return Ok(Self::Root(data));
                    }
                }

                Err("Missing End object".to_string())
            } else {
                Err("Invalid magic".to_string())
            }
        }
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            Object::Root(data) => {
                bytes.extend_from_slice(FORMULAE_MAGIC.as_bytes());

                for (key, object) in data {
                    bytes.push(object.to_obj_type());
                    bytes.extend_from_slice(&(key.len() as u16).to_le_bytes());
                    bytes.extend_from_slice(key.as_bytes());
                    bytes.extend_from_slice(&object.into_bytes())
                }

                bytes.push(obj_types::END);
            }
            Object::Bool(value) => bytes.extend_from_slice(&(*value as u8).to_le_bytes()),
            Object::Int32(value) => bytes.extend_from_slice(&value.to_le_bytes()),
            Object::Int64(value) => bytes.extend_from_slice(&value.to_le_bytes()),
            Object::String(value) => {
                bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
                bytes.extend_from_slice(value.as_bytes())
            }
            Object::Dictionary(data) => {
                for (key, object) in data {
                    bytes.push(object.to_obj_type());
                    bytes.extend_from_slice(&(key.len() as u16).to_le_bytes());
                    bytes.extend_from_slice(key.as_bytes());
                    bytes.extend_from_slice(&object.into_bytes())
                }
                bytes.push(obj_types::END);
            }
            Object::Array(items) => {
                for object in items {
                    bytes.push(object.to_obj_type() as u8);
                    bytes.extend_from_slice(&object.into_bytes())
                }
                bytes.push(obj_types::END);
            }
        }

        bytes
    }
}
