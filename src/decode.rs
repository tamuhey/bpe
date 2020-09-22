use crate::protos::sentencepiece::*;
use crate::protos::sentencepiece_model::ModelProto;
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
    let model: ModelProto = protobuf::parse_from_reader(&mut model)?;
    log::info!("Loaded model");
    eprintln!("{:?}", model); // DEBUG
    Ok(())
}
