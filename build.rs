fn main() {
    println!("cargo:rerun-if-changed=assets/icon.rc");
    let _ = embed_resource::compile("assets/icon.rc", embed_resource::NONE);
}
