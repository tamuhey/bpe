use crate::protos::sentencepiece::*;
use crate::protos::sentencepiece::*;
use anyhow::Result;
use bytes::Buf;
use clap::Clap;
use log;
use prost::{self, Message};
use std::fs::File;
use std::io::prelude::*;
#[derive(Clap)]
pub struct DecodeOpts {
    #[clap(short, long)]
    model_path: String,
}

use quick_protobuf::{BytesReader, MessageRead};
use std::io::Cursor;
pub fn decode(opts: DecodeOpts) -> Result<()> {
    let mut model = File::open(&opts.model_path)?;
    let model = model.bytes().collect::<Result<Vec<_>, _>>()?;
    let mut reader = BytesReader::from_bytes(&model);
    let model = SentencePieceText::from_reader(&mut reader, &model);
    // let model: SentencePieceText = protobuf::parse_from_reader(&mut model)?;
    log::info!("Loaded model");
    eprintln!("{:?}", model); // DEBUG
    Ok(())
}
