use chrono::Local;
use cowl_cli::{cli, utils::config};
use env_logger::Builder;
use log::LevelFilter;
use std::{
    fs::OpenOptions,
    io::{stdout, Write},
    sync::Mutex,
};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Initialize logger
    init_logger();

    // Initialize config
    config::init().await;

    cli::run().await;
}

fn init_logger() {
    // Open or create the log file in append mode
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("app.log")
        .expect("Failed to open log file");

    let log_file = Mutex::new(log_file);

    Builder::new()
        .filter(None, LevelFilter::Info)
        .format(move |_, record| {
            // Lock the log file for writing
            let mut log_file = log_file.lock().unwrap();

            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let log_message = format!("{} [{}] {}\n", timestamp, record.level(), record.args());

            // Write to the log file
            log_file
                .write_all(log_message.as_bytes())
                .expect("Failed to write log");

            // Also write to the default logger output (stdout by default)
            writeln!(stdout(), "{}", log_message.trim_end())
        })
        .init();
}
