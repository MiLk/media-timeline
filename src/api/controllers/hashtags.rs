use crate::api::dto::hashtag::SuggestTagDTO;
use crate::domain::services::hashtag::SubscribedHashtagService;
use crate::domain::services::status::StatusService;
use actix_web::web::Html;
use actix_web::{HttpResponse, Responder, error, get, post, web};
use std::error::Error;
use tera::{Context, Tera};

#[get("")]
async fn list_tags(
    subscribed_hashtags_service: web::Data<dyn SubscribedHashtagService>,
    tmpl: web::Data<Tera>,
) -> Result<impl Responder, error::Error> {
    let hashtags: Vec<String> = subscribed_hashtags_service.list_hashtags()?;
    let mut context = Context::new();
    context.insert("hashtags", &hashtags);
    Ok(Html::new(
        tmpl.render("hashtags/list.html", &context)
            .map_err(|e| error::ErrorInternalServerError(e))?,
    ))
}

#[get("/popular")]
async fn list_popular_tags(
    status_service: web::Data<dyn StatusService>,
    tmpl: web::Data<Tera>,
) -> Result<impl Responder, error::Error> {
    let hashtags = status_service.popular_tags(vec![7, 30], 5)?;
    let mut context = Context::new();
    context.insert("hashtags", &hashtags);
    Ok(Html::new(
        tmpl.render("hashtags/list_popular.html", &context)
            .map_err(|e| error::ErrorInternalServerError(e))?,
    ))
}

#[post("")]
async fn suggest_tag(
    subscribed_hashtags_service: web::Data<dyn SubscribedHashtagService>,
    form: web::Form<SuggestTagDTO>,
) -> Result<impl Responder, Box<dyn Error>> {
    subscribed_hashtags_service
        .suggest_hashtag(form.hashtag.as_str())
        .await?;
    Ok(HttpResponse::Ok()
        .append_header(("HX-Trigger", "tags-updated"))
        .finish())
}

pub fn hashtags_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tags")
            .service(list_tags)
            .service(list_popular_tags)
            .service(suggest_tag),
    );
}
