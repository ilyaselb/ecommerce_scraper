use anyhow::Result;
use scraper::{Html, Selector};
use std::collections::HashSet;

async fn fetch_html(client: &reqwest::Client, url: &str) -> Result<String> {
    let html = client.get(url).send().await?.text().await?;

    Ok(html)
}

fn parse_listing_page(html: &str) -> Result<(HashSet<String>, Option<String>)> {
    let document = Html::parse_document(html);
    let links_selector =
        Selector::parse("a").map_err(|e| anyhow::anyhow!("bad selector 'a': {e:?}"))?;

    let links: HashSet<String> = document
        .select(&links_selector)
        .filter_map(|n| {
            n.value()
                .attr("href")
                .map(|s| s.to_string())
                .filter(|href| href.contains("/product/"))
        })
        .collect();

    let next_sel = Selector::parse("a.next.page-numbers")
        .map_err(|e| anyhow::anyhow!("bad selector a.next.page-numbers: {e:?}"))?;

    let next_page = document
        .select(&next_sel)
        .next()
        .and_then(|el| el.value().attr("href"))
        .map(|href| href.to_string());

    Ok((links, next_page))
}

async fn crawl_shop(client: &reqwest::Client, start_url: &str) -> Result<HashSet<String>> {
    let mut visited_pages = HashSet::new();
    let mut all_products = HashSet::new();
    let mut current = Some(start_url.to_string());

    while let Some(ref page_url) = current {
        if !visited_pages.insert(page_url.clone()) {
            break;
        }

        let html = fetch_html(client, &page_url).await?;
        let (links, next) = parse_listing_page(&html)?;

        all_products.extend(links);

        println!("Visited: {page_url}");
        println!("Total unique products so far: {}", all_products.len());
        println!("Next: {}", next.as_deref().unwrap_or("None"));

        current = next;
    }

    Ok(all_products)
}

#[tokio::main]
async fn main() -> Result<()> {
    let url = "https://www.scrapingcourse.com/ecommerce/";

    let client = reqwest::Client::builder()
        .user_agent("ecommerce_scraper/0.1 (test)")
        .timeout(std::time::Duration::from_secs(10))
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

    crawl_shop(&client, url).await?;

    Ok(())
}
