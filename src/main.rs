#![feature(iterator_try_collect)]

mod mastodon;
mod services;
mod storage;

use crate::mastodon::MastodonClient;
use crate::services::hashtags::hashtags_config;
use crate::services::timeline::timeline_config;
use crate::storage::Storage;
use actix_files::{Files, NamedFile};
use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::{App, HttpServer, get, middleware};
use chrono::{DateTime, Utc};
use log::debug;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::ops::Sub;
use tera::{to_value, try_get_value};

#[get("/")]
async fn index() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("./static/index.html")?)
}

fn timedelta_filter(
    value: &tera::Value,
    _args: &HashMap<String, tera::Value>,
) -> tera::Result<tera::Value> {
    let v = try_get_value!("timedelta_filter", "value", String, value);
    let datetime = DateTime::parse_from_rfc3339(v.as_str())
        .map_err(|err| tera::Error::from(err.to_string()))?;
    let delta = Utc::now().with_timezone(datetime.offset()).sub(datetime);
    if delta.num_days() > 0 {
        return Ok(to_value(format!("{}d", delta.num_days()))?);
    }
    if delta.num_hours() > 0 {
        return Ok(to_value(format!("{}h", delta.num_hours()))?);
    }
    Ok(to_value(format!("{}m", delta.num_minutes()))?)
}

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let mut tera = match tera::Tera::new("templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    tera.register_filter("timedelta", timedelta_filter);
    let tera_data = Data::new(tera);

    let user_agent = Some(String::from(format!("{}/{}", PKG_NAME, PKG_VERSION)));
    debug!("Using the following User-Agent: {:?}", user_agent);
    let client =
        Data::new(MastodonClient::new("https://dice.camp".to_owned(), user_agent).unwrap());

    let storage = Data::new(Storage::new().await.unwrap());
    storage.rebuild_index_statuses().await?;

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
