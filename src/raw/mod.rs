/*
 * Copyright (c) VisualDevelopment 2021-2022.
 * This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.
 */

#![allow(clippy::return_self_not_must_use, clippy::unnecessary_cast)]

pub const FORMULAE_MAGIC: [u8; 8] = [b'f', b'o', b'r', b'm', b'u', b'l', b'a', b'e'];

pub trait AnyToBytes: Sized {
    /// # Safety
    /// The callee must ensure safe operation
    unsafe fn to_bytes(&self) -> &[u8] {
        core::slice::from_raw_parts(self as *const _ as *const u8, core::mem::size_of::<Self>())
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum NodeType {
    Bool = 0,
    Int32,
    Int64,
    String,
    Array,
    Dictionary,
    End = 0xFF,
}

impl AnyToBytes for NodeType {}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(C, packed)]
pub struct NodeHeader {
    pub node_type: NodeType,
    pub key_len: u16,
}

impl AnyToBytes for NodeHeader {}

impl NodeHeader {
    pub fn new(node_type: NodeType, key_len: u16) -> Self {
        Self { node_type, key_len }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(C, packed)]
pub struct BoolNode {
    pub value: bool,
}

impl AnyToBytes for BoolNode {}

impl BoolNode {
    pub fn new(value: bool) -> Self {
        Self { value }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(C, packed)]
pub struct Int32Node {
    pub value: u32,
}

impl AnyToBytes for Int32Node {}

impl Int32Node {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(C, packed)]
pub struct Int64Node {
    pub value: u64,
}

impl AnyToBytes for Int64Node {}

impl Int64Node {
    pub fn new(value: u64) -> Self {
        Self { value }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(C, packed)]
pub struct StringNode {
    pub len: u64,
}

impl AnyToBytes for StringNode {}

impl StringNode {
    pub fn new(len: u64) -> Self {
        Self { len }
    }
}
