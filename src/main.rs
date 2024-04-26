use std::{collections::HashMap, fs::File, path::PathBuf};

use anyhow::{anyhow, Context};
use clap::Parser;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde_yaml::{to_value, Mapping, Value};

#[derive(Parser, Debug)]
#[command(name = "texp")]
#[command(bin_name = "texp")]
#[command(arg_required_else_help(true))]
#[command(version, about)]
struct TextCli {
    #[arg(name = "path")]
    path: PathBuf,

    /// Path to yaml file with values
    #[arg(short, long)]
    values: Option<PathBuf>,

    /// Set value, e.g "--set foo.a=bar --set foo.b=baz"
    #[arg(long)]
    set: Option<Vec<String>>,

    /// Path to output file
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let cli = TextCli::parse();
    let mut tera = tera::Tera::default();
    let mut context = tera::Context::new();

    println!("{:?}", cli);

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

    let mut values: Mapping = if let Some(path) = cli.values {
        let reader = File::open(path).context("Failed to read file with values")?;
        serde_yaml::from_reader(reader)?
    } else {
        Default::default()
    };

    if let Some(set_values) = &cli.set {
        for entry in set_values.iter() {
            let (key, value) = entry
                .split_once('=')
                .ok_or(anyhow!("Failed to parse --set value"))?;

            let key = key
                .split('.')
                .map(|k| to_value(k))
                .collect::<Result<Vec<_>, _>>()?;
            let value = to_value(value)?;

            let mut current = &mut values;

            for (i, k) in key.iter().enumerate() {
                if i == key.len() - 1 {
                    current.insert(k.clone(), value.clone());
                } else {
                    current = current
                        .entry(k.clone())
                        .or_insert_with(|| Value::Mapping(Mapping::new()))
                        .as_mapping_mut()
                        .unwrap();
                }
            }
        }
    }

    println!("{:?}", values);

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
