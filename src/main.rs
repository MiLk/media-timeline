mod mastodon;
mod services;
mod storage;
mod templating;

use crate::mastodon::MastodonClient;
use crate::services::hashtags::hashtags_config;
use crate::services::timeline::timeline_config;
use crate::storage::Storage;
use crate::storage::sqlite::SqliteDal;
use crate::templating::init_tera;
use actix_files::{Files, NamedFile};
use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::{App, HttpServer, get, middleware};
use std::env;
use std::error::Error;

#[get("/")]
async fn index() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("./static/index.html")?)
}

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let tera_data = Data::new(init_tera()?);

    let user_agent = Some(String::from(format!("{}/{}", PKG_NAME, PKG_VERSION)));
    let client = Data::new(MastodonClient::new(
        "https://dice.camp".to_owned(),
        user_agent,
    )?);

    let storage = Data::new(Storage::new(SqliteDal::new()?).await?);

    let listen_addr = env::var("LISTEN_ADDR").unwrap_or("127.0.0.1".to_owned());
    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Compress::default())
            .wrap(middleware::NormalizePath::trim())
            .wrap(Logger::default())
            .app_data(tera_data.clone())
            .app_data(client.clone())
            .app_data(storage.clone())
            .configure(hashtags_config)
            .configure(timeline_config)
            .service(Files::new("/", "static").index_file("index.html"))
    })
    .bind((listen_addr, 1337))?;

    for (addr, scheme) in server.addrs_with_scheme() {
        println!("Listening on {}://{}", scheme, addr);
    }

    server.run().await?;
    Ok(())
}
