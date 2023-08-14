#[macro_use]
extern crate rocket;
use std::env;

use log::LevelFilter;
use util::Logger;

mod gdm_routes;
mod gdm_server;
mod util;

static LOGGER: Logger = Logger;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    log::set_logger(&LOGGER)
        .map(|()| {
            log::set_max_level(if cfg!(debug_assertions) {
                LevelFilter::Trace
            } else {
                LevelFilter::Info
            })
        })
        .unwrap();

    let handle = tokio::spawn(async {
        let gdm_port = env::var("GDM_PORT").unwrap_or("53790".to_string());
        if let Err(e) = gdm_server::gdm_server(&gdm_port).await {
            error!("Error in the server: {}", e);
        }
    });

    let boxed = Box::new(handle);
    Box::leak(boxed);

    let _rocket = rocket::build()
        .mount("/gdm/", gdm_routes::build_routes())
        .launch()
        .await?;

    Ok(())
}
