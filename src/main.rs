use tracing::{info, level_filters::LevelFilter};
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};

mod compute_pipeline;
mod context;
mod event_loop;
mod render_pipeline;

pub(crate) type Result<T> = color_eyre::eyre::Result<T>;

fn main() -> Result<()> {
    setup_tracing()?;
    pollster::block_on(event_loop::run())?;

    info!("Done");
    Ok(())
}

fn setup_tracing() -> Result<()> {
    color_eyre::install()?;

    let s = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or(EnvFilter::new(LevelFilter::INFO.to_string())),
        )
        .compact()
        .finish()
        .with(ErrorLayer::default());
    tracing::subscriber::set_global_default(s)?;

    Ok(())
}
