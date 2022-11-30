use std::{collections::HashSet, fmt::Display};

use scraper::{Html, Selector};

use crate::{filtering, query, Queriable};

#[derive(Debug, Eq, PartialEq, Clone, Hash, Default)]
pub struct WikiPage(pub String);
impl Queriable for WikiPage {
    fn full_title(&self) -> String {
        self.0.to_string()
    }
}

impl Display for WikiPage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl WikiPage {
    pub async fn page_contents(page: String) -> Vec<String> {
        let params = &[
            ("action", "parse"),
            ("prop", "text"),
            ("formatversion", "2"),
            ("page", &page),
        ];
        let res = query(params).await;
        let text = match res["parse"]["text"].as_str() {
            Some(text) => text,
            None => {
                println!("{res:?}");
                println!("Error parsing page {page}");
                return Vec::new();
            }
        };

        let fragment = Html::parse_fragment(text);
        let selector = Selector::parse("div").unwrap();

        let div = fragment.select(&selector).next().unwrap();
        let mut text: Vec<String> = div.text().map(|e| filtering::filter_noise(e)).collect();
        // let mut text: HashSet<String> = div.text().map(|e| filtering::filter_noise(e)).collect();
        text.retain(|e| !filtering::IGNORED_ENTRIES.contains(e as &str));
        text
    }

    pub async fn neighbors(page: &str) -> Vec<WikiPage> {
        let params = &[
            ("action", "query"),
            ("prop", "links"),
            ("pllimit", "max"),
            ("plnamespace", "0"),
            ("titles", page),
        ];
        let res = query(params).await;
        let pages = res["query"]["pages"].as_object().unwrap();
        let pages: Vec<(&String, &serde_json::Value)> = pages.iter().collect();
        let pages = pages[0].1.as_object().unwrap();
        let pages = pages["links"].as_array().unwrap();
        pages
            .iter()
            .map(|item| WikiPage(item["title"].as_str().unwrap().to_string()))
            .collect()
    }

    pub async fn linkshere(page: &str) -> Vec<WikiPage> {
        let params = &[
            ("action", "query"),
            ("prop", "linkshere"),
            ("lhnamespace", "0"),
            ("titles", page),
        ];
        let res = query(params).await;
        let pages = res["query"]["pages"].as_object().unwrap();
        let pages: Vec<(&String, &serde_json::Value)> = pages.iter().collect();
        let pages = pages[0].1.as_object().unwrap();
        let pages = pages["linkshere"].as_array().unwrap();
        pages
            .iter()
            .map(|item| WikiPage(item["title"].as_str().unwrap().to_string()))
            .collect()
    }
}
