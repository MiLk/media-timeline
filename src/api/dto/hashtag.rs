use serde::Deserialize;

#[derive(Deserialize)]
pub struct SuggestTagDTO {
    pub hashtag: String,
}
