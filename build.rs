use clap::CommandFactory;
use clap::ValueEnum;
use clap_complete::{generate_to, Shell};
use clap_complete_fig::Fig;
use std::io::Error;
use std::path::Path;

include!("src/driver/cli.rs");

/// Generate autocompletion files for shells and fig
fn generate_autocompletion() -> Result<(), Error> {
    let out_dir = "target/autocomplete";
    std::fs::create_dir(out_dir).unwrap_or(());

    let mut cmd = Args::command();
    for &shell in Shell::value_variants() {
        generate_to(shell, &mut cmd, "ppl", &out_dir)?;
    }
    generate_to(Fig, &mut cmd, "ppl", &out_dir)?;

    Ok(())
}

/// Precompile the builtin module
fn compile_builtin_module() -> Result<(), Error> {
    let ppl = Path::new("target/debug/ppl");
    if !ppl.exists() {
        return Ok(());
    }

    let status = std::process::Command::new(ppl)
        .args(&["compile", "src/runtime/ppl.ppl"])
        .args(&["--emit", "dynamic-library"])
        .args(&["--output-dir", "target/debug/deps"])
        .arg("--no-core")
        .status()?;

    assert!(status.success());

    Ok(())
}

fn main() -> Result<(), Error> {
    generate_autocompletion()?;

    compile_builtin_module()?;

    Ok(())
}
