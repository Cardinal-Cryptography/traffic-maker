use std::{
    io::Result,
    sync::{Arc, Mutex},
    time::Duration,
};

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use clap::Parser;

use chain_support::create_connection;
use scenario_transfer::SimpleTransferScenario;
use traffic::{run_schedule, EventListener};

use crate::{config::Config, data_export::DataExporter, stats::Stats};

mod config;
mod data_export;
mod stats;

async fn run_backend<EL: 'static + EventListener>(address: &str, event_listener: EL) {
    let connection = create_connection(address);

    // TODO: read from some config scenarios to launch together with parameters
    let scenarios = vec![SimpleTransferScenario::new(
        &connection,
        Duration::from_secs(5),
    )];

    run_schedule(scenarios, event_listener).await;
}

async fn serve_details<DE: DataExporter>(data: web::Data<Arc<Mutex<DE>>>) -> impl Responder {
    HttpResponse::Ok().body(data.export_details())
}

async fn serve_logs<DE: DataExporter>(data: web::Data<Arc<Mutex<DE>>>) -> impl Responder {
    HttpResponse::Ok().body(data.export_logs())
}

#[actix_web::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=trace");
    env_logger::init();
    let config: Config = Config::parse();

    let stats = Arc::new(Mutex::new(Stats::new()));
    let stats_for_backend = stats.clone();

    tokio::spawn(async move {
        run_backend(&config.node, stats_for_backend).await;
    });

    HttpServer::new(move || {
        App::new().app_data(web::Data::new(stats.clone())).service(
            web::scope("")
                .route("details", web::get().to(serve_details::<Stats>))
                .route("logs", web::get().to(serve_logs::<Stats>)),
        )
    })
    .bind(config.expose_host)?
    .run()
    .await
}
