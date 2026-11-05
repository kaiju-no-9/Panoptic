use lsp_server::Connection;

fn main() {
    if std::env::args().any(|a| a == "--version" || a == "-V") {
        println!("panoptic-lsp {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    // Stdout carries LSP traffic only; all logging goes to stderr.
    eprintln!("panoptic-lsp {} starting", env!("CARGO_PKG_VERSION"));
    let (connection, io_threads) = Connection::stdio();
    let result = panoptic_lsp::server::run(connection);
    if let Err(e) = io_threads.join() {
        eprintln!("panoptic-lsp: io error: {e}");
    }
    match result {
        Ok(()) => eprintln!("panoptic-lsp: shut down cleanly"),
        Err(e) => {
            eprintln!("panoptic-lsp: fatal: {e}");
            std::process::exit(1);
        }
    }
}
