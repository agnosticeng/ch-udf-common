use std::fs::File;
use std::io::{BufWriter, Read, Result, Write, stdin, stdout};

pub fn open_input(path: Option<&str>) -> Result<Box<dyn Read>> {
    match path {
        None => Ok(Box::new(stdin())),
        Some(s) => {
            let file = File::open(s)?;
            Ok(Box::new(file))
        }
    }
}

pub fn open_output(path: Option<&str>) -> Result<Box<dyn Write>> {
    match path {
        None => Ok(Box::new(BufWriter::new(stdout()))),
        Some(s) => {
            let file = File::create_new(s)?;
            Ok(Box::new(BufWriter::new(file)))
        }
    }
}
