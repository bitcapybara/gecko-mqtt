fn main() {
    tonic_build::configure().out_dir("src/").compile(&["proto/peer.proto"], &["proto"]).unwrap();
}
