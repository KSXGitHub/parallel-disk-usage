use parallel_disk_usage::usage_md::render_usage_md;

fn main() {
    match render_usage_md() {
        Ok(content) => println!("{}", content.trim_end()),
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(1);
        }
    }
}
