use clap::Parser;

/// Generate configuration for an AECO patch server
#[derive(Parser, Debug)]
struct Args {
    /// Path to ECO folder
    eco_dir: String,

    /// Path in which to generate configuration files
    output_dir: String,

    /// If the server should be in maintenance mode
    #[clap(short, long)]
    maintenance_mode: bool,
}

fn main() {
    let args = Args::parse();

    if let Err(why) =
        aeco_patch_config::generate_config(args.eco_dir, args.output_dir, args.maintenance_mode)
    {
        eprintln!("{why:?}");
    }
}
