use url::{self, ParseError};

const WIKIPEDIA_HOST: &str = "wikipedia.org";

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct Url {
    val: url::Url,
}

impl Url {
    pub fn new(val: &str) -> Result<Self, ParseError> {
        let val = url::Url::parse(val)?;

        Ok(Self { val })
    }

    pub fn is_wiki(&self) -> bool {
        self.val.host_str().unwrap().contains(WIKIPEDIA_HOST)
    }

    pub fn val(&self) -> &str {
        self.val.as_str()
    }
}
