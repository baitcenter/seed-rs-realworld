use serde::Deserialize;
use crate::entity::{Viewer, form::settings::{ValidForm, Problem}};
use crate::{request, coder::decoder};
use futures::prelude::*;
use seed::fetch;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RootDecoder {
    user: decoder::Viewer
}

pub fn update<Ms: 'static>(
    viewer: Option<&Viewer>,
    valid_form: &ValidForm,
    f: fn(Result<Viewer, Vec<Problem>>) -> Ms
) -> impl Future<Item=Ms, Error=Ms>  {
    request::new_api_request(
        "user",
        viewer
    )
        .method(fetch::Method::Put)
        .send_json(&valid_form.to_encoder())
        .fetch_json_data(move |data_result: fetch::ResponseDataResult<RootDecoder>| {
            f(data_result
                .map(|root_decoder| root_decoder.user.into_viewer())
                .map_err(request::fail_reason_into_problems)
            )
        })
}
