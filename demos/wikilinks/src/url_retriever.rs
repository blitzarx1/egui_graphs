use crossbeam::channel::Sender;
use log::{debug, error, info};
use reqwest::Error;
use scraper::{Html, Selector};
use tokio::task::JoinHandle;
use url::quirks::protocol;

use crate::url::Url;

/// Retrieves urls from wikipedia article
pub struct UrlRetriever {
    client: reqwest::Client,
    results: Sender<Result<Url, Error>>,
}

impl UrlRetriever {
    pub fn new(results: Sender<Result<Url, Error>>) -> Self {
        Self {
            client: reqwest::Client::new(),
            results,
        }
    }

    pub fn run(self, task: Url) -> JoinHandle<()> {
        tokio::spawn(async move {
            let results = self.get_links(&task).await;

            match results {
                Ok(urls) => {
                    for url in urls {
                        self.results.send(Ok(url)).unwrap();
                    }
                }
                Err(err) => {
                    self.results.send(Err(err));
                }
            }
        })
    }

    pub async fn get_links(&self, url: &Url) -> Result<Vec<Url>, Error> {
        let mut links = Vec::new();
        let res = self.client.get(url.val()).send().await?;

        let protocol = format!("{}://", res.url().scheme());
        let host = res.url().host().unwrap().to_string();

        let doc = Html::parse_document(res.text().await?.as_str());
        let a_selector = Selector::parse("a").unwrap();

        doc.select(&a_selector).for_each(|selection| {
            let href_val = selection.value().attr("href").unwrap_or_default();

            if let Some(url) =
                self.parse_href_val(protocol.clone(), host.clone(), href_val.to_string())
            {
                info!("found url: {}", url.val());
                links.push(url);
            }
        });

        Ok(links)
    }

    pub fn parse_href_val(&self, protocol: String, host: String, href_val: String) -> Option<Url> {
        let mut href_val_final = href_val;

        if href_val_final.starts_with("/wiki") | href_val_final.starts_with("/w") {
            href_val_final = format!("{}{}{}", protocol, host, href_val_final);
            info!(
                "transformed relative url to absolute url: {}",
                href_val_final
            );
        }

        if href_val_final.starts_with("//") {
            href_val_final = format!("{}{}", protocol, href_val_final);
            info!(
                "transformed relative url to absolute url: {}",
                href_val_final
            );
        }

        let url = Url::new(href_val_final.as_str());
        match url {
            Ok(url) => Some(url),
            Err(err) => {
                error!("error parsing url: {} -  {}", href_val_final, err);
                None
            }
        }
    }
}
