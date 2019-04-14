extern crate yacht;
use yacht::{exec::interpret, pe::reader};

extern crate clap;
use clap::{App, Arg};

extern crate ansi_term;
use ansi_term::Colour;

use std::{cell::RefCell, rc::Rc};

const VERSION_STR: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let app = App::new("Yacht")
        .version(VERSION_STR)
        .author("uint256_t")
        .about("An ECMA-335 Implementation written in Rust")
        .arg(Arg::with_name("file").help("Input file name").index(1));
    let app_matches = app.clone().get_matches();

    let filename = match app_matches.value_of("file") {
        Some(filename) => filename,
        None => return,
    };

    #[rustfmt::skip]
    macro_rules! expect { ($expr:expr, $msg:expr) => {{ match $expr {
        Some(some) => some,
        None => { eprintln!("{}: {}", Colour::Red.bold().paint("error"), $msg); return }
    } }}; }

    let mut pe_file_reader = expect!(
        reader::PEFileReader::new(filename),
        format!("File not found '{}'", filename)
    );

    let mut image = expect!(pe_file_reader.create_image(), "Broken file");
    let method = pe_file_reader.read_entry_method(&mut image).unwrap();
    image.reader = Some(Rc::new(RefCell::new(pe_file_reader)));
    let mut interpreter = interpret::Interpreter::new();
    interpreter.interpret(&mut image, method);
}

#[cfg(test)]
mod tests {
    use yacht::pe::reader;

    #[test]
    fn pe_file_reader() {
        for filename in &["./examples/hello.exe"] {
            reader::PEFileReader::new(filename).unwrap().read().unwrap()
        }
    }
}
