use crate::api::controllers::hashtags::hashtags_config;
use crate::api::controllers::timeline::timeline_config;
use crate::container::Container;
use actix_files::Files;
use actix_web::App;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::http::header;
use actix_web::middleware::{Condition, Logger};
use actix_web::{Error, middleware};
use std::sync::Arc;

pub fn create_app(
    container: Arc<Container>,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = Error,
    >,
> {
    App::new()
        .configure(|cfg| container.config(cfg))
        .wrap(Condition::new(
            container.settings.actix.enable_log,
            Logger::new(r#"%{r}a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T"#),
        ))
        .wrap(Condition::new(
            container.settings.actix.enable_compression,
            middleware::Compress::default(),
        ))
        // Without an explicit directive, intermediaries invent their own TTL:
        // Cloudflare served a two-month-old index.js alongside fresh HTML. Handlers
        // that set their own Cache-Control (the timeline) keep it; everything else,
        // static assets included, must revalidate against the ETag actix-files sends.
        .wrap(middleware::DefaultHeaders::new().add((header::CACHE_CONTROL, "no-cache")))
        .wrap(middleware::NormalizePath::trim())
        .configure(hashtags_config)
        .configure(timeline_config)
        .service(Files::new("/", "static").index_file("index.html"))
}
