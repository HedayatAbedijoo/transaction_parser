use transaction_parser::app;
fn main() {
    if let Err(e) = app::run(std::env::args()) {
        eprintln!("{e}");
    }
}
