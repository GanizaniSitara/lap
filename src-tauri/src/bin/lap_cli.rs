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
        "ocr-all" => {
            run_ocr_all();
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
        .get("current_library_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "current_library_id not found".to_string())?;

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
        SELECT id, name, folder_id, ocr_text 
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
            "folder_id": row.get::<_, Option<i64>>(2)?,
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

fn run_ocr_all() {
    let db_path = match get_current_db_path() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to get DB path: {}", e);
            return;
        }
    };

    let conn = match Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to open DB: {}", e);
            return;
        }
    };

    let models_dir = std::path::PathBuf::from("C:\\git\\lap\\models");
    let det_path = models_dir.join("ocr_det.onnx");
    let cls_path = models_dir.join("ocr_cls.onnx");
    let rec_path = models_dir.join("ocr_rec.onnx");

    let mut det_session = ort::session::Session::builder()
        .unwrap()
        .with_execution_providers([ort::execution_providers::CUDAExecutionProvider::default().build()])
        .unwrap()
        .commit_from_file(det_path)
        .unwrap();
        
    let _cls_session = ort::session::Session::builder()
        .unwrap()
        .with_execution_providers([ort::execution_providers::CUDAExecutionProvider::default().build()])
        .unwrap()
        .commit_from_file(cls_path)
        .unwrap();
        
    let mut rec_session = ort::session::Session::builder()
        .unwrap()
        .with_execution_providers([ort::execution_providers::CUDAExecutionProvider::default().build()])
        .unwrap()
        .commit_from_file(rec_path)
        .unwrap();

    let vocab_chars: Vec<char> = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!\"#$%&'()*+,-./:;<=>?@[\\]_`~ ".chars().collect();

    let query = "
        SELECT a.id, f.path, a.name 
        FROM afiles a 
        JOIN afolders f ON a.folder_id = f.id 
        WHERE a.ocr_text IS NULL AND a.file_type IN (1,2,3)
    ";

    let mut stmt = conn.prepare(query).unwrap();
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    }).unwrap();

    let files: Vec<(i64, String, String)> = rows.filter_map(Result::ok).collect();
    println!("Found {} files to process", files.len());

    use image::GenericImageView;
    use ndarray::Array4;

    for (i, (id, path, name)) in files.iter().enumerate() {
        let full_path = format!("{}\\{}", path, name);
        println!("[{}/{}] Processing: {}", i + 1, files.len(), full_path);

        let img = match image::open(&full_path) {
            Ok(i) => i,
            Err(_) => {
                let _ = conn.execute("UPDATE afiles SET ocr_text = ?1 WHERE id = ?2", rusqlite::params!["", id]);
                continue;
            }
        };

        let (orig_w, orig_h) = img.dimensions();
        if orig_w == 0 || orig_h == 0 { 
            let _ = conn.execute("UPDATE afiles SET ocr_text = ?1 WHERE id = ?2", rusqlite::params!["", id]);
            continue; 
        }

        let new_w = (orig_w / 32).max(1) * 32;
        let new_h = (orig_h / 32).max(1) * 32;

        let resized_det = img.resize_exact(new_w, new_h, image::imageops::FilterType::Triangle).to_rgb8();
        
        let mut det_tensor = Array4::<f32>::zeros((1, 3, new_h as usize, new_w as usize));
        for (x, y, pixel) in resized_det.enumerate_pixels() {
            let r = (pixel[0] as f32 / 255.0 - 0.485) / 0.229;
            let g = (pixel[1] as f32 / 255.0 - 0.456) / 0.224;
            let b = (pixel[2] as f32 / 255.0 - 0.406) / 0.225;
            det_tensor[[0, 0, y as usize, x as usize]] = b;
            det_tensor[[0, 1, y as usize, x as usize]] = g;
            det_tensor[[0, 2, y as usize, x as usize]] = r;
        }

        let det_value = match ort::value::Value::from_array(det_tensor) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let det_outputs = match det_session.run(ort::inputs![det_value]) {
            Ok(o) => o,
            Err(_) => continue,
        };
        
        let (_, det_slice) = match det_outputs[0].try_extract_tensor::<f32>() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let w_usize = new_w as usize;
        let h_usize = new_h as usize;
        let mut binary_map = vec![false; w_usize * h_usize];
        for (idx, &val) in det_slice.iter().enumerate() {
            if val > 0.3 {
                binary_map[idx] = true;
            }
        }

        let mut boxes = Vec::new();
        let mut visited = vec![false; w_usize * h_usize];

        for y in 0..h_usize {
            for x in 0..w_usize {
                let idx = y * w_usize + x;
                if binary_map[idx] && !visited[idx] {
                    let mut q = vec![(x, y)];
                    visited[idx] = true;
                    let mut min_x = x;
                    let mut max_x = x;
                    let mut min_y = y;
                    let mut max_y = y;
                    let mut head = 0;
                    while head < q.len() {
                        let (cx, cy) = q[head];
                        head += 1;
                        min_x = min_x.min(cx);
                        max_x = max_x.max(cx);
                        min_y = min_y.min(cy);
                        max_y = max_y.max(cy);
                        
                        for &(dx, dy) in &[(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (1, 1), (-1, 1), (1, -1)] {
                            let nx = cx as i32 + dx;
                            let ny = cy as i32 + dy;
                            if nx >= 0 && nx < w_usize as i32 && ny >= 0 && ny < h_usize as i32 {
                                let n_idx = (ny as usize) * w_usize + (nx as usize);
                                if binary_map[n_idx] && !visited[n_idx] {
                                    visited[n_idx] = true;
                                    q.push((nx as usize, ny as usize));
                                }
                            }
                        }
                    }
                    if max_x - min_x > 2 && max_y - min_y > 2 {
                        boxes.push((min_x, min_y, max_x, max_y));
                    }
                }
            }
        }

        let mut extracted_texts = Vec::new();

        boxes.sort_by(|a, b| {
            let y_diff = (a.1 as i32 - b.1 as i32).abs();
            if y_diff < 15 { 
                a.0.cmp(&b.0)
            } else {
                a.1.cmp(&b.1)
            }
        });

        for (min_x, min_y, max_x, max_y) in boxes {
            let orig_min_x = (min_x as f32 / new_w as f32 * orig_w as f32) as u32;
            let orig_max_x = (max_x as f32 / new_w as f32 * orig_w as f32) as u32;
            let orig_min_y = (min_y as f32 / new_h as f32 * orig_h as f32) as u32;
            let orig_max_y = (max_y as f32 / new_h as f32 * orig_h as f32) as u32;

            let crop_w = (orig_max_x.saturating_sub(orig_min_x)).max(1);
            let crop_h = (orig_max_y.saturating_sub(orig_min_y)).max(1);

            let crop = img.crop_imm(orig_min_x, orig_min_y, crop_w, crop_h);
            
            let mut rec_w = (48.0 * (crop_w as f32 / crop_h as f32)) as u32;
            rec_w = rec_w.max(1);
            let resized_crop = crop.resize_exact(rec_w, 48, image::imageops::FilterType::Triangle).to_rgb8();
            
            let mut rec_tensor = Array4::<f32>::zeros((1, 3, 48, rec_w as usize));
            for (x, y, pixel) in resized_crop.enumerate_pixels() {
                let r = (pixel[0] as f32 / 255.0 - 0.5) / 0.5;
                let g = (pixel[1] as f32 / 255.0 - 0.5) / 0.5;
                let b = (pixel[2] as f32 / 255.0 - 0.5) / 0.5;
                rec_tensor[[0, 0, y as usize, x as usize]] = b;
                rec_tensor[[0, 1, y as usize, x as usize]] = g;
                rec_tensor[[0, 2, y as usize, x as usize]] = r;
            }

            let rec_value = match ort::value::Value::from_array(rec_tensor) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let rec_outputs = match rec_session.run(ort::inputs![rec_value]) {
                Ok(o) => o,
                Err(_) => continue,
            };
            
            let (rec_shape, rec_slice) = match rec_outputs[0].try_extract_tensor::<f32>() {
                Ok(m) => m,
                Err(_) => continue,
            };
            
            let seq_len = rec_shape[1] as usize;
            let num_classes = rec_shape[2] as usize;
            
            let mut text = String::new();
            let mut last_idx = 999;
            
            for t in 0..seq_len {
                let mut max_prob = -1.0;
                let mut max_idx = 0;
                for c in 0..num_classes {
                    let prob = rec_slice[t * num_classes + c];
                    if prob > max_prob {
                        max_prob = prob;
                        max_idx = c;
                    }
                }
                
                if max_idx != 96 && max_idx != last_idx {
                    if max_idx < vocab_chars.len() {
                        text.push(vocab_chars[max_idx]);
                    }
                }
                last_idx = max_idx;
            }
            if !text.is_empty() {
                extracted_texts.push(text);
            }
        }

        let final_text = extracted_texts.join("\n");
        let _ = conn.execute(
            "UPDATE afiles SET ocr_text = ?1 WHERE id = ?2",
            rusqlite::params![final_text, id],
        );
    }
}
