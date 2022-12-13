use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!("cargo:rustc-link-search={}", out_dir.display());

    if env::var("CARGO_FEATURE_USB_LOGGING").is_ok() {
        fs::copy("./bin/libt4usb.a", out_dir.join("libt4usb.a")).unwrap();
    }

    let mut script;
    let link_x;

    if env::var("CARGO_FEATURE_T41").is_ok() {
        link_x = include_bytes!("t41link.x");
        fs::copy("./bin/libt41eeprom.a", out_dir.join("libt4eeprom.a")).unwrap();
        script = File::create(out_dir.join("t41link.x")).unwrap();
    } else {
        link_x = include_bytes!("t40link.x");
        fs::copy("./bin/libt40eeprom.a", out_dir.join("libt4eeprom.a")).unwrap();
        script = File::create(out_dir.join("t40link.x")).unwrap();
    };

    script.write_all(link_x).unwrap();
}
