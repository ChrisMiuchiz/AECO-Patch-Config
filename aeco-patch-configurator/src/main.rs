use clap::Parser;

/// Generate configuration for an AECO patch server
#[derive(Parser, Debug)]
struct Args {
    /// Path to ECO folder
    eco_dir: String,

    /// Path in which to generate configuration files
    output_dir: String,
}

fn main() {
    let args = Args::parse();

    if let Err(why) = aeco_patch_config::generate_config(args.eco_dir, args.output_dir) {
        eprintln!("{why:?}");
    }
}
