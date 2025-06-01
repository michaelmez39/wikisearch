#[cfg(test)]
mod test {
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
    // #[ignore = "utility to download articles"]
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
