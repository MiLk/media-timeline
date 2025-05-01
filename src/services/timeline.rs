use crate::mastodon::MastodonClient;
use actix_web::web::Html;
use actix_web::{error, get, web, Responder};
use megalodon::entities;
use serde::Serialize;
use tera::{Context, Tera};

#[derive(Serialize)]
struct TimelineContext {
    statuses: Vec<entities::Status>,
}

#[get("")]
async fn get_timeline(
    tmpl: web::Data<Tera>,
    mastodon_client: web::Data<MastodonClient>,
) -> Result<impl Responder, error::Error> {
    let statuses = mastodon_client
        .get_tag_timeline("hobbystreak".to_owned())
        .await
        .map_err(error::ErrorInternalServerError)?;
    // for status in &statuses {
    //     println!("{:#?}", status);
    // }
    let timeline_context = TimelineContext { statuses };
    Context::from_serialize(timeline_context)
        .and_then(|context| tmpl.render("timeline.html", &context))
        .map(|rendered| Html::new(rendered))
        .map_err(error::ErrorInternalServerError)
}

pub fn timeline_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/timeline").service(get_timeline));
}
