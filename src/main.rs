mod run;
mod worker;

use run::run;

fn main() {
    // Set the RUST_LOG, if it hasn't been explicitly defined
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "web_server=info")
    }

    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    run();
}
