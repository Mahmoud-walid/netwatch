use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser, Default, PartialEq, Eq)]
#[command(
    name = "netwatch",
    version,
    about = "NetWatch - Linux Network Monitoring"
)]
pub struct CliOptions {
    #[arg(long, help = "Initialize or override the storage path")]
    pub storage_path: Option<PathBuf>,
}

impl CliOptions {
    // تغليف دالة التحليل حتى لا تحتاج الحزم الأخرى لاستيراد clap::Parser
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_storage_path() {
        let options =
            CliOptions::parse_from(["netwatch", "--storage-path", "/mnt/stor/netwatch-data"]);

        assert_eq!(
            options,
            CliOptions {
                storage_path: Some(PathBuf::from("/mnt/stor/netwatch-data")),
            }
        );
    }

    #[test]
    fn returns_empty_options_without_arguments() {
        let options = CliOptions::parse_from(["netwatch"]);
        assert_eq!(options, CliOptions::default());
    }
}
