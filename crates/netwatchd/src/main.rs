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
    discovery::NetworkScanner,
    error::NetWatchError,
    storage::StorageManager,
    storage_mount::{LinuxMountProvider, MountProvider},
    traffic::{Collector, LocalCollector},
};
use tokio::sync::broadcast;

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

    // تهيئة الجداول مرة واحدة عند بدء التشغيل
    let db_manager = DatabaseManager::initialize(storage.path())?;

    {
        let conn = db_manager.get_connection()?;
        DeviceRepository::mark_all_offline(&conn)
            .map_err(|err| NetWatchError::Database(err.into()))?;

        if let Ok(deleted) =
            netwatch_core::database::accounting::AccountingRepository::cleanup_old_records(
                &conn, 60,
            )
        {
            println!(
                "🧹 Database Cleanup: Removed {} old usage records.",
                deleted
            );
        }
    }

    let (live_tx, _) = broadcast::channel::<String>(16);

    let storage_path = storage.path().to_path_buf();
    let storage_path_scan = storage.path().to_path_buf();
    let bg_live_tx = live_tx.clone();

    // 1. محرك المحاسبة
    thread::spawn(move || {
        let mut conn = DatabaseManager::connect(&storage_path).unwrap();
        let collector = LocalCollector;
        let mut engine = AccountingEngine::new();

        println!("--- Background Accounting Engine Started ---");
        loop {
            if let Ok(traffics) = collector.collect() {
                if let Ok(live_bw) = engine.process_poll(&mut conn, &traffics) {
                    for (mac, bw) in &live_bw {
                        if bw.rx_bps > 0 || bw.tx_bps > 0 {
                            let dl_mbps = (bw.rx_bps as f64 * 8.0) / 1_000_000.0;
                            let ul_mbps = (bw.tx_bps as f64 * 8.0) / 1_000_000.0;

                            let device_name = match DeviceRepository::get_by_mac(&conn, mac) {
                                Ok(dev) => dev
                                    .display_name
                                    .or(dev.hostname)
                                    .or(dev.vendor)
                                    .unwrap_or_else(|| mac.clone()),
                                Err(_) => mac.clone(),
                            };

                            println!(
                                "Live Bandwidth [{}]: DL: {:.2} Mbps, UL: {:.2} Mbps",
                                device_name, dl_mbps, ul_mbps
                            );
                        }
                    }

                    if let Ok(json_str) = serde_json::to_string(&live_bw) {
                        let _ = bg_live_tx.send(json_str);
                    }
                }
            }
            thread::sleep(Duration::from_secs(2));
        }
    });

    // 2. محرك مسح الأجهزة (The Network Scanner)
    tokio::spawn(async move {
        loop {
            let discovered_devices = NetworkScanner::run_scan().await;

            if !discovered_devices.is_empty() {
                if let Ok(conn) = DatabaseManager::connect(&storage_path_scan) {
                    let _ = DeviceRepository::mark_all_offline(&conn);

                    for dev in discovered_devices {
                        if let Err(e) = DeviceRepository::upsert(
                            &conn,
                            &dev.mac,
                            Some(&dev.ip),
                            None,
                            Some(&dev.vendor),
                        ) {
                            eprintln!("Failed to save discovered device {}: {}", dev.mac, e);
                        } else {
                            println!(
                                "✅ Discovered: IP: {:<15} MAC: {:<20} Vendor: {}",
                                dev.ip, dev.mac, dev.vendor
                            );
                        }
                    }
                }
            }

            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    });

    // 3. خادم الـ API
    let api_storage_path = storage.path().to_path_buf();
    if let Err(e) = start_api_server(api_storage_path, live_tx).await {
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
