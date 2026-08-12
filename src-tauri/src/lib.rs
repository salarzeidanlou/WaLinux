use std::path::Path;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, WebviewWindowBuilder};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_single_instance::init;
use tokio::fs;

// Permission management (Linux always grants permission)
#[tauri::command]
fn check_notification_permission(app: AppHandle) -> Result<String, String> {
    let state = app
        .notification()
        .permission_state()
        .map_err(|e| e.to_string())?;

    Ok(match state {
        tauri_plugin_notification::PermissionState::Granted => "granted".to_string(),
        tauri_plugin_notification::PermissionState::Denied => "denied".to_string(),
        _ => "default".to_string(),
    })
}

#[tauri::command]
fn request_notification_permission(app: AppHandle) -> Result<String, String> {
    let state = app
        .notification()
        .request_permission()
        .map_err(|e| e.to_string())?;

    Ok(match state {
        tauri_plugin_notification::PermissionState::Granted => "granted".to_string(),
        tauri_plugin_notification::PermissionState::Denied => "denied".to_string(),
        _ => "default".to_string(),
    })
}
async fn is_downloaded(full_path_generated: &Path) -> Result<bool, String> {
    let file_exists = fs::metadata(full_path_generated)
        .await
        .map_err(|e| e.to_string());
    match file_exists {
        Ok(metadata) => Ok(metadata.is_file()),
        Err(e) => Err(e),
    }
}
async fn open_file(absolute_path: &Path) -> Result<(), String> {
    open::that(absolute_path).map_err(|e| {
        eprintln!("[WaLinux] Failed to open file: {}", e);
        e.to_string()
    })?;
    Ok(())
}

fn sanitize_download_filename(filename: &str) -> Result<String, String> {
    // Treat both Unix and Windows separators as path separators. The filename
    // originates in remote web content and must never be allowed to escape the
    // application download directory.
    let normalized = filename.trim().replace('\\', "/");
    let name = Path::new(&normalized)
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "Download filename is missing or invalid".to_string())?;

    if name.is_empty() || name == "." || name == ".." {
        return Err("Download filename is missing or invalid".to_string());
    }

    let sanitized: String = name
        .chars()
        .map(|character| {
            if character.is_control()
                || matches!(character, '<' | '>' | ':' | '"' | '|' | '?' | '*')
            {
                '_'
            } else {
                character
            }
        })
        .collect();

    if sanitized.is_empty() {
        Err("Download filename is missing or invalid".to_string())
    } else {
        Ok(sanitized)
    }
}

#[tauri::command]
async fn save_blob(bytes: Vec<u8>, filename: String) -> Result<String, String> {
    let filename = sanitize_download_filename(&filename)?;

    // Determine download base folder: ~/Downloads/WaLinux
    let base_dir = dirs::download_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join("WaLinux");

    println!("Base directory: {}", base_dir.display());
    // println!("Bytes: {:?}", bytes);
    println!("Filename: {}", filename);

    // Detect file type
    let kind = infer::get(&bytes);
    let subfolder = if let Some(kind) = kind {
        match kind.mime_type() {
            t if t.starts_with("image/") => "Images",
            t if t.starts_with("video/") => "Videos",
            t if t == "application/pdf"
                || t.starts_with("text/")
                || t.starts_with("application/ms") =>
            {
                "Documents"
            }
            _ => "Others",
        }
    } else {
        // Fallback if unknown
        "Others"
    };

    // Create folder if it doesn't exist
    let save_dir = base_dir.join(subfolder);
    fs::create_dir_all(&save_dir)
        .await
        .map_err(|e| e.to_string())?;

    // Final file path
    let file_path = save_dir.join(&filename);

    // Check if file already exists
    if is_downloaded(&file_path).await.unwrap_or(false) {
        println!(
            "[WaLinux] File already exists: {}",
            file_path.to_string_lossy()
        );

        // Try to open the file with default application
        if let Err(e) = open_file(&file_path).await {
            eprintln!("[WaLinux] Could not open existing file: {}", e);
            // Continue anyway - file exists but couldn't be opened
        } else {
            println!(
                "[WaLinux] Opened existing file: {}",
                file_path.to_string_lossy()
            );
        }

        return Err(file_path.to_string_lossy().to_string());
    }

    // Write bytes to file
    fs::write(&file_path, &bytes)
        .await
        .map_err(|e| e.to_string())?;
    // Return full path
    Ok(file_path.to_string_lossy().to_string())
}

