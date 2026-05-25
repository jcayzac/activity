fn main() {
    let proto_dir = "proto";
    println!("cargo:rerun-if-changed={proto_dir}");
    prost_build::compile_protos(
        &["proto/biome_infocus.proto", "proto/biome_wifi.proto"],
        &[proto_dir],
    )
    .expect("prost_build failed");
}
