use parallel_disk_usage::usage_md::render_usage_md;

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), parallel_disk_usage::usage_md::RenderUsageMdError> {
    let content = render_usage_md()?;
    println!("{}", content.trim_end());
    Ok(())
}
