use std::io::{Read, Write};
use std::env;
use std::path::{Path, PathBuf};

fn main() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let mut base_path = None;
    let mut out_path = None;
    while let (Some(a), Some(b)) = (args.next(), args.next()) {
        if a == "-p" {
            let pb = PathBuf::from(&b);
            if pb.exists() && pb.is_dir() {
                base_path = Some(pb);
            } else {
                return Err(String::from("No dir at informed path"));
            }
        } else if a == "-o" {
            let pb = PathBuf::from(&b);
            if pb.exists() && pb.is_dir() {
                out_path = Some(pb);
            }
        } else {
            return Err(String::from("Unrecognized option"));
        }
    }
    let mut latex = Vec::new();
    std::io::stdin().read_to_end(&mut latex).unwrap();
    let latex = String::from_utf8(latex).unwrap();
    if let (Some(base_path), Some(out_path)) = (base_path, out_path) {
        let doc = papers::typesetter::typeset_document(&latex, &base_path, &out_path)?;
        std::io::stdout().write_all(&doc);
        Ok(())
    } else {
        Err(String::from("Missing base or output path"))
    }
}


