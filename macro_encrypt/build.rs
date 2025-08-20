use rand::{Rng, thread_rng};
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    let mut rand = thread_rng();
    let key: u16 = rand.r#gen();
    let path = Path::new("config.rs");
    let mut file = File::create(&path).unwrap();

    writeln!(
        file,
        "const KEY: u16 = {};\n
        ",
        key
    )
    .unwrap();
}
