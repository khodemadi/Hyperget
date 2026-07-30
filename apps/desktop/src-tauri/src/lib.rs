use hyper_core::{AddDownloadRequest, DownloadFilter, DownloadId, DownloadManager, DownloadService};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;
struct State(DownloadManager);
struct Inbox(std::path::PathBuf);
type Cmd<T> = std::result::Result<T, String>;
fn err(e: impl ToString) -> String {
    e.to_string()
}
#[tauri::command]
async fn add_download(s: tauri::State<'_, State>, request: AddDownloadRequest) -> Cmd<DownloadId> {
    s.0.add(request).await.map_err(err)
}
#[tauri::command]
async fn list_downloads(s: tauri::State<'_, State>) -> Cmd<Vec<hyper_core::DownloadSnapshot>> {
    s.0.list(DownloadFilter::default()).await.map_err(err)
}
#[tauri::command]
async fn get_download(s: tauri::State<'_, State>, id: DownloadId) -> Cmd<hyper_core::DownloadSnapshot> {
    s.0.get(id).await.map_err(err)
}
macro_rules! action {
    ($name:ident,$method:ident) => {
        #[tauri::command]
        async fn $name(s: tauri::State<'_, State>, id: DownloadId) -> Cmd<()> {
            s.0.$method(id).await.map_err(err)
        }
    };
}
action!(start_download, start);
action!(pause_download, pause);
action!(resume_download, resume);
action!(cancel_download, cancel);
#[tauri::command]
async fn remove_download(s: tauri::State<'_, State>, id: DownloadId, delete_data: bool) -> Cmd<()> {
    s.0.remove(id, delete_data).await.map_err(err)
}
#[tauri::command]
async fn clear_downloads(s: tauri::State<'_, State>) -> Cmd<u64> {
    s.0.pause_all().await.map_err(err)?;
    let downloads = s.0.list(DownloadFilter::default()).await.map_err(err)?;
    let count = downloads.len() as u64;
    for download in downloads {
        s.0.remove(download.id, false).await.map_err(err)?;
    }
    Ok(count)
}
#[tauri::command]
async fn start_all(s: tauri::State<'_, State>) -> Cmd<()> {
    s.0.start_all().await.map_err(err)
}
#[tauri::command]
async fn pause_all(s: tauri::State<'_, State>) -> Cmd<()> {
    s.0.pause_all().await.map_err(err)
}
#[tauri::command]
async fn get_global_status(s: tauri::State<'_, State>) -> Cmd<hyper_core::GlobalStatus> {
    s.0.global_status().await.map_err(err)
}
#[tauri::command]
async fn reorder_downloads(s: tauri::State<'_, State>, ids: Vec<DownloadId>) -> Cmd<()> {
    s.0.reorder(ids).await.map_err(err)
}
#[tauri::command]
async fn set_download_priority(
    s: tauri::State<'_, State>,
    id: DownloadId,
    priority: hyper_core::Priority,
) -> Cmd<()> {
    s.0.set_priority(id, priority).await.map_err(err)
}
#[tauri::command]
async fn move_download_to_top(s: tauri::State<'_, State>, id: DownloadId) -> Cmd<()> {
    let mut ids =
        s.0.list(Default::default())
            .await
            .map_err(err)?
            .into_iter()
            .map(|d| d.id)
            .collect::<Vec<_>>();
    ids.retain(|x| *x != id);
    ids.insert(0, id);
    s.0.reorder(ids).await.map_err(err)
}
#[tauri::command]
async fn move_download_to_bottom(s: tauri::State<'_, State>, id: DownloadId) -> Cmd<()> {
    let mut ids =
        s.0.list(Default::default())
            .await
            .map_err(err)?
            .into_iter()
            .map(|d| d.id)
            .collect::<Vec<_>>();
    ids.retain(|x| *x != id);
    ids.push(id);
    s.0.reorder(ids).await.map_err(err)
}
#[tauri::command]
async fn get_settings(s: tauri::State<'_, State>) -> Cmd<hyper_core::Settings> {
    s.0.settings().await.map_err(err)
}
#[tauri::command]
async fn update_settings(s: tauri::State<'_, State>, settings: hyper_core::Settings) -> Cmd<()> {
    s.0.update_settings(settings).await.map_err(err)
}
#[tauri::command]
fn preview_batch_download(request: hyper_core::BatchPreviewRequest) -> Cmd<Vec<String>> {
    hyper_core::expand_wildcards(&request).map_err(err)
}
#[tauri::command]
async fn probe_download_url(url: String) -> Cmd<serde_json::Value> {
    hyper_core::probe_url(&url).await.map_err(err)
}
#[tauri::command]
async fn discover_batch_download(pattern: String, padding: usize, maximum: usize) -> Cmd<Vec<String>> {
    hyper_core::discover_wildcard_urls(&pattern, padding, maximum)
        .await
        .map_err(err)
}
#[tauri::command]
async fn add_batch_downloads(
    s: tauri::State<'_, State>,
    urls: Vec<String>,
    connections: u8,
    start_immediately: bool,
    destination: Option<std::path::PathBuf>,
) -> Cmd<Vec<DownloadId>> {
    if urls.len() > 10_000 {
        return Err("batch exceeds 10,000 URLs".into());
    }
    let mut ids = Vec::with_capacity(urls.len());
    for url in urls {
        ids.push(
            s.0.add(AddDownloadRequest {
                url,
                connections,
                start_immediately,
                output: None,
                destination_directory: destination.clone(),
                checksum_sha256: None,
            })
            .await
            .map_err(err)?,
        );
    }
    Ok(ids)
}
#[tauri::command]
fn get_system_download_directory(app: tauri::AppHandle) -> Cmd<String> {
    app.path()
        .download_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(err)
}
#[tauri::command]
fn validate_download_directory(path: std::path::PathBuf) -> Cmd<()> {
    if !path.is_dir() {
        return Err("selected path is not a directory".into());
    }
    let probe = path.join(format!(".hyper-get-write-test-{}", uuid::Uuid::new_v4()));
    std::fs::write(&probe, b"").map_err(|e| format!("directory is not writable: {e}"))?;
    std::fs::remove_file(probe).map_err(err)
}
#[tauri::command]
async fn choose_download_directory(app: tauri::AppHandle) -> Cmd<Option<String>> {
    let (sender, receiver) = tokio::sync::oneshot::channel();
    app.dialog().file().pick_folder(move |path| {
        let _ = sender.send(path.map(|p| p.to_string()));
    });
    receiver.await.map_err(err)
}
#[tauri::command]
fn open_logs_folder(app: tauri::AppHandle) -> Cmd<()> {
    let path = app.path().app_log_dir().map_err(err)?;
    std::fs::create_dir_all(&path).map_err(err)?;
    #[cfg(target_os = "windows")]
    let mut command = std::process::Command::new("explorer");
    #[cfg(target_os = "macos")]
    let mut command = std::process::Command::new("open");
    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = std::process::Command::new("xdg-open");
    command.arg(path).spawn().map(|_| ()).map_err(err)
}
#[tauri::command]
fn receive_browser_links(inbox: tauri::State<'_, Inbox>) -> Cmd<Vec<serde_json::Value>> {
    let mut messages = Vec::new();
    std::fs::create_dir_all(&inbox.0).map_err(err)?;
    for entry in std::fs::read_dir(&inbox.0).map_err(err)? {
        let path = entry.map_err(err)?.path();
        if path.extension().and_then(|v| v.to_str()) != Some("json") {
            continue;
        }
        let data = std::fs::read(&path).map_err(err)?;
        if data.len() > 1024 * 1024 {
            continue;
        }
        messages.push(serde_json::from_slice(&data).map_err(err)?);
        std::fs::remove_file(path).map_err(err)?;
    }
    Ok(messages)
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            let downloads = app
                .path()
                .download_dir()
                .unwrap_or_else(|_| dir.join("downloads"));
            app.manage(State(
                DownloadManager::open(dir.join("hyper-get.sqlite3"), downloads)
                    .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?,
            ));
            app.manage(Inbox(dir.join("browser-inbox")));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            add_download,
            list_downloads,
            get_download,
            start_download,
            pause_download,
            resume_download,
            cancel_download,
            remove_download,
            clear_downloads,
            start_all,
            pause_all,
            get_global_status,
            reorder_downloads,
            set_download_priority,
            move_download_to_top,
            move_download_to_bottom,
            get_settings,
            update_settings,
            preview_batch_download,
            probe_download_url,
            discover_batch_download,
            add_batch_downloads,
            receive_browser_links,
            get_system_download_directory,
            validate_download_directory,
            choose_download_directory,
            open_logs_folder
        ])
        .run(tauri::generate_context!())
        .expect("Tauri runtime failed")
}
