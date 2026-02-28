use parallel_disk_usage::usage_md::render_usage_md;

fn main() {
    println!("{}", render_usage_md().trim_end());
}
