use crate::protos::sentencepiece_model::ModelProto;
use anyhow::Result;
use protobuf::{self, Message};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

impl ModelProto {
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut f = BufWriter::new(File::create(path)?);
        self.write_to_writer(&mut f)?;
        Ok(())
    }
}
