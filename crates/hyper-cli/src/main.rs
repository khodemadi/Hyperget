use clap::{Parser, Subcommand};
use hyper_core::{AddDownloadRequest, DownloadFilter, DownloadId, DownloadManager, DownloadService};
use std::path::PathBuf;
#[derive(Parser)]
#[command(version, about = "Persistent, resumable download manager")]
struct Args {
    #[arg(long, global = true)]
    json: bool,
    #[arg(long, global = true, env = "HYPER_GET_DATA_DIR")]
    data_dir: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Add {
        url: String,
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[arg(short, long, default_value_t = 4)]
        connections: u8,
        #[arg(long)]
        no_start: bool,
    },
    List,
    Status,
    Start {
        id: DownloadId,
    },
    Pause {
        id: DownloadId,
    },
    Resume {
        id: DownloadId,
    },
    Cancel {
        id: DownloadId,
    },
    Remove {
        id: DownloadId,
        #[arg(long)]
        delete_data: bool,
    },
    StartAll,
    PauseAll,
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let a = Args::parse();
    let base = a.data_dir.unwrap_or_else(default_data);
    let manager = DownloadManager::open(base.join("hyper-get.sqlite3"), base.join("downloads"))?;
    match a.command {
        Command::Add {
            url,
            output,
            connections,
            no_start,
        } => {
            let id = manager
                .add(AddDownloadRequest {
                    url,
                    output,
                    destination_directory: None,
                    connections,
                    start_immediately: !no_start,
                    checksum_sha256: None,
                })
                .await?;
            if a.json {
                println!("{}", serde_json::json!({"id":id}))
            } else {
                println!("Added {id}")
            }
        }
        Command::List => {
            let x = manager.list(DownloadFilter::default()).await?;
            if a.json {
                println!("{}", serde_json::to_string_pretty(&x)?)
            } else {
                for d in x {
                    println!(
                        "{}  {:<12?} {:>6.1}%  {}",
                        d.id,
                        d.state,
                        d.percentage().unwrap_or(0.0),
                        d.filename
                    )
                }
            }
        }
        Command::Status => {
            let x = manager.global_status().await?;
            if a.json {
                println!("{}", serde_json::to_string_pretty(&x)?)
            } else {
                println!(
                    "{} active, {} queued, {} paused; {:.1}%",
                    x.active,
                    x.queued,
                    x.paused,
                    x.percentage.unwrap_or(0.0)
                )
            }
        }
        Command::Start { id } => manager.start(id).await?,
        Command::Pause { id } => manager.pause(id).await?,
        Command::Resume { id } => manager.resume(id).await?,
        Command::Cancel { id } => manager.cancel(id).await?,
        Command::Remove { id, delete_data } => manager.remove(id, delete_data).await?,
        Command::StartAll => manager.start_all().await?,
        Command::PauseAll => manager.pause_all().await?,
    }
    Ok(())
}
fn default_data() -> PathBuf {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("APPDATA").map(PathBuf::from))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .join("hyper-get")
}
