use anyhow::Result;
use cup_notifier::Config;
use schemars::generate::SchemaSettings;
use std::fs;

fn main() -> Result<()> {
    let output = "schema.json";
    let generator = SchemaSettings::draft07().into_generator();
    let schema = generator.into_root_schema_for::<Config>();

    fs::write(output, serde_json::to_string_pretty(&schema).unwrap())?;

    let absolute = fs::canonicalize(output)?;
    println!("Schema saved to {}", absolute.display());

    Ok(())
}
