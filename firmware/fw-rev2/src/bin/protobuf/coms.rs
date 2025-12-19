// Automatically generated rust module for 'coms.proto' file

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt_skip)]


use alloc::vec::Vec;
use alloc::borrow::Cow;
use quick_protobuf::{MessageInfo, MessageRead, MessageWrite, BytesReader, Writer, WriterBackend, Result};
use quick_protobuf::sizeofs::*;
use super::*;

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct QRequest<'a> {
    pub id: i32,
    pub op: i32,
    pub data: Cow<'a, [u8]>,
}

impl<'a> MessageRead<'a> for QRequest<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.id = r.read_int32(bytes)?,
                Ok(16) => msg.op = r.read_int32(bytes)?,
                Ok(26) => msg.data = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for QRequest<'a> {
    fn get_size(&self) -> usize {
        0
        + if self.id == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.id) as u64) }
        + if self.op == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.op) as u64) }
        + if self.data == Cow::Borrowed(b"") { 0 } else { 1 + sizeof_len((&self.data).len()) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.id != 0i32 { w.write_with_tag(8, |w| w.write_int32(*&self.id))?; }
        if self.op != 0i32 { w.write_with_tag(16, |w| w.write_int32(*&self.op))?; }
        if self.data != Cow::Borrowed(b"") { w.write_with_tag(26, |w| w.write_bytes(&**&self.data))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct QResponse<'a> {
    pub id: i32,
    pub error: i32,
    pub data: Cow<'a, [u8]>,
}

impl<'a> MessageRead<'a> for QResponse<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.id = r.read_int32(bytes)?,
                Ok(16) => msg.error = r.read_int32(bytes)?,
                Ok(26) => msg.data = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for QResponse<'a> {
    fn get_size(&self) -> usize {
        0
        + if self.id == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.id) as u64) }
        + if self.error == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.error) as u64) }
        + if self.data == Cow::Borrowed(b"") { 0 } else { 1 + sizeof_len((&self.data).len()) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.id != 0i32 { w.write_with_tag(8, |w| w.write_int32(*&self.id))?; }
        if self.error != 0i32 { w.write_with_tag(16, |w| w.write_int32(*&self.error))?; }
        if self.data != Cow::Borrowed(b"") { w.write_with_tag(26, |w| w.write_bytes(&**&self.data))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct QControl {
    pub sdn: i32,
    pub pwm: i32,
    pub dac0: i32,
    pub dac1: i32,
    pub dac2: i32,
    pub dac3: i32,
}

impl<'a> MessageRead<'a> for QControl {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.sdn = r.read_int32(bytes)?,
                Ok(16) => msg.pwm = r.read_int32(bytes)?,
                Ok(24) => msg.dac0 = r.read_int32(bytes)?,
                Ok(32) => msg.dac1 = r.read_int32(bytes)?,
                Ok(40) => msg.dac2 = r.read_int32(bytes)?,
                Ok(48) => msg.dac3 = r.read_int32(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for QControl {
    fn get_size(&self) -> usize {
        0
        + if self.sdn == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.sdn) as u64) }
        + if self.pwm == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.pwm) as u64) }
        + if self.dac0 == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.dac0) as u64) }
        + if self.dac1 == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.dac1) as u64) }
        + if self.dac2 == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.dac2) as u64) }
        + if self.dac3 == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.dac3) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.sdn != 0i32 { w.write_with_tag(8, |w| w.write_int32(*&self.sdn))?; }
        if self.pwm != 0i32 { w.write_with_tag(16, |w| w.write_int32(*&self.pwm))?; }
        if self.dac0 != 0i32 { w.write_with_tag(24, |w| w.write_int32(*&self.dac0))?; }
        if self.dac1 != 0i32 { w.write_with_tag(32, |w| w.write_int32(*&self.dac1))?; }
        if self.dac2 != 0i32 { w.write_with_tag(40, |w| w.write_int32(*&self.dac2))?; }
        if self.dac3 != 0i32 { w.write_with_tag(48, |w| w.write_int32(*&self.dac3))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct QState {
    pub ch0: i32,
    pub ch1: i32,
    pub ch2: i32,
    pub ch3: i32,
    pub cal: i32,
    pub v: i32,
    pub temp: i32,
    pub sdn: i32,
}

impl<'a> MessageRead<'a> for QState {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.ch0 = r.read_int32(bytes)?,
                Ok(16) => msg.ch1 = r.read_int32(bytes)?,
                Ok(24) => msg.ch2 = r.read_int32(bytes)?,
                Ok(32) => msg.ch3 = r.read_int32(bytes)?,
                Ok(40) => msg.cal = r.read_int32(bytes)?,
                Ok(48) => msg.v = r.read_int32(bytes)?,
                Ok(56) => msg.temp = r.read_int32(bytes)?,
                Ok(64) => msg.sdn = r.read_int32(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for QState {
    fn get_size(&self) -> usize {
        0
        + if self.ch0 == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.ch0) as u64) }
        + if self.ch1 == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.ch1) as u64) }
        + if self.ch2 == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.ch2) as u64) }
        + if self.ch3 == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.ch3) as u64) }
        + if self.cal == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.cal) as u64) }
        + if self.v == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.v) as u64) }
        + if self.temp == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.temp) as u64) }
        + if self.sdn == 0i32 { 0 } else { 1 + sizeof_varint(*(&self.sdn) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.ch0 != 0i32 { w.write_with_tag(8, |w| w.write_int32(*&self.ch0))?; }
        if self.ch1 != 0i32 { w.write_with_tag(16, |w| w.write_int32(*&self.ch1))?; }
        if self.ch2 != 0i32 { w.write_with_tag(24, |w| w.write_int32(*&self.ch2))?; }
        if self.ch3 != 0i32 { w.write_with_tag(32, |w| w.write_int32(*&self.ch3))?; }
        if self.cal != 0i32 { w.write_with_tag(40, |w| w.write_int32(*&self.cal))?; }
        if self.v != 0i32 { w.write_with_tag(48, |w| w.write_int32(*&self.v))?; }
        if self.temp != 0i32 { w.write_with_tag(56, |w| w.write_int32(*&self.temp))?; }
        if self.sdn != 0i32 { w.write_with_tag(64, |w| w.write_int32(*&self.sdn))?; }
        Ok(())
    }
}

