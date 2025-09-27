use std::env;

use color_eyre::eyre::Error;
use mora_core::result::{MoraError, MoraResult};

pub(crate) mod app;
pub(crate) mod selectable;
pub(crate) mod widgets;

#[tokio::main]
async fn main() -> MoraResult<()> {
    let base_url = env::var("MORA_BASE_URL").unwrap_or("localhost".to_string());
    let port = env::var("MORA_PORT").unwrap_or("2626".to_string());
    let id_key = env::var("MORA_ID_KEY").unwrap_or("test".to_string());
    let mora_client = mora_client::MoraClient::new(base_url, port.parse::<u16>().unwrap(), id_key);

    color_eyre::install().map_err(handle_error)?;
    let terminal = ratatui::init();

    let app_result = app::App::new(&mora_client).run(terminal).await;
    ratatui::restore();
    app_result.map_err(handle_error)
}

fn handle_error(err: Error) -> MoraError {
    MoraError::GenericError(format!("Application error: {err}"))
}
