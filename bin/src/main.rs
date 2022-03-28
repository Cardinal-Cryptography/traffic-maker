use std::{
    fs,
    io::Result,
    sync::{Arc, Mutex},
};

use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};

use traffic::run_schedule;

use crate::{config::Config, data_export::DataExporter, stats::Stats};

mod config;
mod data_export;
mod stats;

async fn serve_details<DE: DataExporter>(data: web::Data<DE>) -> impl Responder {
    HttpResponse::Ok().body(data.export_details())
}

async fn serve_logs<DE: DataExporter>(
    data: web::Data<DE>,
    scenario_ident: web::Path<String>,
) -> impl Responder {
    HttpResponse::Ok().body(data.export_logs(scenario_ident.into_inner().into()))
}

fn read_config() -> Config {
    let config_content =
        fs::read_to_string("Timetable.toml").expect("Config file should exist and be readable");
    toml::from_str(&*config_content).expect("Should deserialize")
}

#[actix_web::main]
async fn main() -> Result<()> {
    let config = read_config();

    let stats = Arc::new(Mutex::new(Stats::new()));
    let stats_for_backend = stats.clone();

    let scenarios = config.get_scenarios();
    tokio::spawn(async move {
        run_schedule(scenarios, stats_for_backend).await;
    });

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET"]),
            )
            .app_data(web::Data::new(stats.clone()))
            .service(
                web::scope("")
                    .route("details", web::get().to(serve_details::<Arc<Mutex<Stats>>>))
                    .route(
                        "logs/{scenario_ident}",
                        web::get().to(serve_logs::<Arc<Mutex<Stats>>>),
                    ),
            )
    })
    .bind(config.get_expose_host())?
    .run()
    .await
}
