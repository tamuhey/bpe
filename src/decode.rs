use crate::protos::sentencepiece_model::ModelProto;
use anyhow::Result;
use clap::Clap;
use log;
use std::fs::File;

#[derive(Clap)]
pub struct DecodeOpts {
    #[clap(short, long)]
    model_path: String,
}

pub fn decode(opts: DecodeOpts) -> Result<()> {
    let mut model = File::open(&opts.model_path)?;
    let model: ModelProto = protobuf::parse_from_reader(&mut model)?;
    log::info!("Loaded model");
    println!("{:?}", model); // DEBUG
    for p in model.get_pieces() {
        eprintln!("{:?}", p); // DEBUG
    }
    Ok(())
}
