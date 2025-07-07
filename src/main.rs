use std::{
    io::{BufRead, Write},
    path::PathBuf,
    sync::Arc,
};

use anyhow::Error as E;
use mediawiki::MediaWiki;
use search_db::SearchDb;
use tracing::info;

mod embedder;
mod mediawiki;
mod search_db;
mod wikitext;

#[tokio::main]
async fn main() -> Result<(), E> {
    // tracing_subscriber::fmt()
    //     .with_span_events(FmtSpan::NEW)
    //     .init();

    let search_db = SearchDb::new();
    let search_db = Arc::new(search_db);
    if !search_db.has_wikitext().await? && false {
        info!("Wikitext collection not found, recreating the database");
        search_db.delete_wikitext_collection().await?;
        search_db.create_wikitext_collection().await?;
        build_index(search_db.clone()).await?;
    }

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut input_iter = stdin.lock().lines();

    print!("Search: ");
    stdout.lock().flush()?;
    while let Some(Ok(ref line)) = input_iter.next() {
        if line == "exit" {
            println!("Goodbye!");
            return Ok(());
        }

        let response = search_db.search(line).await?;
        println!("{}\n", response);

        stdout.lock().flush()?;
        print!("Search: ");
        stdout.lock().flush()?;
    }

    Ok(())
}

/// TODO:
/// This might spawn too many threads on tokios default threadpool.
/// Consider using a library like rayon, and do a bulk embedding
async fn build_index(search_db: Arc<SearchDb>) -> Result<(), E> {
    let mut files = tokio::fs::read_dir("resources/articles").await?;

    let mut handles = Vec::new();
    while let Ok(Some(file)) = files.next_entry().await {
        let search_db_clone = search_db.clone();
        handles.push(tokio::task::spawn_blocking(move || {
            index_vector(search_db_clone, file.path())
        }));
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
