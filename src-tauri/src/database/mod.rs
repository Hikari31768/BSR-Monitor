pub mod schema;
pub mod operations;

use tauri::AppHandle;
use tauri::Manager;

pub fn init_database(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let app_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&app_dir)?;

    let db_path = app_dir.join("bsr_monitor.db");
    let db = rusqlite::Connection::open(&db_path)?;

    // 启用外键约束
    db.execute_batch("PRAGMA foreign_keys = ON;")?;

    schema::create_tables(&db)?;

    // Store the connection in app state
    app.manage(std::sync::Mutex::new(db));

    println!("Database initialized at: {}", db_path.display());
    Ok(())
}