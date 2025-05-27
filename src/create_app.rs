use crate::api::controllers::hashtags::hashtags_config;
use crate::api::controllers::timeline::timeline_config;
use crate::container::Container;
use actix_files::Files;
use actix_web::App;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::middleware::Logger;
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
        .wrap(Logger::default())
        .wrap(middleware::Compress::default())
        .wrap(middleware::NormalizePath::trim())
        .configure(hashtags_config)
        .configure(timeline_config)
        .service(Files::new("/", "static").index_file("index.html"))
}
