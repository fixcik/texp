use std::{collections::HashMap, fs::File, path::PathBuf};

use anyhow::{anyhow, Context};
use clap::Parser;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
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

    tera.register_function(
        "randomString",
        Box::new(|args: &HashMap<String, tera::Value>| {
            let rng = thread_rng();
            let length = match args.get("length") {
                Some(value) => value.as_number().unwrap().as_u64().unwrap(),
                None => return Err(tera::Error::msg("No length provided")),
            };
            let hex_string: String = rng
                .sample_iter(&Alphanumeric)
                .take(length as usize)
                .map(char::from)
                .collect();
            tera::Result::Ok(tera::to_value(hex_string)?)
        }),
    );

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

    let template_name = cli.path.as_path().display().to_string();
    tera.add_template_file(cli.path, Some(&template_name))
        .context("Failed to read template file")?;

    if let Some(path) = cli.output {
        let file = std::fs::File::create(path).context("Failed to create file")?;
        tera.render_to(&template_name, &context, file)?;
    } else {
        println!("{}", tera.render(&template_name, &context)?)
    }

    Ok(())
}
