//! oslq - Command-line utility to query OSL shader parameters

use clap::Parser as ClapParser;
use oslquery_petite::OslQuery;
use std::io::{self, IsTerminal};
use std::process;
use std::time::Instant;
use yansi::{Paint, Style};

#[derive(ClapParser, Debug)]
#[command(name = "oslq")]
#[command(about = "Query OSL shader parameters", long_about = None)]
struct Args {
    /// OSO files to query
    files: Vec<String>,

    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Search path for shaders (colon-separated list)
    #[arg(short = 'p', long)]
    searchpath: Option<String>,

    /// Query specific parameter by name
    #[arg(long)]
    param: Option<String>,

    /// Output in JSON format (requires json feature)
    #[arg(long)]
    json: bool,

    /// Show timing statistics
    #[arg(long)]
    runstats: bool,

    /// Disable colored output
    #[arg(long)]
    no_color: bool,
}

fn main() {
    let args = Args::parse();

    // Disable colors if requested or if not a terminal
    if args.no_color || !io::stdout().is_terminal() {
        yansi::disable();
    }

    if args.files.is_empty() {
        eprintln!("Error: No input files specified");
        eprintln!("Usage: oslq [OPTIONS] <FILES>...");
        process::exit(1);
    }

    let searchpath = args.searchpath.as_deref().unwrap_or("");

    for filename in &args.files {
        let start_time = if args.runstats {
            Some(Instant::now())
        } else {
            None
        };

        match OslQuery::open_with_searchpath(filename, searchpath) {
            Ok(query) => {
                if args.json {
                    print_json(&query, &args);
                } else {
                    print_query(&query, &args);
                }

                if let Some(start) = start_time {
                    let elapsed = start.elapsed();
                    eprintln!("Parse time: {:.3}ms", elapsed.as_secs_f64() * 1000.0);
                }
            }
            Err(e) => {
                eprintln!("Error reading {}: {}", filename, e);
                process::exit(1);
            }
        }
    }
}

fn print_json(query: &OslQuery, args: &Args) {
    #[cfg(feature = "serde")]
    {
        use serde_json::json;

        let output = if let Some(ref param_name) = args.param {
            if let Some(param) = query.param_by_name(param_name) {
                json!(param)
            } else {
                eprintln!("Parameter '{}' not found", param_name);
                process::exit(1);
            }
        } else {
            json!(query)
        };

        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    }

    #[cfg(not(feature = "serde"))]
    {
        eprintln!("JSON output requires the 'json' feature to be enabled");
        eprintln!("Rebuild with: cargo build --features json");
        process::exit(1);
    }
}

fn print_query(query: &OslQuery, args: &Args) {
    // Set up color styles
    let styles = ColorStyles {
        keyword: Style::new().magenta().bold(),
        type_name: Style::new().cyan(),
        identifier: Style::new().green(),
        value: Style::new().yellow(),
        delimiter: Style::new().white().dim(),
    };

    println!(
        "{} {} \"{}\"",
        query.shader_type().paint(styles.keyword),
        query.shader_name().paint(styles.identifier),
        query.shader_name()
    );

    // Print global metadata
    for meta in query.metadata() {
        print_metadata(meta, "\t");
    }

    // Calculate max widths for alignment
    let mut max_name_width = 0;
    let mut max_type_width = 0;

    for param in query.params() {
        // Pre-filter for calculating widths
        if let Some(ref filter_name) = args.param
            && param.name.as_str() != filter_name
        {
            continue;
        }
        max_name_width = max_name_width.max(param.name.len());
        // For type width, include "output " prefix if needed for verbose mode
        let type_str = param.typed_param().to_string();
        let type_len = if args.verbose && param.is_output() {
            type_str.len() + 7 // "output " is 7 chars
        } else {
            type_str.len()
        };
        max_type_width = max_type_width.max(type_len);
    }

    for param in query.params() {
        // Filter by parameter name if specified
        if let Some(ref filter_name) = args.param
            && param.name.as_str() != filter_name
        {
            continue;
        }

        let typestring = param.typed_param().to_string();

        if args.verbose {
            // Print with alignment in verbose mode too
            print!("    \"{}\"", param.name.as_str().paint(styles.identifier));

            // Add padding after the name
            let name_padding = if max_name_width > param.name.len() {
                " ".repeat(max_name_width - param.name.len() + 1)
            } else {
                " ".to_string()
            };
            print!("{} \"", name_padding);

            // Print type with alignment
            let type_str = if param.is_output() {
                format!(
                    "{} {}",
                    "output".paint(styles.keyword),
                    typestring.paint(styles.type_name)
                )
            } else {
                format!("{}", typestring.paint(styles.type_name))
            };

            // Just print the type without internal padding
            println!("{}\"", type_str);
        } else {
            // Print with column alignment
            // Format: name type value  OR  name output type value

            // Print name with padding first
            print!(
                "{:width$} ",
                param.name.as_str().paint(styles.identifier),
                width = max_name_width
            );

            // Print output keyword if needed
            if param.is_output() {
                print!("{} ", "output".paint(styles.keyword));
            }

            // Print type with padding
            let type_padding = if param.is_output() {
                max_type_width.saturating_sub(typestring.len())
            } else {
                max_type_width.saturating_sub(typestring.len()) + 7 // Add 7 for "output " width
            };

            print!(
                "{}{} ",
                typestring.paint(styles.type_name),
                " ".repeat(type_padding)
            );
        }

        // Print default values based on the typed parameter
        print_default_values(param, args.verbose, &styles);

        if args.verbose {
            for meta in &param.metadata {
                print_metadata(meta, "\t\t");
            }
        }
    }
}

