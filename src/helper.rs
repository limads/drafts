use std::io::{Read, Write};

fn main() -> Result<(), String> {
    let mut latex = Vec::new();
    std::io::stdin().read_to_end(&mut latex).unwrap();
    let latex = String::from_utf8(latex).unwrap();
    let doc = papers::typesetter::typeset_document(&latex)?;
    std::io::stdout().write_all(&doc);
    Ok(())
}


