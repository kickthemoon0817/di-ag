mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "di-ag",
    version,
    about = "Diagram-as-code framework with agent feedback loop"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Render a diagram to SVG or PNG
    Render {
        /// Input file (.diag, .json, .yaml). Use - for stdin.
        input: String,
        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
        /// Output format
        #[arg(short, long, default_value = "svg")]
        format: String,
        /// Theme override
        #[arg(long)]
        theme: Option<String>,
        /// Layout algorithm override
        #[arg(long)]
        layout: Option<String>,
        /// Include inspection report in output
        #[arg(long)]
        inspect: bool,
        /// Fail if layout score is below threshold
        #[arg(long)]
        score_threshold: Option<f64>,
        /// Output as JSON (for piping)
        #[arg(long)]
        json: bool,
    },
    /// Validate a diagram (tier 1 constraint checks)
    Validate {
        /// Input file
        input: String,
        /// Treat warnings as errors
        #[arg(long)]
        strict: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Convert between formats
    Convert {
        /// Input file
        input: String,
        /// Target format
        #[arg(long)]
        to: String,
        /// Source format (auto-detected if omitted)
        #[arg(long)]
        from: Option<String>,
        /// Output file
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Format a .diag file
    Fmt {
        /// Input file
        input: String,
        /// Check only (don't modify)
        #[arg(long)]
        check: bool,
    },
    /// Create a starter .diag file
    Init,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Render {
            input,
            output,
            format,
            theme,
            layout: _,
            inspect,
            score_threshold,
            json,
        } => commands::render::run(
            &input,
            output.as_deref(),
            &format,
            theme.as_deref(),
            inspect,
            score_threshold,
            json,
        ),
        Commands::Validate {
            input,
            strict,
            json,
        } => commands::validate::run(&input, strict, json),
        Commands::Convert {
            input,
            to,
            from,
            output,
        } => commands::convert::run(&input, &to, from.as_deref(), output.as_deref()),
        Commands::Fmt { input, check } => commands::fmt::run(&input, check),
        Commands::Init => commands::init::run(),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
