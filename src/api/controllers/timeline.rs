use crate::domain::services::hashtag::SubscribedHashtagService;
use crate::domain::services::status::StatusService;
use actix_web::web::Html;
use actix_web::{error, get, web, Responder};
use log::debug;
use megalodon::entities::Status;
use serde::Serialize;
use std::cmp::Reverse;
use tera::{Context, Tera};

#[derive(Serialize)]
struct TimelineContext {
    statuses: Vec<Status>,
}

#[get("")]
async fn get_timeline(
    subscribed_hashtag_service: web::Data<dyn SubscribedHashtagService>,
    status_service: web::Data<dyn StatusService>,
    tmpl: web::Data<Tera>,
) -> Result<impl Responder, error::Error> {
    let hashtags = subscribed_hashtag_service.list_hashtags()?;

    let mut statuses = status_service.retrieve_statuses(Some(&hashtags)).await?;
    statuses.sort_by_key(|status| Reverse(status.created_at));

    debug!("{} statuses retrieved from storage", statuses.len());

    let timeline_context = TimelineContext { statuses };
    Context::from_serialize(timeline_context)
        .and_then(|context| tmpl.render("timeline.html", &context))
        .map(|rendered| Html::new(rendered))
        .map_err(error::ErrorInternalServerError)
}

pub fn timeline_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/timeline").service(get_timeline));
}
