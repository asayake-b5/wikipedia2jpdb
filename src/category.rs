use std::fmt::Display;

use crate::{query, wikipage::WikiPage, Queriable};

impl Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({} subcats, {} pages)", self.2, self.1, self.0)
    }
}

#[derive(Debug, Eq, Clone, PartialEq, Hash)]
pub struct Category(pub u64, pub u64, pub String);

impl Category {
    pub async fn pages(full: &str) -> Vec<WikiPage> {
        let params = &[
            ("action", "query"),
            ("list", "categorymembers"),
            ("cmtype", "page"),
            ("cmtitle", full),
        ];
        let res = query(params).await;
        res["query"]["categorymembers"]
            .as_array()
            .unwrap()
            .iter()
            // filter out portals and whatnot
            .filter(|value| value["ns"] == 0)
            .filter_map(|value| value["title"].as_str())
            .map(|s| WikiPage(s.to_owned()))
            .collect()
    }
    pub async fn parent_categories(full: &str, filter_hidden: bool) -> Vec<Category> {
        let params = &[
            ("action", "query"),
            ("prop", "categories"),
            ("clprop", "hidden"),
            ("titles", full),
        ];
        let res = query(params).await;
        let categories: Vec<Category> = res["query"]["pages"]
            .as_object()
            .unwrap()
            .iter()
            .flat_map(|(_page_id, page)| {
                page["categories"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|c| c["hidden"].as_str().is_none())
                    .map(|c| Category::from_full(c["title"].as_str().unwrap()))
            })
            .collect();
        categories
    }

    pub async fn subcategories(full: &str) -> Vec<Category> {
        let params = &[
            ("action", "query"),
            ("list", "categorymembers"),
            ("cmtype", "subcat"),
            ("cllimit", "500"),
            ("cmtitle", full),
        ];
        let res = query(params).await;
        let categories: Vec<Category> = res["query"]["categorymembers"]
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|v| v["title"].as_str())
            .map(Category::from_full)
            .collect();
        categories
    }

    pub async fn info(full: &str) -> Vec<Category> {
        let params = &[
            ("action", "query"),
            ("prop", "categoryinfo"),
            ("climit", "500"),
            ("titles", full),
        ];
        let res = query(params).await;
        if let Some(r) = res["query"]["pages"].as_object() {
            r.iter()
                .map(|(_, value)| {
                    let pages_count = value["categoryinfo"]["pages"].as_u64().unwrap();
                    let subcat_count = value["categoryinfo"]["subcats"].as_u64().unwrap();
                    let title = value["title"].as_str().unwrap().to_string();
                    let r = Category::from_full(&title);
                    Category(pages_count, subcat_count, r.2)
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    // TODO overhaul this, return result, remove panic, remove dafuk
    pub fn from_full(full: &str) -> Self {
        let f = full.find("Category:");
        let title = if let Some(index) = f {
            full[index + 9..].to_owned()
        } else {
            panic!("dafuk");
        };

        Category(0, 0, title)
    }
}

impl Queriable for Category {
    fn full_title(&self) -> String {
        format!("Category:{}", self.2)
    }
}
