use clap::Parser;

#[derive(Parser)]
struct CliArgs {
    /// The Rust crate (or workspace) to depubify
    #[clap(short, long)]
    path: std::path::PathBuf,

    /// The list of list of arguments to pass to `cargo check`
    #[clap(short, long)]
    check_args: Vec<String>,
}

fn main() {
    let args = CliArgs::parse();
    for entry in walkdir::WalkDir::new(&args.path) {
        let entry = entry.unwrap();
        if entry.file_name().to_str().unwrap().ends_with(".rs")
            && !entry.path().to_str().unwrap().contains("target")
        {
            println!("{}", entry.path().to_string_lossy());
            for (find, replace) in [
                ("vec!", ""),
                ("pub ", ""),
                ("pub(crate) ", ""),
                ("pub ", "pub(crate) "),
            ] {
                let contents: String =
                    String::from_utf8(std::fs::read(entry.path()).unwrap()).unwrap();
                let mut keep = vec![];
                let mut count = 0;
                let mut count_replaced = 0;
                for mat in regex::Regex::new(&regex::escape(find))
                    .unwrap()
                    .find_iter(&contents)
                {
                    count += 1;
                    let mut replaced = contents.clone();
                    replaced.replace_range(mat.range(), replace);
                    std::fs::write(entry.path(), replaced).unwrap();
                    let mut success = true;
                    for check_args in &args.check_args {
                        if !check_args
                            .split(',')
                            .fold(
                                std::process::Command::new("cargo").arg("check"),
                                |acc, arg| {
                                    if !arg.is_empty() {
                                        acc.arg(arg.trim())
                                    } else {
                                        acc
                                    }
                                },
                            )
                            .arg("--manifest-path")
                            .arg(format!("{}/Cargo.toml", args.path.to_string_lossy()))
                            .output()
                            .unwrap()
                            .status
                            .success()
                        {
                            success = false;
                            break;
                        }
                    }
                    if success {
                        keep.push(mat);
                        count_replaced += 1;
                    }
                }
                let mut cleaned = contents.clone();
                for mat in keep.into_iter().rev() {
                    cleaned.replace_range(mat.range(), replace)
                }
                std::fs::write(entry.path(), cleaned).unwrap();
                println!("    replaced {count_replaced} / {count} for '{find}' -> '{replace}'",);
            }
        }
    }
}
