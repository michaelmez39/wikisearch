use anyhow::Error as E;
use search_db::SearchDb;

mod embedder;
mod search_db;
mod wiki_parser;

#[tokio::main]
async fn main() -> Result<(), E> {
    let messages = vec![
        "Roses are red, violets are blue",
        "Calculus is the study of rates of change",
        "It is hard to define a species, and scientists have used multliple definitions for the word",
        "Four score and seven years ago",
    ];

    let search_db = SearchDb::new();
    search_db.upsert_bulk(&messages).await?;
    let message = search_db.search("roses").await?;
    println!("{message}");

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
        // rt.block_on(search_db.delete_wikitext_collection())?;
        rt.block_on(search_db.create_wikitext_collection())?;
        Ok(())
    }
}
