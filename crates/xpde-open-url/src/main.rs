fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(url) = args.get(1) {
        println!("xpde-open-url: {url}");
    }
}
