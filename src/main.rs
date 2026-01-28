use anyhow::Result;
use scraper::{Html, Selector};
use std::collections::HashSet;

#[tokio::main]
async fn main() -> Result<()> {
    let url = "https://www.scrapingcourse.com/ecommerce/";

    let client = reqwest::Client::builder()
        .user_agent("ecommerce_scraper/0.1 (test)")
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let html: String = client.get(url).send().await?.text().await?;
    let document = Html::parse_document(&html);

    let title_selector = Selector::parse("title").unwrap();

    if let Some(title_element) = document.select(&title_selector).next() {
        let title_text = title_element.text().collect::<Vec<_>>().join("");

        println!("Found title: {}", title_text);
    } else {
        println!("Not title tag found!");
    }

    let links_selector = Selector::parse("a").unwrap();
    let links: HashSet<String> = document
        .select(&links_selector)
        .filter_map(|n| {
            n.value()
                .attr("href")
                .map(|s| s.to_string())
                .filter(|href| href.contains("/product/"))
        })
        .collect();

    println!("Total links found: {}", links.len());

    // for (i, item) in links.iter().enumerate() {
    //     println!("Link {}, {}", i + 1, item);
    //     if i > 3 {
    //         break;
    //     }
    // }

    let next_sel = Selector::parse("a.next.page-numbers").unwrap();

    let next_page = document
        .select(&next_sel)
        .next()
        .and_then(|el| el.value().attr("href"))
        .map(|href| href.to_string());

    match next_page {
        Some(url) => println!("Next page: {url}"),
        None => println!("No next page found on this page"),
    }

    Ok(())
}
