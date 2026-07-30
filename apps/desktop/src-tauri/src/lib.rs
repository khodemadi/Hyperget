use hyper_core::{AddDownloadRequest, DownloadFilter, DownloadId, DownloadManager, DownloadService};
use tauri::Manager;
struct State(DownloadManager);
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
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
            start_all,
            pause_all,
            get_global_status,
            reorder_downloads,
            set_download_priority,
            move_download_to_top,
            move_download_to_bottom,
            get_settings,
            update_settings
        ])
        .run(tauri::generate_context!())
        .expect("Tauri runtime failed")
}
