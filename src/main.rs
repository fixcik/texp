use std::{collections::HashMap, fs::File, path::PathBuf};

use anyhow::{anyhow, Context};
use clap::Parser;
use hmac::Mac;
use rand::{
    distributions::{Alphanumeric, Slice},
    thread_rng, Rng,
};
use serde_yaml::{to_value, Mapping, Value};

#[cfg(feature = "jwt")]
use hmac::Hmac;
#[cfg(feature = "jwt")]
use jwt::SignWithKey;
#[cfg(feature = "jwt")]
use sha2::Sha256;

#[cfg(feature = "base64")]
use base64::{engine::general_purpose::STANDARD, Engine as _};

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

    tera.register_function(
        "randomString",
        Box::new(|args: &HashMap<String, tera::Value>| {
            let rng = thread_rng();
            let length = match args.get("length") {
                Some(value) => value.as_number().unwrap().as_u64().unwrap(),
                None => return Err(tera::Error::msg("No length provided")),
            };
            let string: String = match args.get("chars") {
                Some(value) => {
                    let vowels = value.as_str().unwrap().chars().collect::<Vec<_>>();
                    let vowels_dist = Slice::new(&vowels).unwrap();
                    rng.sample_iter(&vowels_dist)
                        .take(length as usize)
                        .collect()
                }
                None => rng
                    .sample_iter(&Alphanumeric)
                    .take(length as usize)
                    .map(char::from)
                    .collect(),
            };
            tera::Result::Ok(tera::to_value(string)?)
        }),
    );

    if cfg!(feature = "base64") {
        tera.register_filter(
            "base64",
            |value: &tera::Value, _: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
                let value = value.as_str().unwrap().as_bytes();
                tera::Result::Ok(tera::to_value(STANDARD.encode(value))?)
            },
        );
    }

    if cfg!(feature = "jwt") {
        tera.register_function(
            "jwtToken",
            Box::new(|args: &HashMap<String, tera::Value>| {
                let claims = {
                    let claims = args
                        .get("claims")
                        .ok_or(tera::Error::msg("No claims provided"))?;

                    let claims = claims
                        .as_object()
                        .ok_or(tera::Error::msg("Claims should be an object"))?;

                    claims
                };

                let secret = args
                    .get("secret")
                    .ok_or(tera::Error::msg("No secret provided"))?
                    .as_str()
                    .ok_or(tera::Error::msg("Secret should be a string"))?;

                let key: Hmac<Sha256> = Hmac::new_from_slice(secret.as_bytes()).unwrap();

                let token = claims
                    .sign_with_key(&key)
                    .map_err(|e| tera::Error::msg(e))?;

                tera::Result::Ok(tera::to_value(token)?)
            }),
        );
    }

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
