use crate::t_sqlite;

use ndarray::Array4;
use rusqlite::params;

use std::thread;
use std::time::Duration;
use tauri::AppHandle;
use tauri::Manager;
use image::GenericImageView;

const VOCAB: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!\"#$%&'()*+,-./:;<=>?@[\\]_`~ ";

pub fn start_ai_worker(app: AppHandle) {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(10));
            if let Err(e) = process_pending_files(&app) {
                eprintln!("Error in advanced AI worker: {}", e);
            }
        }
    });
}

fn process_pending_files(app: &AppHandle) -> Result<(), String> {
    let conn = t_sqlite::open_conn()?;

    let query = "
        SELECT a.id, b.path, a.name, a.file_type
        FROM afiles a
        JOIN afolders b ON a.folder_id = b.id
        WHERE a.ocr_text IS NULL
          AND COALESCE(b.ai_excluded, 0) = 0
          AND a.file_type IN (1, 2, 3)
        LIMIT 1
    ";

    let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let files: Vec<(i64, String, String)> = rows.filter_map(Result::ok).collect();

    if files.is_empty() {
        return Ok(());
    }

    let resource_dir = app.path().resource_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let models_dir = resource_dir.join("models");

    for (id, folder_path, name) in files {
        let full_path = std::path::Path::new(&folder_path).join(&name);
        
        match run_ocr_pipeline(full_path.to_str().unwrap(), Some(models_dir.clone())) {
            Ok(final_text) => {
                let _ = conn.execute(
                    "UPDATE afiles SET ocr_text = ?1 WHERE id = ?2",
                    params![final_text, id],
                );
            }
            Err(_) => {
                let _ = conn.execute(
                    "UPDATE afiles SET ocr_text = ?1 WHERE id = ?2",
                    params!["", id],
                );
            }
        }
    }

    Ok(())
}

pub fn run_ocr_pipeline(image_path: &str, models_dir_opt: Option<std::path::PathBuf>) -> Result<String, String> {
    let models_dir = models_dir_opt.unwrap_or_else(|| {
        if std::path::Path::new("models/ocr_det.onnx").exists() {
            std::path::PathBuf::from("models")
        } else {
            std::env::current_exe()
                .map(|p| p.parent().unwrap().join("models"))
                .unwrap_or_else(|_| std::path::PathBuf::from("models"))
        }
    });

    let det_path = models_dir.join("ocr_det.onnx");
    let cls_path = models_dir.join("ocr_cls.onnx");
    let rec_path = models_dir.join("ocr_rec.onnx");

    let mut det_session = ort::session::Session::builder()
        .map_err(|e| e.to_string())?
        .with_execution_providers([ort::execution_providers::CUDAExecutionProvider::default().build()])
        .map_err(|e| e.to_string())?
        .commit_from_file(det_path)
        .map_err(|e| e.to_string())?;
        
    let _cls_session = ort::session::Session::builder()
        .map_err(|e| e.to_string())?
        .with_execution_providers([ort::execution_providers::CUDAExecutionProvider::default().build()])
        .map_err(|e| e.to_string())?
        .commit_from_file(cls_path)
        .map_err(|e| e.to_string())?;
        
    let mut rec_session = ort::session::Session::builder()
        .map_err(|e| e.to_string())?
        .with_execution_providers([ort::execution_providers::CUDAExecutionProvider::default().build()])
        .map_err(|e| e.to_string())?
        .commit_from_file(rec_path)
        .map_err(|e| e.to_string())?;

    let vocab_chars: Vec<char> = VOCAB.chars().collect();

    let img = image::open(image_path).map_err(|_| "Failed to open image".to_string())?;

    let (orig_w, orig_h) = img.dimensions();
    if orig_w == 0 || orig_h == 0 { 
        return Ok(String::new());
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

    let det_value = ort::value::Value::from_array(det_tensor).map_err(|e| e.to_string())?;
    let det_outputs = det_session.run(ort::inputs![det_value]).map_err(|e| e.to_string())?;
    
    let (_, det_slice) = det_outputs[0].try_extract_tensor::<f32>().map_err(|e| e.to_string())?;

    let w_usize = new_w as usize;
    let h_usize = new_h as usize;
    let mut binary_map = vec![false; w_usize * h_usize];
    for (i, &val) in det_slice.iter().enumerate() {
        if val > 0.3 {
            binary_map[i] = true;
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
        println!("Decoded box: {}", text);
        if !text.is_empty() {
            extracted_texts.push(text);
        }
    }

    Ok(extracted_texts.join("\n"))
}
