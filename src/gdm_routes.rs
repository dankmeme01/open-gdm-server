use std::{io, path::PathBuf};

use rocket::{fs::NamedFile, Route};
use reqwest::Client;

#[get("/update.version")]
pub async fn version() -> io::Result<NamedFile> {
    NamedFile::open("static/update.version").await
}

#[get("/isVip.php?<id>&<cv>&<pass>")]
pub async fn vip(id: String, cv: Option<String>, pass: Option<String>) -> &'static str {
    debug!("/vip id={id}, cv={cv:?}, pass={pass:?}");
    "true"
}

#[allow(unused_variables)]
#[get("/isRainbow.php?<id>")]
pub async fn is_rainbow(id: String) -> &'static str {
    r#"{"israinbow":false,"israinbowpastel":false,"hexcolor":"\#ffffff"}"#
}

#[get("/getInfo.php?<id>&<iid>")]
pub async fn get_info(id: String, iid: Option<String>) -> &'static str {
    debug!("getInfo.php id={id}, iid={iid:?}");
    "lmao"
}

#[allow(non_snake_case)]
#[get("/getIcon.php?<form>&<col1>&<col2>&<icon>&<id>&<glow>&<cubeID>")]
pub async fn get_icon(form: String, col1: String, col2: String, icon: String, id: String, glow: String, cubeID: String) -> Result<Vec<u8>, String> {
    debug!("getInfo.php form={form}, col1={col1}, col2={col2}, icon={icon}, id={id}, glow={glow}, cubeID={cubeID}");
    let redirect_url = format!("http://95.111.251.138/gdm/getIcon.php?form={form}&col1={col1}&col2={col2}&icon={icon}&id={id}&glow={glow}&cubeID={cubeID}");
    let client = Client::new();
    let result = client.get(redirect_url).send().await;
    // god i hate this
    Ok(result.and_then(|response| Ok(response.bytes())).map_err(|e| e.to_string())?.await.map_err(|e| e.to_string())?.into())
}

#[get("/lobbies/<path..>")]
pub async fn lobbies(path: PathBuf) -> io::Result<NamedFile> {
    let fname = path.file_name().unwrap();
    warn!("/lobbies/{fname:?} accessed, is unimplemented!");

    Err(io::Error::new(io::ErrorKind::Other, "oopsie"))
}

pub fn build_routes() -> Vec<Route> {
    routes![version, vip, is_rainbow, get_info, lobbies, get_icon]
}
