use std::fs;

fn main() {
    println!("cargo:rerun-if-changed=floppy.img");

    let data = fs::read("floppy.img").expect("floppy.img não encontrado");
    println!("cargo:rustc-env=FAT12_BYTES={}", base64::encode(data));
}
