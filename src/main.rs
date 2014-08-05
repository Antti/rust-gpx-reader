extern crate serialize;

mod gpx;
mod bitbuffer;

fn main(){
  use std::io::fs::File;
  let args = std::os::args();
  let stream = if args.len() > 1 {
    File::open(&Path::new(args[1].as_slice())).read_to_end()
  } else {
    let mut stdin = std::io::stdio::stdin();
    stdin.read_to_end()
  };
  let file_data = stream.unwrap();
  let files = gpx::read(file_data).unwrap();
  println!("{}", files);
}
