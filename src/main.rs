use std::{env, error::Error, sync::Arc};

use log::{error, info, LevelFilter};
use roa::{tcp::Listener, App};
use state::State;
use tokio::{net::UdpSocket, sync::Mutex};
use util::Logger;

mod gdm_routes;
mod gdm_server;
mod state;
mod util;

static LOGGER: Logger = Logger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    log::set_logger(&LOGGER)
        .map(|()| {
            log::set_max_level(if cfg!(debug_assertions) {
                LevelFilter::Trace
            } else {
                LevelFilter::Info
            })
        })
        .unwrap();

    let bind_addr = env::var("BIND_ADDRESS").unwrap_or("0.0.0.0".to_string());
    let gdm_port = env::var("GDM_PORT").unwrap_or("53790".to_string());
    let http_port = env::var("HTTP_PORT").unwrap_or("53789".to_string());

    let gdm_addr = format!("{bind_addr}:{gdm_port}");
    let socket = Arc::new(UdpSocket::bind(&gdm_addr).await?);

    let state = Arc::new(Mutex::new(State::new(socket.clone())));
    let state_cloned = state.clone();

    let handle = tokio::spawn(async move {
        if let Err(e) = gdm_server::gdm_server(state_cloned, &gdm_addr).await {
            error!("Error in the server: {}", e);
        }
    });

    let boxed = Box::new(handle);
    Box::leak(boxed);

    let gdm_router = gdm_routes::build_router();
    let app = App::state(state).end(gdm_router.routes("/gdm")?);

    app.listen(format!("{bind_addr}:{http_port}"), |addr| {
        info!("HTTP server listening on: {addr}");
    })?
    .await?;

    Ok(())
}
