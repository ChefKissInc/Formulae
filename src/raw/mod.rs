/*
 * Copyright (c) VisualDevelopment 2021-2022.
 * This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.
 */

#![allow(clippy::return_self_not_must_use, clippy::unnecessary_cast)]

use modular_bitfield::prelude::*;

pub const FORMULAE_MAGIC: [u8; 8] = [b'f', b'o', b'r', b'm', b'u', b'l', b'a', b'e'];

#[derive(Debug, BitfieldSpecifier, PartialEq)]
#[repr(u8)]
#[bits = 4]
pub enum NodeType {
    Bool = 0,
    Int32 = 1,
    Int64 = 2,
    String = 3,
    Dictionary = 4,
    End = 0b1111,
}

#[bitfield(bits = 24)]
#[derive(Debug, PartialEq)]
#[repr(C)]
pub struct NodeHeader {
    pub node_type: NodeType,
    #[skip]
    __: B4,
    pub key_len: u16,
}

#[bitfield(bits = 8)]
#[derive(Debug, PartialEq)]
#[repr(C)]
pub struct BoolNode {
    pub value: bool,
    #[skip]
    __: B7,
}

#[bitfield(bits = 32)]
#[derive(Debug, PartialEq)]
#[repr(C)]
pub struct Int32Node {
    pub value: u32,
}

#[bitfield(bits = 64)]
#[derive(Debug, PartialEq)]
#[repr(C)]
pub struct Int64Node {
    pub value: u64,
}

#[bitfield(bits = 64)]
#[derive(Debug, PartialEq)]
#[repr(C)]
pub struct StringNode {
    pub length: u64,
}
