use tokio::io::AsyncReadExt;

pub const DEFAULT_SOCKET_NAME: &'static str = "mangadex-fsd.sock";
pub const DEFAULT_CONFIG_NAME: &'static str = "config.toml";

pub fn project_dirs() -> directories::ProjectDirs {
    directories::ProjectDirs::from("", "", "mangadex-fs").unwrap()
}

pub fn config_file_path() -> std::path::PathBuf {
    let project_dirs = project_dirs();
    let config_dir = project_dirs.config_dir();
    config_dir.join(std::path::Path::new(DEFAULT_CONFIG_NAME))
}

pub fn default_socket_path() -> std::path::PathBuf {
    let project_dirs = project_dirs();
    let runtime_dir = project_dirs.runtime_dir().unwrap();
    runtime_dir.join(std::path::Path::new(DEFAULT_SOCKET_NAME))
}

#[derive(serde::Deserialize)]
pub struct Config {
    #[serde(default = "default_socket_path")]
    pub socket: std::path::PathBuf,
    pub mountpoint: Option<std::path::PathBuf>
}

impl std::default::Default for Config {
    fn default() -> Config {
        Config {
            socket: default_socket_path(),
            mountpoint: None
        }
    }
}

impl Config {
    pub async fn load() -> std::io::Result<Result<Config, toml::de::Error>> {
        let config_path = config_file_path();
        let mut file = tokio::fs::File::open(config_path).await?;

        let mut contents = vec![];
        file.read_to_end(&mut contents).await?;

        Ok(toml::from_slice(&contents))
    }
}