use serde::Deserialize;
use crate::entity::{form::article_editor as form, Credentials, article};
use crate::{request, dto};
use futures::prelude::*;
use seed::fetch;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RootDto {
    article: dto::ArticleDto
}

pub fn create<Ms: 'static>(
    credentials: Option<Credentials>,
    valid_form: &form::ValidForm,
    f: fn(Result<article::Article, Vec<form::Problem>>) -> Ms
) -> impl Future<Item=Ms, Error=Ms>  {
    request::new_api_request(
        "articles",
        credentials.as_ref()
    )
        .method(fetch::Method::Post)
        .send_json(&valid_form.dto())
        .fetch_json_data(move |data_result: fetch::ResponseDataResult<RootDto>| {
            f(data_result
                .map_err(request::fail_reason_into_problems)
                .and_then(move |root_dto| {
                    root_dto.article.try_into_article(credentials)
                        .map_err(|error| vec![form::Problem::new_server_error(error)])
                })
            )
        })
}


