use tracing::{info, level_filters::LevelFilter};
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};

pub(crate) type Result<T> = color_eyre::eyre::Result<T>;

fn main() -> Result<()> {
    setup_tracing()?;

    info!("Hello, world!");
    Ok(())
}

fn setup_tracing() -> Result<()> {
    color_eyre::install()?;
    let s = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or(EnvFilter::new(LevelFilter::INFO.to_string())),
        )
        .compact()
        .finish()
        .with(ErrorLayer::default());
    tracing::subscriber::set_global_default(s)?;
    Ok(())
}
