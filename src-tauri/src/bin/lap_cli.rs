use rusqlite::Connection;
use serde_json::json;
use std::env;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: lap-cli <search|ocr> [args...]");
        std::process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "search" => {
            if args.len() < 3 {
                eprintln!("Usage: lap-cli search <query>");
                std::process::exit(1);
            }
            let query = &args[2];
            search_db(query);
        }
        "ocr" => {
            if args.len() < 3 {
                eprintln!("Usage: lap-cli ocr <path>");
                std::process::exit(1);
            }
            let path = &args[2];
            run_ocr(path);
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            std::process::exit(1);
        }
    }
}

fn get_app_data_dir() -> Result<PathBuf, String> {
    let identifier = "com.julyx10.lap";
    let app_dir_name = if cfg!(debug_assertions) {
        format!("{}.debug", identifier)
    } else {
        identifier.to_string()
    };
    dirs::data_local_dir()
        .ok_or_else(|| "Failed to get local AppData directory".to_string())
        .map(|p| p.join(app_dir_name))
}

fn get_current_db_path() -> Result<String, String> {
    let config_path = get_app_data_dir()?.join("app-config.json");
    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read app-config.json: {}", e))?;
    let config: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse app-config.json: {}", e))?;
    let current_library_id = config
        .get("currentLibraryId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "currentLibraryId not found".to_string())?;

    let db_dir = if let Some(dir) = config.get("dbStorageDir").and_then(|v| v.as_str()) {
        PathBuf::from(dir)
    } else {
        get_app_data_dir()?.join("libraries")
    };
    Ok(db_dir
        .join(format!("{}.db", current_library_id))
        .to_string_lossy()
        .into_owned())
}

fn search_db(query: &str) {
    let db_path = match get_current_db_path() {
        Ok(p) => p,
        Err(e) => {
            let res = json!({ "error": e });
            println!("{}", res.to_string());
            return;
        }
    };

    let conn = match Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => {
            let res = json!({ "error": format!("Failed to open DB: {}", e) });
            println!("{}", res.to_string());
            return;
        }
    };

    let sql = "
        SELECT id, name, path, ocr_text 
        FROM afiles 
        WHERE name LIKE ?1 OR ocr_text LIKE ?1 
        LIMIT 50
    ";

    let mut stmt = match conn.prepare(sql) {
        Ok(s) => s,
        Err(e) => {
            let res = json!({ "error": format!("Failed to prepare statement: {}", e) });
            println!("{}", res.to_string());
            return;
        }
    };

    let search_term = format!("%{}%", query);

    let rows = match stmt.query_map([&search_term], |row| {
        Ok(json!({
            "id": row.get::<_, i64>(0)?,
            "name": row.get::<_, String>(1)?,
            "path": row.get::<_, Option<String>>(2)?,
            "ocr_text": row.get::<_, Option<String>>(3)?,
        }))
    }) {
        Ok(r) => r,
        Err(e) => {
            let res = json!({ "error": format!("Failed to execute query: {}", e) });
            println!("{}", res.to_string());
            return;
        }
    };

    let mut results = Vec::new();
    for row in rows {
        if let Ok(r) = row {
            results.push(r);
        }
    }

    println!("{}", serde_json::to_string(&results).unwrap());
}

fn run_ocr(path: &str) {
    match lap_casa::t_advanced_ai::run_ocr_pipeline(path, None) {
        Ok(text) => {
            let res = json!({
                "status": "success",
                "text": text
            });
            println!("{}", res.to_string());
        }
        Err(e) => {
            let res = json!({
                "status": "error",
                "message": e
            });
            println!("{}", res.to_string());
        }
    }
}
