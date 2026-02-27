#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

fn main() {
  tracing_subscriber::registry()
      .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "lantern_riddle=debug,backend_rust=debug,tower_http=debug,axum=debug".into()))
      .with(tracing_subscriber::fmt::layer())
      .init();

  lantern_riddle::run();
}
