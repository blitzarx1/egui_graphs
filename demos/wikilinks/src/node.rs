use crate::url::Url;

#[derive(Clone)]
pub struct Node {
    url: Url,
}

impl Node {
    pub fn new(url: Url) -> Self {
        Self { url }
    }

    pub fn url(&self) -> &Url {
        &self.url
    }
}
