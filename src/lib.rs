pub mod category;
pub mod filtering;
pub mod wikipage;

pub trait Queriable {
    fn full_title(&self) -> String;
}

async fn query(params: &[(&str, &str)]) -> serde_json::value::Value {
    let api = mediawiki::api::Api::new("https://jp.wikipedia.org/w/api.php")
        .await
        .unwrap();
    let params = api.params_into(params);
    api.get_query_api_json_all(&params).await.unwrap()
}
