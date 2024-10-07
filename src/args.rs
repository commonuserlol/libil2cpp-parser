use clap::Parser;

#[derive(Parser)]
pub struct Arguments {
    #[arg(required = false, short = '1', default_value_t = false, help = "Download all available editors")]
    pub stage_1: bool,
    #[arg(required = false, short = '2', default_value_t = false, help = "Build VERSIONS.md")]
    pub stage_2: bool,
    #[arg(
        required = false,
        short = '3',
        default_value_t = false,
        help = "Build single-header structs and api, and diff them"
    )]
    pub stage_3: bool,
}
