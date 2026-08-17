use std::process;
use std::thread;
use std::time::Duration;

use netwatch_core::{
    accounting::AccountingEngine,
    api::start_api_server,
    cli::CliOptions,
    config::NetWatchConfig,
    config_file::ConfigFile,
    database::{DatabaseManager, devices::DeviceRepository},
    error::NetWatchError,
    storage::StorageManager,
    storage_mount::{LinuxMountProvider, MountProvider},
    traffic::{Collector, LocalCollector},
};

#[tokio::main]
async fn run_daemon() -> Result<(), NetWatchError> {
    let cli = CliOptions::parse_args();

    let config_path = ConfigFile::default_path()
        .ok_or_else(|| NetWatchError::Storage(netwatch_core::error::StorageError::EmptyPath))?;

    let config = if let Some(cli_storage) = cli.storage_path {
        let mount_provider = LinuxMountProvider;
        let mount_point = mount_provider
            .mount_for_path(&cli_storage)
            .unwrap_or(None)
            .map(|m| m.mount_point);

        NetWatchConfig::new(cli_storage, mount_point)
    } else if config_path.exists() {
        ConfigFile::load(&config_path)?
    } else {
        eprintln!("Error: NetWatch configuration not found.");
        eprintln!("Please run with `--storage-path <path>` to initialize.");
        process::exit(1);
    };

    ConfigFile::save(&config_path, &config)?;

    let mount_provider = LinuxMountProvider;
    let storage = StorageManager::new(&config.storage, &mount_provider)?;

    println!("NetWatch Daemon started successfully.");
    println!("Config Path: {}", config_path.display());
    println!("Storage Path: {}", storage.path().display());

    let db_manager = DatabaseManager::initialize(storage.path())?;

    {
        let conn = db_manager.get_connection()?;
        DeviceRepository::mark_all_offline(&conn)
            .map_err(|err| NetWatchError::Database(err.into()))?;
    }

    let storage_path = storage.path().to_path_buf();
    thread::spawn(move || {
        let bg_db = DatabaseManager::initialize(&storage_path).unwrap();
        let mut conn = bg_db.get_connection().unwrap();
        let collector = LocalCollector;
        let mut engine = AccountingEngine::new();

        println!("--- Background Accounting Engine Started ---");
        loop {
            if let Ok(traffics) = collector.collect() {
                let _ = engine.process_poll(&mut conn, &traffics);
            }
            thread::sleep(Duration::from_secs(2));
        }
    });

    let db_file_path = storage.path().join("netwatch.sqlite");
    if let Err(e) = start_api_server(db_file_path).await {
        eprintln!("API Server Error: {}", e);
    }

    Ok(())
}

fn main() {
    if let Err(e) = run_daemon() {
        eprintln!("Fatal Error: {}", e);
        process::exit(1);
    }
}
