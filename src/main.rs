#![feature(iter_map_windows)]
use std::{path::PathBuf, sync::Arc};

use anyhow::Error as E;
use mediawiki::MediaWiki;
use search_db::SearchDb;

mod embedder;
mod mediawiki;
mod search_db;
mod wikitext;

#[tokio::main]
async fn main() -> Result<(), E> {
    let search_db = SearchDb::new();

    // let messages = vec![
    //     "Roses are red, violets are blue",
    //     "Calculus is the study of rates of change",
    //     "It is hard to define a species, and scientists have used multliple definitions for the word",
    //     "Four score and seven years ago",
    // ];

    // search_db.upsert_bulk(&messages).await?;
    search_db.delete_wikitext_collection().await?;
    search_db.create_wikitext_collection().await?;
    let search_db = Arc::new(search_db);
    build_index(search_db.clone()).await?;
    let message = search_db.search("August Borsig").await?;
    println!("{message}");

    Ok(())
}

async fn build_index(search_db: Arc<SearchDb>) -> Result<(), E> {
    let mut files = tokio::fs::read_dir("resources/articles").await?;

    let mut handles = Vec::new();
    while let Ok(Some(file)) = files.next_entry().await {
        handles.push(tokio::spawn(index_vector(search_db.clone(), file.path())));
    }

    for handle in handles {
        let _ = handle.await;
    }

    Ok(())
}

async fn index_vector(search_db: Arc<SearchDb>, path: PathBuf) -> Result<(), E> {
    let contents = tokio::fs::read_to_string(path).await?;
    let mw: MediaWiki = quick_xml::de::from_str(&contents)?;
    let Some(article) = mediawiki::get_article_text(&mw) else {
        return Err(E::msg("missing article"));
    };

    let truncated = &article.as_str()[0..512];
    search_db.upsert(&truncated).await?;
    Ok(())
}

#[cfg(test)]
mod test {

    use crate::search_db::SearchDb;
    use anyhow::Error as E;

    #[test]
    #[ignore = "utility to delete and remake the wikitext collection"]
    fn wipe_wikitext() -> Result<(), E> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let search_db = SearchDb::new();
        rt.block_on(search_db.delete_wikitext_collection())?;
        rt.block_on(search_db.create_wikitext_collection())?;
        Ok(())
    }
}
