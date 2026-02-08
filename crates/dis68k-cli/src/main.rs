use clap::Parser;
use std::process;

/// Amiga 68k hunk executable disassembler
#[derive(Parser)]
#[command(name = "dis68k", version, about)]
struct Cli {
    /// Input Amiga hunk executable file
    input: String,

    /// Write output to file instead of stdout
    #[arg(short, long)]
    output: Option<String>,

    /// CPU variant for instruction decoding
    #[arg(short, long, default_value = "68000")]
    cpu: String,

    /// Show hunk structure info only (no disassembly)
    #[arg(long)]
    hunk_info: bool,

    /// Disable Amiga OS symbol resolution
    #[arg(long)]
    no_symbols: bool,

    /// Hide hex byte dump column
    #[arg(long)]
    no_hex: bool,

    /// Hide line numbers
    #[arg(long)]
    no_line_numbers: bool,

    /// Use uppercase mnemonics (MOVE instead of move)
    #[arg(long)]
    uppercase: bool,

    /// Show additional debug information
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();

    let data = match std::fs::read(&cli.input) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error reading '{}': {}", cli.input, e);
            process::exit(1);
        }
    };

    let hunk_file = match dis68k::parse_hunk_file(&data) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Error parsing '{}': {}", cli.input, e);
            process::exit(1);
        }
    };

    if cli.hunk_info {
        print_hunk_info(&hunk_file, &cli);
        return;
    }

    let cpu = dis68k::CpuVariant::from_str(&cli.cpu).unwrap_or_else(|| {
        eprintln!(
            "Unknown CPU variant '{}'. Use: 68000, 68010, 68020, 68030, 68040, 68060",
            cli.cpu
        );
        process::exit(1);
    });

    let options = dis68k::ListingOptions {
        show_hex: !cli.no_hex,
        show_addresses: true,
        show_line_numbers: !cli.no_line_numbers,
        uppercase: cli.uppercase,
        cpu,
    };

    let listing = dis68k::generate_listing(&hunk_file, &options);

    // Write output
    let output_text: String = listing.iter().map(|l| format!("{}\n", l.text)).collect();

    if let Some(path) = &cli.output {
        if let Err(e) = std::fs::write(path, &output_text) {
            eprintln!("Error writing '{}': {}", path, e);
            process::exit(1);
        }
    } else {
        print!("{output_text}");
    }
}

fn print_hunk_info(hunk_file: &dis68k::HunkFile, cli: &Cli) {
    println!("Amiga Hunk Executable: {}", cli.input);
    println!(
        "Hunks: {} (first: {}, last: {})",
        hunk_file.hunks.len(),
        hunk_file.first_hunk,
        hunk_file.last_hunk
    );
    println!();

    for hunk in &hunk_file.hunks {
        println!(
            "  Hunk {:2}: {:<16} mem={:<6} alloc={:>6} bytes  data={:>6} bytes",
            hunk.index,
            hunk.hunk_type,
            hunk.memory_type.to_string(),
            hunk.alloc_size,
            hunk.data.len(),
        );

        if let Some(name) = &hunk.name {
            println!("           name: \"{}\"", name);
        }

        if !hunk.relocations.is_empty() {
            let total_relocs: usize = hunk.relocations.iter().map(|r| r.offsets.len()).sum();
            let targets: Vec<String> = hunk
                .relocations
                .iter()
                .map(|r| format!("hunk_{}", r.target_hunk))
                .collect();
            println!(
                "           relocations: {} entries -> [{}]",
                total_relocs,
                targets.join(", ")
            );
        }

        if !hunk.symbols.is_empty() {
            println!("           symbols: {}", hunk.symbols.len());
            if cli.verbose {
                for sym in &hunk.symbols {
                    println!("             0x{:08X}  {}", sym.value, sym.name);
                }
            }
        }

        if hunk.debug_data.is_some() {
            println!("           debug data: present");
        }
    }
}
