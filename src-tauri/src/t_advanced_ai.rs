use crate::t_sqlite;
use image::{DynamicImage, GenericImageView};
use ort::{GraphOptimizationLevel, Session};
use rusqlite::params;
use std::thread;
use std::time::Duration;
use std::path::Path;

pub fn start_worker() {
    thread::spawn(|| {
        // low-priority background thread
        loop {
            thread::sleep(Duration::from_secs(60));
            if let Err(e) = process_pending_files() {
                eprintln!("Error in advanced AI worker: {}", e);
            }
        }
    });
}

fn process_pending_files() -> Result<(), String> {
    let mut conn = t_sqlite::open_conn()?;
    
    // Find afiles needing processing, strictly bypassing any file whose parent folder is marked ai_excluded = 1
    let query = "
        SELECT a.id, b.path, a.name, a.file_type
        FROM afiles a
        JOIN afolders b ON a.folder_id = b.id
        WHERE (a.ocr_text IS NULL OR a.ai_tags IS NULL)
          AND COALESCE(b.ai_excluded, 0) = 0
          AND a.file_type IN (1, 2, 3)
        LIMIT 50
    ";

    let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;
    
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
    }).map_err(|e| e.to_string())?;

    let files: Vec<(i64, String, String)> = rows.filter_map(Result::ok).collect();
    
    for (id, folder_path, name) in files {
        let path = Path::new(&folder_path).join(&name);
        let img_result = image::open(&path);
        match img_result {
            Ok(img) => {
                let ocr_text = extract_ocr_text(&img);
                let ai_tags = detect_objects(&img);
                
                let _ = conn.execute(
                    "UPDATE afiles SET ocr_text = ?1, ai_tags = ?2 WHERE id = ?3",
                    params![ocr_text, ai_tags, id]
                );
            },
            Err(_) => {
                let _ = conn.execute(
                    "UPDATE afiles SET ocr_text = '', ai_tags = '' WHERE id = ?1",
                    params![id]
                );
            }
        }
    }

    Ok(())
}

fn extract_ocr_text(_img: &DynamicImage) -> String {
    // Scaffolding for ONNX OCR model
    let _session = Session::builder()
        .unwrap()
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .unwrap();
        // .commit_from_file("ocr.onnx")
        // .unwrap();
    "placeholder_ocr_text".to_string()
}

fn detect_objects(_img: &DynamicImage) -> String {
    // Scaffolding for ONNX YOLO model
    let _session = Session::builder()
        .unwrap()
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .unwrap();
        // .commit_from_file("yolo.onnx")
        // .unwrap();
    "placeholder_ai_tags".to_string()
}