struct ColorStyles {
    keyword: Style,
    type_name: Style,
    identifier: Style,
    value: Style,
    delimiter: Style, // For brackets, quotes, etc.
}

fn print_default_values(param: &oslquery_petite::Parameter, verbose: bool, styles: &ColorStyles) {
    use oslquery_petite::TypedParameter;

    // For output parameters or closures, show <no default>
    if param.is_output() {
        if verbose {
            println!(
                "\t\tDefault value: {}{}{}",
                " <".paint(styles.delimiter),
                "no default".paint(styles.value),
                ">".paint(styles.delimiter)
            );
        } else {
            println!(
                "  {}{}{}",
                "<".paint(styles.delimiter),
                "no default".paint(styles.value),
                ">".paint(styles.delimiter)
            );
        }
        return;
    }

    match param.typed_param() {
        TypedParameter::Int { default } => {
            if let Some(v) = default {
                if verbose {
                    println!("\t\tDefault value: {}", v.to_string().paint(styles.value));
                } else {
                    println!("   {}", v.to_string().paint(styles.value));
                }
            } else {
                print_no_default(verbose, styles);
            }
        }
        TypedParameter::Float { default } => {
            if let Some(v) = default {
                if verbose {
                    println!("\t\tDefault value: {}", v.to_string().paint(styles.value));
                } else {
                    println!("   {}", v.to_string().paint(styles.value));
                }
            } else {
                print_no_default(verbose, styles);
            }
        }
        TypedParameter::String { default } => {
            if let Some(s) = default {
                if verbose {
                    println!(
                        "\t\tDefault value: {}{}{}",
                        " \"".paint(styles.delimiter),
                        escape_string(s).paint(styles.value),
                        "\"".paint(styles.delimiter)
                    );
                } else {
                    println!(
                        "  {}{}{}",
                        "\"".paint(styles.delimiter),
                        escape_string(s).paint(styles.value),
                        "\"".paint(styles.delimiter)
                    );
                }
            } else {
                print_no_default(verbose, styles);
            }
        }
        TypedParameter::Color { default, .. }
        | TypedParameter::Point { default, .. }
        | TypedParameter::Vector { default, .. }
        | TypedParameter::Normal { default, .. } => {
            if let Some([x, y, z]) = default {
                if verbose {
                    println!(
                        "\t\tDefault value: {}{}{}",
                        "[".paint(styles.delimiter),
                        format!("{} {} {}", x, y, z).paint(styles.value),
                        "]".paint(styles.delimiter)
                    );
                } else {
                    println!(
                        "  {}{}{}",
                        "[".paint(styles.delimiter),
                        format!("{} {} {}", x, y, z).paint(styles.value),
                        "]".paint(styles.delimiter)
                    );
                }
            } else {
                print_no_default(verbose, styles);
            }
        }
        TypedParameter::Matrix { default } => {
            if let Some(m) = default {
                let vals: Vec<String> = m.iter().map(|v| v.to_string()).collect();
                if verbose {
                    println!(
                        "\t\tDefault value: {}{}{}",
                        "[".paint(styles.delimiter),
                        vals.join(" ").paint(styles.value),
                        "]".paint(styles.delimiter)
                    );
                } else {
                    println!(
                        "  {}{}{}",
                        "[".paint(styles.delimiter),
                        vals.join(" ").paint(styles.value),
                        "]".paint(styles.delimiter)
                    );
                }
            } else {
                print_no_default(verbose, styles);
            }
        }
        // Arrays
        TypedParameter::IntArray { default, .. } | TypedParameter::IntDynamicArray { default } => {
            if let Some(vals) = default {
                let s: Vec<String> = vals.iter().map(|v| v.to_string()).collect();
                if verbose {
                    println!(
                        "\t\tDefault value: {}{}{}",
                        "[".paint(styles.delimiter),
                        s.join(" ").paint(styles.value),
                        "]".paint(styles.delimiter)
                    );
                } else {
                    println!(
                        "  {}{}{}",
                        "[".paint(styles.delimiter),
                        s.join(" ").paint(styles.value),
                        "]".paint(styles.delimiter)
                    );
                }
            } else {
                print_no_default(verbose, styles);
            }
        }
        TypedParameter::FloatArray { default, .. }
        | TypedParameter::FloatDynamicArray { default } => {
            if let Some(vals) = default {
                let s: Vec<String> = vals.iter().map(|v| v.to_string()).collect();
                if verbose {
                    println!(
                        "\t\tDefault value: {}{}{}",
                        "[".paint(styles.delimiter),
                        s.join(" ").paint(styles.value),
                        "]".paint(styles.delimiter)
                    );
                } else {
                    println!(
                        "  {}{}{}",
                        "[".paint(styles.delimiter),
                        s.join(" ").paint(styles.value),
                        "]".paint(styles.delimiter)
                    );
                }
            } else {
                print_no_default(verbose, styles);
            }
        }
        TypedParameter::StringArray { default, .. }
        | TypedParameter::StringDynamicArray { default } => {
            if let Some(vals) = default {
                let s: Vec<String> = vals
                    .iter()
                    .map(|v| format!("\"{}\"", escape_string(v)))
                    .collect();
                if verbose {
                    println!(
                        "\t\tDefault value: {}{}{}",
                        "[".paint(styles.delimiter),
                        s.join(" ").paint(styles.value),
                        "]".paint(styles.delimiter)
                    );
                } else {
                    println!(
                        "  {}{}{}",
                        "[".paint(styles.delimiter),
                        s.join(" ").paint(styles.value),
                        "]".paint(styles.delimiter)
                    );
                }
            } else {
                print_no_default(verbose, styles);
            }
        }
        TypedParameter::ColorArray { default, .. }
        | TypedParameter::PointArray { default, .. }
        | TypedParameter::VectorArray { default, .. }
        | TypedParameter::NormalArray { default, .. }
        | TypedParameter::ColorDynamicArray { default, .. }
        | TypedParameter::PointDynamicArray { default, .. }
        | TypedParameter::VectorDynamicArray { default, .. }
        | TypedParameter::NormalDynamicArray { default, .. } => {
            if let Some(vals) = default {
                let s: Vec<String> = vals
                    .iter()
                    .map(|[x, y, z]| format!("[{} {} {}]", x, y, z))
                    .collect();
                if verbose {
                    println!(
                        "\t\tDefault value: {}{}{}",
                        "[".paint(styles.delimiter),
                        s.join(" ").paint(styles.value),
                        "]".paint(styles.delimiter)
                    );
                } else {
                    println!(
                        " {}{}{}",
                        "[".paint(styles.delimiter),
                        s.join(" ").paint(styles.value),
                        "]".paint(styles.delimiter)
                    );
                }
            } else {
                print_no_default(verbose, styles);
            }
        }
        TypedParameter::MatrixArray { default, .. }
        | TypedParameter::MatrixDynamicArray { default } => {
            if let Some(vals) = default {
                let s: Vec<String> = vals
                    .iter()
                    .map(|m| {
                        let v: Vec<String> = m.iter().map(|f| f.to_string()).collect();
                        format!("[{}]", v.join(" "))
                    })
                    .collect();
                if verbose {
                    println!(
                        "\t\tDefault value: {}{}{}",
                        "[".paint(styles.delimiter),
                        s.join(" ").paint(styles.value),
                        "]".paint(styles.delimiter)
                    );
                } else {
                    println!(
                        " {}{}{}",
                        "[".paint(styles.delimiter),
                        s.join(" ").paint(styles.value),
                        "]".paint(styles.delimiter)
                    );
                }
            } else {
                print_no_default(verbose, styles);
            }
        }
        TypedParameter::Closure { .. } => {
            // Closures never have defaults
            print_no_default(verbose, styles);
        }
    }
}

