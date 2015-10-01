extern crate guitar_tabs;
extern crate env_logger;

use std::fs::File;
use std::path::Path;
use std::io::Read;
use guitar_tabs::legacy::GPFile;

fn main() {
    env_logger::init().unwrap();
    let args : Vec<_> = std::env::args().collect();

    let io : Box<Read> = if args.len() > 1 {
        Box::new(File::open(&Path::new(&args[1])).unwrap())
    } else {
        Box::new(std::io::stdin())
    };
    let mut gpf = GPFile::new(io);
    println!("{}", gpf.read_version().unwrap());
}
