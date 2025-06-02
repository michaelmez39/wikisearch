#![allow(dead_code)]
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename = "mediawiki")]
pub struct MediaWiki {
    #[serde(rename = "@version")]
    pub version: String,

    #[serde(rename = "@lang")]
    pub lang: String,

    #[serde(rename = "siteinfo")]
    pub siteinfo: Option<SiteInfo>,

    #[serde(rename = "page")]
    pub pages: Vec<Page>,
}

#[derive(Debug, Deserialize)]
pub struct SiteInfo {
    pub sitename: Option<String>,
    pub dbname: Option<String>,
    pub base: Option<String>,
    pub generator: Option<String>,
    #[serde(rename = "case")]
    pub case_type: Option<String>,
    pub namespaces: Option<Namespaces>,
}

#[derive(Debug, Deserialize)]
pub struct Namespaces {
    #[serde(rename = "namespace")]
    pub namespaces: Vec<Namespace>,
}

#[derive(Debug, Deserialize)]
pub struct Namespace {
    #[serde(rename = "$value")]
    pub name: Option<String>,

    #[serde(rename = "@key")]
    pub key: Option<i32>,

    #[serde(rename = "@case")]
    pub case_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Page {
    pub title: String,
    pub ns: u32,
    pub id: u64,
    pub redirect: Option<Redirect>,
    pub restrictions: Option<String>,

    #[serde(rename = "revision")]
    pub revisions: Option<Vec<Revision>>,
}

#[derive(Debug, Deserialize)]
pub struct Redirect {
    #[serde(rename = "@title")]
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct Revision {
    pub id: u64,
    pub parentid: Option<u64>,
    pub timestamp: String,
    pub contributor: Contributor,
    pub minor: Option<String>,
    pub comment: Option<Comment>,
    pub origin: u64,
    pub model: String,
    pub format: String,
    pub text: TextType,
    pub content: Option<Vec<Content>>,
    pub sha1: String,
}

#[derive(Debug, Deserialize)]
pub struct Contributor {
    pub username: Option<String>,
    pub id: Option<u64>,
    pub ip: Option<String>,

    #[serde(rename = "@deleted")]
    pub deleted: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Comment {
    #[serde(rename = "$value")]
    pub value: Option<String>,

    #[serde(rename = "@deleted")]
    pub deleted: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TextType {
    #[serde(rename = "$value")]
    pub value: Option<String>,

    #[serde(rename = "@space")]
    pub xml_space: Option<String>,

    #[serde(rename = "@deleted")]
    pub deleted: Option<String>,

    #[serde(rename = "@id")]
    pub id: Option<String>,

    #[serde(rename = "@location")]
    pub location: Option<String>,

    #[serde(rename = "@sha1")]
    pub sha1: Option<String>,

    #[serde(rename = "@bytes")]
    pub bytes: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct Content {
    pub role: String,
    pub origin: u64,
    pub model: String,
    pub format: String,
    pub text: ContentTextType,
}

#[derive(Debug, Deserialize)]
pub struct ContentTextType {
    #[serde(rename = "$value")]
    pub value: Option<String>,

    #[serde(rename = "space")]
    pub xml_space: Option<String>,

    #[serde(rename = "@deleted")]
    pub deleted: Option<String>,

    #[serde(rename = "@location")]
    pub location: Option<String>,

    #[serde(rename = "@sha1")]
    pub sha1: Option<String>,

    #[serde(rename = "@bytes")]
    pub bytes: Option<u64>,
}

#[cfg(test)]
mod test {
    use super::MediaWiki;
    use anyhow::Error as E;
    use anyhow::Result as AnyResult;
    use tokio::io::AsyncWriteExt;

    const ARTICLES: &'static [&'static str] = &[
        "Royal_Automobile_Club_of_Queensland",
        "Break_My_Bank",
        "Santiago_Files",
        "Glenn_Frey",
        "Viola_Pitts",
        "Battle_of_Pouancé",
        "August_Borsig",
        "Convention_City",
    ];

    #[test]
    fn deserialize() -> Result<(), E> {
        let test_xml = include_str!("../resources/articles/August_Borsig.xml");
        let media_wiki: MediaWiki = quick_xml::de::from_str(test_xml)?;
        Ok(())
    }

    #[test]
    #[ignore = "utility to download articles"]
    fn download() -> AnyResult<()> {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(download_articles(ARTICLES))?;
        Ok(())
    }

    async fn download_articles(articles: &[&str]) -> Result<(), E> {
        let client = reqwest::Client::new();
        for article in articles {
            save_article(&client, article).await?;
        }
        Ok(())
    }

    async fn save_article(client: &reqwest::Client, article_title: &str) -> Result<(), E> {
        let url = format!("https://en.wikipedia.org/wiki/Special:Export/{article_title}");
        let article = client.get(url).send().await?.text().await?.into_bytes();
        let file_name = format!("resources/articles/{article_title}.xml");
        let mut file = tokio::fs::File::create(file_name).await?;
        file.write_all(&article).await?;
        Ok(())
    }
}
