pub mod loader;
pub mod settings;

pub use loader::{load_or_create_config, save_config, CONFIG_FILE_NAME};
pub use settings::{VoteSiteConfig, VotifierConfig};
