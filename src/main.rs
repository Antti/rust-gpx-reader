mod gpx;
mod bitbuffer;

fn main(){
  use std::io::fs::File;
  let args = std::os::args();
  let stream = if args.len() > 1{
    File::open(&Path::new(args[1].as_slice())).read_to_end()
  } else {
    let mut stdin = std::io::stdio::stdin();
    stdin.read_to_end()
  };
  let file_data = stream.unwrap();
  let content = match gpx::check_file_type(file_data.as_slice()) {
    gpx::BCFZ => { let data = Vec::from_slice(file_data.tailn(4)); gpx::decompress_bcfz(data)},
    gpx::BCFS => fail!("Parsing BCFS is not implemented yet"),
    gpx::Unknown => fail!("Unknown file type (wrong file header)")
  };
  let mut stdout = std::io::stdio::stdout();
  stdout.write(content.as_slice()).unwrap();
}
