extern crate select;
use select::document::Document;
use select::predicate::Name;

// Returns complete URLs based on the contents of a listable folder
pub fn scrape_urls(content: String, original_url: String) -> Vec<String>
{
    let mut output_urls:Vec<String> = Vec::new();
    let mut scraped_urls: Vec<String> = Vec::new();

    // Get the contents of href attributes from the given response content
    Document::from_read(content.as_bytes()).unwrap()
        .find(Name("a"))
        .filter_map(|n| n.attr("href"))
        .for_each(|scraped_url| scraped_urls.push(String::from(scraped_url)) );

    // Parse urls and add them to the list of urls to return
    for scraped_url in scraped_urls {
        let mut _complete_url = String::new();

        // If a url starts with of these then it is potentially a parent
        // or a mechanism for sorting the directory
        // not of interest or may cause issues when scanning so are skipped
        if scraped_url.starts_with("../") || scraped_url.starts_with("?") 
            || scraped_url.starts_with("./"){
                continue
        }
        // The scaped url is a path from the base URL
        else if scraped_url.starts_with("/") {
            // need to get the base address from the original url and append this
            let mut start_index = 7;
            if original_url.starts_with("https://") {
                start_index = 8;
            }
            let end_index = original_url[start_index..].find("/").unwrap();
            _complete_url = format!("{}{}", 
                &original_url[0..end_index+start_index], scraped_url);
        }
        // Where the URL is a complete url that doesn't need modifying
        else if scraped_url.contains("://") {
            _complete_url = scraped_url;
        }

        // Relative paths from the current directory
        else {
            _complete_url = format!("{}{}", original_url, scraped_url);
        }

        // Only add to the list if it's a subdirectory of the current directory
        // And if the current directory doesn't begin with it
        if !original_url.starts_with(&_complete_url) && _complete_url.starts_with(&original_url)
        {
            output_urls.push(_complete_url);
        }

    }

    output_urls
}