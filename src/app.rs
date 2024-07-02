use std::{io, sync::Arc};

pub mod cli;
pub mod config;

use chrono::TimeDelta;
use clap::Parser;
use tokio::signal::ctrl_c;
use tracing::info;
use uuid::uuid;

use crate::{
    config_store::ConfigStore,
    logging::{self, LoggingConfig},
    scheduler::run_scheduler,
    types::check_config::{CheckConfig, CheckInterval},
};

pub fn execute() -> io::Result<()> {
    let app = cli::CliApp::parse();
    let config = config::Config::extract(&app).expect("Configuration invalid");

    logging::init(LoggingConfig::from_config(&config));

    info!(config = ?config);

    match app.command {
        cli::Commands::Run => tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let config_store = Arc::new(ConfigStore::new_rw());

                // XXX: Example config while we build out the consumer that loads configs
                config_store
                    .write()
                    .expect("Lock poisoned")
                    .add_config(Arc::new(CheckConfig {
                        partition: 0,
                        url: "https://downtime-simulator-test1.vercel.app".to_string(),
                        subscription_id: uuid!("663399a09e6340a79c3c7a3f26878904"),
                        interval: CheckInterval::FiveMinutes,
                        timeout: TimeDelta::seconds(5),
                    }));

                run_scheduler(&config, config_store.clone())
                    .await
                    .expect("Failed to run scheduler");
                ctrl_c().await
            }),
    }
}
