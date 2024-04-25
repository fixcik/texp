use std::{fs::File, path::PathBuf};

use anyhow::{anyhow, Context};
use clap::Parser;
use serde_yaml::Mapping;

#[derive(Parser, Debug)]
#[command(name = "texp")]
#[command(bin_name = "texp")]
#[command(version, about)]
struct TextCli {
    #[arg(name = "path")]
    path: PathBuf,

    #[arg(short, long)]
    values: Option<PathBuf>,

    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let cli = TextCli::parse();
    let mut tera = tera::Tera::default();
    let mut context = tera::Context::new();

    let values: Mapping = if let Some(path) = cli.values {
        let reader = File::open(path).context("Failed to read file with values")?;
        serde_yaml::from_reader(reader)?
    } else {
        Default::default()
    };

    for (key, value) in &values {
        context.insert(
            key.as_str()
                .ok_or(anyhow!("Key of values should be a string"))?,
            value,
        );
    }

    tera.add_template_file(cli.path, Some("template"))
        .context("Failed to read template file")?;

    if let Some(path) = cli.output {
        let file = std::fs::File::create(path).context("Failed to create file")?;
        tera.render_to("template", &context, file)?;
    } else {
        println!("{}", tera.render("template", &context)?)
    }

    Ok(())
}
