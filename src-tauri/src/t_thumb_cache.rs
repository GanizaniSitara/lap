use redb::{Database, TableDefinition};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

const THUMBNAILS_TABLE: TableDefinition<i64, &[u8]> = TableDefinition::new("thumbnails");

static DB_INSTANCE: OnceLock<Arc<Database>> = OnceLock::new();
static WRITE_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

fn get_db_path() -> Result<PathBuf, String> {
    let app_dir = crate::t_config::get_app_data_dir()?;
    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    }
    Ok(app_dir.join("thumbs.redb"))
}

fn get_db() -> Result<Arc<Database>, String> {
    if let Some(db) = DB_INSTANCE.get() {
        return Ok(Arc::clone(db));
    }

    let path = get_db_path()?;
    let db = Database::create(path).map_err(|e| format!("Failed to create redb database: {}", e))?;

    // Ensure the thumbnails table exists
    {
        let write_tx = db
            .begin_write()
            .map_err(|e| format!("Failed to begin write tx: {}", e))?;
        {
            let _ = write_tx
                .open_table(THUMBNAILS_TABLE)
                .map_err(|e| format!("Failed to open thumbnails table: {}", e))?;
        }
        write_tx
            .commit()
            .map_err(|e| format!("Failed to commit table creation: {}", e))?;
    }

    let arc = Arc::new(db);
    let _ = DB_INSTANCE.set(Arc::clone(&arc));
    Ok(arc)
}

pub fn put(file_id: i64, data: &[u8]) -> Result<(), String> {
    if file_id <= 0 || data.is_empty() {
        return Ok(());
    }
    let db = get_db()?;
    let write_mutex = WRITE_MUTEX.get_or_init(|| Mutex::new(()));
    let _guard = write_mutex
        .lock()
        .map_err(|_| "Poisoned redb write lock".to_string())?;
    let write_tx = db
        .begin_write()
        .map_err(|e| format!("Failed to begin write tx: {}", e))?;
    {
        let mut table = write_tx
            .open_table(THUMBNAILS_TABLE)
            .map_err(|e| format!("Failed to open thumbnails table: {}", e))?;
        table
            .insert(file_id, data)
            .map_err(|e| format!("Failed to insert into thumbnails table: {}", e))?;
    }
    write_tx
        .commit()
        .map_err(|e| format!("Failed to commit write tx: {}", e))?;
    Ok(())
}

pub fn get(file_id: i64) -> Option<Vec<u8>> {
    if file_id <= 0 {
        return None;
    }
    let db = get_db().ok()?;
    let read_tx = db.begin_read().ok()?;
    let table = read_tx.open_table(THUMBNAILS_TABLE).ok()?;
    let val = table.get(file_id).ok()??;
    Some(val.value().to_vec())
}

pub fn remove(file_id: i64) -> Result<(), String> {
    if file_id <= 0 {
        return Ok(());
    }
    let db = get_db()?;
    let write_mutex = WRITE_MUTEX.get_or_init(|| Mutex::new(()));
    let _guard = write_mutex
        .lock()
        .map_err(|_| "Poisoned redb write lock".to_string())?;
    let write_tx = db
        .begin_write()
        .map_err(|e| format!("Failed to begin write tx: {}", e))?;
    {
        let mut table = write_tx
            .open_table(THUMBNAILS_TABLE)
            .map_err(|e| format!("Failed to open thumbnails table: {}", e))?;
        let _ = table.remove(file_id);
    }
    write_tx
        .commit()
        .map_err(|e| format!("Failed to commit write tx: {}", e))?;
    Ok(())
}
