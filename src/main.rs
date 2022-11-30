use std::collections::HashSet;

use futures::{stream, StreamExt};
use inquire::{MultiSelect, Select};
use itertools::Itertools;
use mediawiki::page::Page;
use wikipedia2jpdb::{category::Category, wikipage::WikiPage, Queriable};

const YES_NO: [&str; 2] = ["Yes", "No"];

//TODO discarded pages set
#[derive(Default, Debug)]
struct Execution {
    pub picked_cateories: HashSet<Category>,
    pub discarded_categories: HashSet<Category>,
    pub pages: HashSet<WikiPage>,
    pub pages_stack: HashSet<WikiPage>,
    pub selection: Vec<Category>,
    pub stack: Vec<Category>,
}

impl Execution {
    pub async fn stack_to_selection(&mut self) {
        // dbg!(&self.stack);
        let mut info = stack_to_info(&self.stack).await;
        info.retain(|category| {
            !self.discarded_categories.contains(category)
                && !self.picked_cateories.contains(category)
        });

        self.selection = MultiSelect::new("Select the categories you'd like: ", info.clone())
            .with_page_size(30)
            .with_default(&Vec::from_iter(0..info.len()))
            .prompt()
            .unwrap_or_default();
        self.picked_cateories.extend(self.selection.iter().cloned());
        info.retain(|e| !self.picked_cateories.contains(e));
        self.discarded_categories.extend(info.iter().cloned());
    }

    pub async fn select_pages(&mut self, pages: Vec<WikiPage>) -> Vec<WikiPage> {
        let len = pages.len();
        let string = format!("Select the pages you'd like: (/{})", len);
        MultiSelect::new(&string, pages)
            .with_default(&Vec::from_iter(0..len))
            .with_page_size(30)
            .prompt()
            .unwrap_or_default()
    }

    pub async fn push_pages(&mut self) {
        let pages: Vec<_> = self
            .selection
            .iter()
            .map(|category| {
                let category = category.clone();
                tokio::spawn(async move { Category::pages(category.full_title().as_str()).await })
            })
            .collect();
        let awaiting = futures::future::join_all(pages).await;
        dbg!(&awaiting);
        awaiting
            .into_iter()
            .filter_map(|v| v.ok())
            .flatten()
            .for_each(|page| {
                self.pages_stack.insert(page);
            });
    }
}

pub fn join_titles(stack: &[Category]) -> String {
    let titles: Vec<String> = stack
        .iter()
        .map(|item| format!("Category:{}", item.2))
        .collect();
    titles.join("|")
}

pub async fn stack_to_info(stack: &[Category]) -> Vec<Category> {
    let substacks: Vec<_> = stack
        .chunks(20)
        .map(|chunk| {
            let category = chunk.to_vec();
            tokio::spawn(async move { Category::info(&join_titles(&category)).await })
        })
        .collect();
    let awaiting: Vec<_> = futures::future::join_all(substacks).await;
    awaiting
        .into_iter()
        .filter_map(|v| v.ok())
        .flatten()
        .collect()
}

#[tokio::main]
pub async fn main() {
    let mut execution = Execution {
        selection: Vec::with_capacity(50),
        stack: Vec::with_capacity(50),
        ..Default::default()
    };
    // WikiPage::page_contents("自転車").await;

    // Category::subcategories("Category:天文学");
    // TODO validator of the url?
    // let origin = Text::new("What is the page you want to start crawling from?")
    //     .with_placeholder("Category:Something or Page:Something")
    //     .with_validator(required!())
    //     .prompt()
    //     .unwrap();
    // let title = "Category:天文学";
    let title = "自転車";
    if title.contains("Category:") {
        let root_cat = Category::from_full(title);
        execution
            .pages
            .extend(Category::pages(root_cat.full_title().as_str()).await);
        execution
            .stack
            .extend_from_slice(&Category::subcategories(root_cat.full_title().as_str()).await);
        execution.picked_cateories.insert(root_cat);
    } else {
        let root_page = WikiPage(title.to_owned());
        execution.pages.insert(root_page);
        // TODO ask if dood wants to see linking pages
        let mut pages_to_select = Vec::with_capacity(50);
        let ans = Select::new("Search for parent pages ? (usually not recommended, as it can add thousands of pretty unrelated pages)", YES_NO.to_vec()).prompt();
        if let Ok("Yes") = ans {
            pages_to_select.extend(WikiPage::linkshere(title).await);
        }
        pages_to_select.extend(WikiPage::neighbors(title).await);
        let selected_pages = execution.select_pages(pages_to_select).await;
        execution.pages.extend(selected_pages);
    }

    execution
        .stack
        .extend_from_slice(&Category::parent_categories(title, true).await);

    loop {
        execution.stack_to_selection().await;

        println!("Fetching Page List, this might take some time...");
        execution.push_pages().await;
        let ans = Select::new("Shall we go deeper ?", YES_NO.to_vec()).prompt();
        match ans {
            Ok(choice) if choice == "No" => break,
            Ok(_) => {}
            Err(_) => println!("There was an error, please try again"),
        }

        let subcategories: Vec<_> = execution
            .selection
            .iter()
            .map(|category| {
                let category = category.clone();
                tokio::spawn(async move {
                    Category::subcategories(category.full_title().as_str()).await
                })
            })
            .collect();
        let awaiting = futures::future::join_all(subcategories).await;
        execution.stack.clear();
        execution.stack = awaiting
            .into_iter()
            .filter_map(|v| v.ok())
            .flatten()
            .collect();
        if execution.stack.is_empty() {
            println!("No more subcategories left to explore! Processing start!");
            break;
        }
    }

    let selected_pages = execution
        .select_pages(Vec::from_iter(execution.pages_stack.clone()))
        .await;
    execution.pages.extend(selected_pages);

    dbg!(execution.picked_cateories.len());
    dbg!(execution.pages.len());

    //TODO try to clean up to use &str again, because of the async stuff
    // maybe with execution.pages.iter instead
    let thingies: Vec<Vec<String>> = stream::iter(execution.pages)
        .map(|page| {
            // let page = page.clone();
            WikiPage::page_contents(page.0)
        })
        .buffer_unordered(100)
        .collect()
        .await;
    // TODO split in chunks of x entries, make a string, write to file, instead, to save multiple output files that can then be copy pasted sequentially into jpdb
    let text: Vec<String> = thingies
        .iter()
        .flatten()
        .cloned()
        .coalesce(|lh, rh| {
            if lh.len() + rh.len() < 400000 {
                Ok(lh + &rh)
            } else {
                Err((lh, rh))
            }
        })
        .collect();
    dbg!(text.len());
    text.iter().enumerate().for_each(|(i, contents)| {
        let file_path = format!("TODOii{i}.txt");
        std::fs::write(file_path, &contents).unwrap();
    });
    // let text = text.join("");
    // dbg!(thingies);
    // let awaiting: Vec<_> = futures::future::join_all(substacks).await;
    // let contents: Vec<_> = awaiting.into_iter().filter_map(|v| v.ok()).collect();
    // dbg!(contents);
}
