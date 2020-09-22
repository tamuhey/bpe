// Automatically generated rust module for 'sentencepiece.proto' file

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt_skip)]


use std::borrow::Cow;
use quick_protobuf::{MessageRead, MessageWrite, BytesReader, Writer, WriterBackend, Result};
use quick_protobuf::sizeofs::*;
use super::*;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct SentencePieceText<'a> {
    pub text: Option<Cow<'a, str>>,
    pub pieces: Vec<sentencepiece::mod_SentencePieceText::SentencePiece<'a>>,
    pub score: Option<f32>,
}

impl<'a> MessageRead<'a> for SentencePieceText<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.text = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(18) => msg.pieces.push(r.read_message::<sentencepiece::mod_SentencePieceText::SentencePiece>(bytes)?),
                Ok(29) => msg.score = Some(r.read_float(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for SentencePieceText<'a> {
    fn get_size(&self) -> usize {
        0
        + self.text.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.pieces.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
        + self.score.as_ref().map_or(0, |_| 1 + 4)
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.text { w.write_with_tag(10, |w| w.write_string(&**s))?; }
        for s in &self.pieces { w.write_with_tag(18, |w| w.write_message(s))?; }
        if let Some(ref s) = self.score { w.write_with_tag(29, |w| w.write_float(*s))?; }
        Ok(())
    }
}

pub mod mod_SentencePieceText {

use std::borrow::Cow;
use super::*;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct SentencePiece<'a> {
    pub piece: Option<Cow<'a, str>>,
    pub id: Option<u32>,
    pub surface: Option<Cow<'a, str>>,
    pub begin: Option<u32>,
    pub end: Option<u32>,
}

impl<'a> MessageRead<'a> for SentencePiece<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.piece = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(16) => msg.id = Some(r.read_uint32(bytes)?),
                Ok(26) => msg.surface = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(32) => msg.begin = Some(r.read_uint32(bytes)?),
                Ok(40) => msg.end = Some(r.read_uint32(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for SentencePiece<'a> {
    fn get_size(&self) -> usize {
        0
        + self.piece.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.id.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.surface.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.begin.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.end.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.piece { w.write_with_tag(10, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.id { w.write_with_tag(16, |w| w.write_uint32(*s))?; }
        if let Some(ref s) = self.surface { w.write_with_tag(26, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.begin { w.write_with_tag(32, |w| w.write_uint32(*s))?; }
        if let Some(ref s) = self.end { w.write_with_tag(40, |w| w.write_uint32(*s))?; }
        Ok(())
    }
}

}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct NBestSentencePieceText<'a> {
    pub nbests: Vec<sentencepiece::SentencePieceText<'a>>,
}

impl<'a> MessageRead<'a> for NBestSentencePieceText<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.nbests.push(r.read_message::<sentencepiece::SentencePieceText>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for NBestSentencePieceText<'a> {
    fn get_size(&self) -> usize {
        0
        + self.nbests.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        for s in &self.nbests { w.write_with_tag(10, |w| w.write_message(s))?; }
        Ok(())
    }
}
