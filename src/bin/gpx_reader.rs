extern crate guitar_tabs;
extern crate env_logger;

use std::fs::File;
use std::io::Read;
use std::path::Path;
use guitar_tabs::gpx;

fn main() {
    env_logger::init().unwrap();

    let args : Vec<_> = std::env::args().collect();
    let mut file_data = vec!();
    if args.len() > 1 {
        File::open(&Path::new(&args[1])).unwrap().read_to_end(&mut file_data).unwrap();
    } else {
        let mut stdin = std::io::stdin();
        stdin.read_to_end(&mut file_data).unwrap();
    };
    let files = match gpx::read(&file_data){
        Ok(files) => files,
        Err(error) => panic!(error)
    };
    println!("{:?}", files);
}
