use http_body_util::{BodyExt, Empty};
use hyper::{body::Bytes, Request};
use hyper_util::rt::TokioIo;
use log::{debug, error, warn};
use roa::{
    http::StatusCode,
    preload::*,
    router::{get, Router},
    status, Context,
};
use tokio::{fs::File, net::TcpStream};

pub async fn version(context: &mut Context) -> roa::Result {
    let file = File::open("static/update.version").await?;
    context.write_reader(file);
    Ok(())
}

pub async fn is_vip(context: &mut Context) -> roa::Result {
    if cfg!(debug_assertions) {
        let id = &*context.must_query("id")?;
        let cv = context.query("cv");
        let pass = context.query("pass");

        debug!("/vip id={id:?}, cv={cv:?}, pass={pass:?}");
    }

    context.write("true");
    Ok(())
}

pub async fn is_rainbow(context: &mut Context) -> roa::Result {
    context.write(r##"{"israinbow":false,"israinbowpastel":false,"hexcolor":"#ffffff"}"##);
    Ok(())
}

pub async fn get_info(context: &mut Context) -> roa::Result {
    if cfg!(debug_assertions) {
        let id = &*context.must_query("id")?;
        let iid = context.query("iid");

        debug!("getInfo.php id={id:?}, iid={iid:?}");
    }

    context.write("lmao");
    Ok(())
}

#[allow(non_snake_case)]
pub async fn get_icon(context: &mut Context) -> roa::Result {
    let form = &*context.must_query("form")?;
    let col1 = &*context.must_query("col1")?;
    let col2 = &*context.must_query("col2")?;
    let icon = &*context.must_query("icon")?;
    let id = &*context.must_query("id")?;
    let glow = &*context.must_query("glow")?;
    let cube_id = &*context.must_query("cubeID")?;

    debug!("getIcon.php form={form}, col1={col1}, col2={col2}, icon={icon}, id={id}, glow={glow}, cubeID={cube_id}");

    let redirect_url = format!("http://95.111.251.138/gdm/getIcon.php?form={form}&col1={col1}&col2={col2}&icon={icon}&id={id}&glow={glow}&cubeID={cube_id}");
    let redirect_url = redirect_url.parse::<hyper::Uri>()?;
    let host = redirect_url
        .host()
        .ok_or(status!(StatusCode::BAD_REQUEST))?;

    let port = redirect_url.port_u16().unwrap_or(80);

    let address = format!("{host}:{port}");
    let stream = TcpStream::connect(address).await?;
    let io = TokioIo::new(stream);

    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            warn!("Connection failed: {:?}", err);
        }
    });

    let authority = redirect_url.authority().unwrap().clone();
    let req = Request::builder()
        .uri(redirect_url)
        .header(hyper::header::HOST, authority.as_str())
        .body(Empty::<Bytes>::new())?;

    let mut res = sender.send_request(req).await?;
    let status = res.status();
    if !status.is_success() {
        error!("GDM getIcon api returned error: {status:?}");
        return Err(status!(StatusCode::INTERNAL_SERVER_ERROR));
    }

    while let Some(next) = res.frame().await {
        let frame = next?;
        if let Some(chunk) = frame.data_ref() {
            context.write(chunk.to_owned());
        }
    }

    Ok(())
}

pub async fn lobbies(context: &mut Context) -> roa::Result {
    let fname = context.must_param("file")?;
    warn!("/lobbies/{fname:?} accessed, is unimplemented!");

    Err(status!(StatusCode::BAD_REQUEST))
}

pub fn build_router() -> Router<()> {
    Router::new()
        .gate(roa::query::query_parser)
        .on("/update.version", get(version))
        .on("/isVip.php", get(is_vip))
        .on("/isRainbow.php", get(is_rainbow))
        .on("/getInfo.php", get(get_info))
        .on("/getIcon.php", get(get_icon))
        .on("/lobbies/:file", get(lobbies))
}