fn print_no_default(verbose: bool, styles: &ColorStyles) {
    if verbose {
        println!(
            "\t\tDefault value: {}{}{}",
            " <".paint(styles.delimiter),
            "no default".paint(styles.value),
            ">".paint(styles.delimiter)
        );
    } else {
        println!(
            "  {}{}{}",
            "<".paint(styles.delimiter),
            "no default".paint(styles.value),
            ">".paint(styles.delimiter)
        );
    }
}

fn print_metadata(meta: &oslquery_petite::Metadata, indent: &str) {
    use oslquery_petite::MetadataValue;

    print!("{}metadata: ", indent);
    match &meta.value {
        MetadataValue::Int(v) => print!("int {} = {}", meta.name, v),
        MetadataValue::Float(v) => print!("float {} = {}", meta.name, v),
        MetadataValue::String(v) => print!("string {} = \"{}\"", meta.name, escape_string(v)),
        MetadataValue::IntArray(v) => {
            print!("int[] {} =", meta.name);
            for val in v {
                print!(" {}", val);
            }
        }
        MetadataValue::FloatArray(v) => {
            print!("float[] {} =", meta.name);
            for val in v {
                print!(" {}", val);
            }
        }
        MetadataValue::StringArray(v) => {
            print!("string[] {} =", meta.name);
            for val in v {
                print!(" \"{}\"", escape_string(val));
            }
        }
    }
    println!();
}

fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
