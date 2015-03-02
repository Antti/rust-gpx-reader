#![feature(old_io)]
#![feature(old_path)]
extern crate gpx;

fn main() {
  use std::old_io::fs::File;
  let args : Vec<_> = std::env::args().collect();
  let stream = if args.len() > 1 {
    File::open(&Path::new(&args[1])).read_to_end()
  } else {
    let mut stdin = std::old_io::stdio::stdin();
    stdin.read_to_end()
  };
  let file_data = stream.unwrap();
  let files = match gpx::read(file_data){
    Ok(files) => files,
    Err(error) => panic!(error)
  };
  println!("{:?}", files);
}
