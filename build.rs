use std::path::Path;
extern crate rustsourcebundler;
use rustsourcebundler::Bundler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut bundler: Bundler =
        Bundler::new(Path::new("src/bin/main.rs"), Path::new("src/bin/merged.rs"));
    bundler.crate_name("cgbot");
    bundler.run();
    Ok(())
}
