use crate::storage::Storage;
use actix_web::web::Html;
use actix_web::{HttpResponse, Responder, error, get, post, web};
use serde::Deserialize;
use tera::{Context, Tera};

#[get("")]
async fn list_tags(
    storage: web::Data<Storage>,
    tmpl: web::Data<Tera>,
) -> Result<impl Responder, error::Error> {
    let hashtags: Vec<String> = storage.list_hashtags();
    let mut context = Context::new();
    context.insert("hashtags", &hashtags);
    Ok(Html::new(
        tmpl.render("hashtags/list.html", &context)
            .map_err(|e| error::ErrorInternalServerError(e))?,
    ))
}

#[get("/popular")]
async fn list_popular_tags(
    storage: web::Data<Storage>,
    tmpl: web::Data<Tera>,
) -> Result<impl Responder, error::Error> {
    let hashtags = storage.popular_tags(vec![7, 30], 5)?;
    let mut context = Context::new();
    context.insert("hashtags", &hashtags);
    Ok(Html::new(
        tmpl.render("hashtags/list_popular.html", &context)
            .map_err(|e| error::ErrorInternalServerError(e))?,
    ))
}

#[derive(Deserialize)]
struct SuggestTagFormData {
    hashtag: String,
}

#[post("")]
async fn suggest_tag(
    storage: web::Data<Storage>,
    form: web::Form<SuggestTagFormData>,
) -> impl Responder {
    storage.suggest_hashtag(form.hashtag.as_str()).await;
    HttpResponse::Ok()
        .append_header(("HX-Trigger", "tags-updated"))
        .finish()
}

pub fn hashtags_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tags")
            .service(list_tags)
            .service(list_popular_tags)
            .service(suggest_tag),
    );
}
