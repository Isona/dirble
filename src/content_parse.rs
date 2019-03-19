extern crate select;
use select::document::Document;
use select::predicate::Name;

pub fn scrape_urls(content: String, original_url: String) -> Vec<String>
{
    let mut output_urls:Vec<String> = Vec::new();
    let mut scraped_urls: Vec<String> = Vec::new();


    Document::from_read(content.as_bytes()).unwrap()
        .find(Name("a"))
        .filter_map(|n| n.attr("href"))
        .for_each(|scraped_url| scraped_urls.push(String::from(scraped_url)) );


    for scraped_url in scraped_urls {
        let mut _complete_url = String::new();

        if scraped_url.starts_with("../") || scraped_url.starts_with("?") 
            || scraped_url.starts_with("./"){
                continue
        }
        else if scraped_url.starts_with("/") {
            // need to get the base address from the original url and append this
            let mut start_index = 7;
            if original_url.starts_with("https://") {
                start_index = 8;
            }
            let end_index = original_url[start_index..].find("/").unwrap();
            _complete_url = format!("{}{}", &original_url[0..end_index+start_index], scraped_url);
        }
        else if scraped_url.contains("://") {
            _complete_url = scraped_url;
        }
        else {
            _complete_url = format!("{}{}", original_url, scraped_url);
        }

        if !original_url.starts_with(&_complete_url) && _complete_url.starts_with(&original_url)
        {
            output_urls.push(_complete_url);
        }

    }

    output_urls
}