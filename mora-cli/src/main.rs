use color_eyre::eyre::Error;
use mora_core::result::{MoraError, MoraResult};

pub(crate) mod app;
pub(crate) mod widgets;

#[tokio::main]
async fn main() -> MoraResult<()> {
    color_eyre::install().map_err(handle_error)?;
    let terminal = ratatui::init();
    let app_result = app::App::default().run(terminal).await;
    ratatui::restore();
    app_result.map_err(handle_error)
}

fn handle_error(err: Error) -> MoraError {
    MoraError::GenericError(format!("Application error: {err}"))
}
