// src/main.rs
// Binary entry point – sets up terminal and runs the app.

use anily::run;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize panic hook to restore terminal on panic (handled inside run).
    run().await
}
