extern crate protoc_rust;

const TARGET_PROTOBUF: &[&str] = &["src/sentencepiece.proto", "src/sentencepiece_model.proto"];
fn main() {
    for path in TARGET_PROTOBUF {
        println!("cargo:rerun-if-changed={}", path);
    }
    println!("Build protobuf");
    protoc_rust::Codegen::new()
        .out_dir("src/protos")
        .inputs(TARGET_PROTOBUF)
        .run()
        .expect("protoc");
}
