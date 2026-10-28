pub mod schema;
pub mod operations;
pub mod score_matching;
#[cfg(test)]
mod tests;

use tauri::AppHandle;
use tauri::Manager;

pub fn init_database(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let app_dir = crate::settings::data_directory(app);
    std::fs::create_dir_all(&app_dir)?;

    // 创建 Playlists 目录
    let playlists_dir = app_dir.join("Playlists");
    std::fs::create_dir_all(&playlists_dir)?;

    let db_path = app_dir.join("bsr_monitor.db");
    let db = rusqlite::Connection::open(&db_path)?;

    // 启用外键约束
    db.execute_batch("PRAGMA foreign_keys = ON;")?;

    if schema::needs_score_migration(&db)? {
        // A consistent backup, including any committed WAL contents. Never
        // overwrite a previous backup when retrying an interrupted migration.
        let mut backup = app_dir.join("bsr_monitor.before-score-normalization.db");
        let mut suffix = 1;
        while backup.exists() {
            backup = app_dir.join(format!("bsr_monitor.before-score-normalization-{suffix}.db"));
            suffix += 1;
        }
        db.execute("VACUUM INTO ?1", [backup.to_string_lossy().as_ref()])?;
    }
    if schema::create_tables(&db)? {
        // Reclaim the old table/index pages after the migration commits.
        db.execute_batch("VACUUM;")?;
    }

    // Store the connection in app state
    app.manage(std::sync::Mutex::new(db));

    println!("Database initialized at: {}", db_path.display());
    println!("Playlists directory: {}", playlists_dir.display());
    Ok(())
}
