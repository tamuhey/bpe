extern crate protoc_rust;

use protoc_rust::Customize;

fn main() {
    protoc_rust::Codegen::new()
        .out_dir("src/protos")
        .inputs(&["src/sentencepiece.proto"])
        .run()
        .expect("protoc");
}
