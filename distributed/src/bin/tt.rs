use std::{env, fs::read_to_string};

fn main() -> std::io::Result<()> {
    let path = env::current_dir()?;
    println!("dir is {}", path.display());
    let xx = read_to_string("out/mr-7-9-44f9b19d-2ea3-4026-ba38-fa0dd8b73b53")?;
    println!("{xx}");
    Ok(())
}
