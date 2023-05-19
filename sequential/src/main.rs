use clap::Parser;
use common::App;
use itertools::Itertools;
use std::{
    fs::{read_to_string, File},
    io::Write,
    path::Path,
};

#[derive(Parser, Debug)]
struct Cli {
    #[arg(short, long)]
    app_name: String,
    input_files: Vec<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let app = App::load(&cli.app_name)?;

    let mut intermediate = cli
        .input_files
        .iter()
        .flat_map(|file| {
            let content = read_to_string(file).unwrap();
            app.map(file, &content)
        })
        .collect_vec();

    intermediate.sort();

    let mut output_file = File::create(Path::new(&format!("mr-{}-seq", &cli.app_name)))?;
    for (k, kvs) in intermediate
        .into_iter()
        .group_by(|kv| kv.key.clone())
        .into_iter()
    {
        let output = app.reduce(&k, kvs.map(|kv| kv.value).collect_vec());
        writeln!(output_file, "{} {}", k, output)?;
    }

    Ok(())
}