// Basic notification
#[tauri::command]
fn show_notification(
    app: AppHandle,
    title: String,
    body: String,
    icon: Option<String>,
) -> Result<(), String> {
    let mut builder = app.notification().builder().title(&title).body(&body);

    if let Some(icon_path) = icon {
        builder = builder.icon(&icon_path);
    }

    builder.show().map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // --- Single Instance Plugin ---
        .plugin(init(|app, _args, _cwd| {
            // Called if another instance is launched
            if let Some(win) = app.get_webview_window("main") {
                win.show().unwrap();
                win.unminimize().ok();
                win.set_focus().unwrap();
            }
        }))
        // --- Other Plugins ---
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        // --- Commands ---
        .invoke_handler(tauri::generate_handler![
            check_notification_permission,
            request_notification_permission,
            show_notification,
            save_blob
        ])
        .setup(|app| {
            // Get window config from tauri.conf.json
            let app_config = app.config().clone();
            let window_config = app_config
                .app
                .windows
                .get(0)
                .expect("Window config not found")
                .clone();

            // Create main window manually
            WebviewWindowBuilder::from_config(app, &window_config)?
                .on_download(|_webview_window, mut event| {
                    match &mut event {
                        tauri::webview::DownloadEvent::Requested { url, destination } => {
                            println!("[WaLinux] Download requested: {}", url);

                            // If it's a blob URL, let JavaScript handle it
                            if url.as_str().starts_with("blob:") {
                                println!("[WaLinux] Blocking blob download - letting JS handle it");
                                return false; // Block - let JavaScript intercept
                            }
                            println!("[WaLinux] Not a blob URL - setting custom download path");
                            // For non-blob URLs, set custom download path
                            if let Some(filename) = url
                                .path_segments()
                                .and_then(|segments| {
                                    segments.filter(|segment| !segment.is_empty()).next_back()
                                })
                                .and_then(|name| sanitize_download_filename(name).ok())
                            {
                                let base_dir = dirs::download_dir()
                                    .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
                                    .join("WaLinux")
                                    .join("Others");

                                if let Err(error) = std::fs::create_dir_all(&base_dir) {
                                    eprintln!(
                                        "[WaLinux] Could not create download directory: {error}"
                                    );
                                    return false;
                                }

                                **destination = base_dir.join(filename);
                                println!("[WaLinux] Download will be saved to: {:?}", destination);
                            }
                            true // Allow download with custom path
                        }
                        tauri::webview::DownloadEvent::Finished { url, path, success } => {
                            if *success {
                                if let Some(path) = path {
                                    println!("[WaLinux] Download completed: {:?}", path);
                                }
                            } else {
                                println!("[WaLinux] Download failed: {}", url);
                            }
                            true
                        }
                        _ => true,
                    }
                })
                .build()?;

            // --- Tray Menu ---
            let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            let _tray = TrayIconBuilder::with_id("main-tray")
                .tooltip("WaLinux")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            window.show().unwrap();
                            window.set_focus().unwrap();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            let window = app.get_webview_window("main").unwrap();

            // Close event → hide instead of exit
            let window_clone = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    window_clone.hide().unwrap();
                    api.prevent_close();
                }
            });

            // Inject Notification bridge
            window.eval(include_str!("notification.js")).ok();

            // Inject Download bridge
            window.eval(include_str!("download.js")).ok();

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::sanitize_download_filename;

    #[test]
    fn keeps_a_normal_filename() {
        assert_eq!(
            sanitize_download_filename("photo.jpg").unwrap(),
            "photo.jpg"
        );
    }

    #[test]
    fn removes_parent_directories() {
        assert_eq!(
            sanitize_download_filename("../../private.txt").unwrap(),
            "private.txt"
        );
        assert_eq!(
            sanitize_download_filename("..\\..\\private.txt").unwrap(),
            "private.txt"
        );
    }

    #[test]
    fn rejects_missing_filenames() {
        assert!(sanitize_download_filename("..").is_err());
        assert!(sanitize_download_filename("/").is_err());
    }
}
