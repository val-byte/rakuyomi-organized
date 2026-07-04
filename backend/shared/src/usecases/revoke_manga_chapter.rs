use anyhow::Error;

use crate::{chapter_storage::ChapterStorage, database::Database, model::ChapterId};

pub async fn revoke_manga_chapter(
    database: &Database,
    chapter_storage: &ChapterStorage,
    chapter: &ChapterId,
    use_ram: bool,
) -> Result<bool, Error> {
    let info = database.find_cached_chapter_information(chapter).await?;
    let chapter_number = info.as_ref().and_then(|i| i.chapter_number);
    let volume_number = info.as_ref().and_then(|i| i.volume_number);
    let manga_title = info.as_ref().and_then(|i| i.title.as_deref()).unwrap_or("Unknown");
    let Some(path) = chapter_storage.get_stored_chapter(chapter_number, manga_title, volume_number, chapter, use_ram) else {
        // No chapter stored → nothing removed
        return Ok(false);
    };

    let removed_main = (tokio::fs::remove_file(&path).await).is_ok();

    // Get path to "errors file" (optional) and delete it,
    // but ignore all failures because it's best-effort cleanup.
    if let Ok(path_errors) = chapter_storage.errors_source_path(&path) {
        // fire-and-forget but awaited; failure ignored
        let _ = tokio::fs::remove_file(path_errors).await;
    }

    Ok(removed_main)
}
