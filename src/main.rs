use mora_core::result::MoraResult;
use mora_server::{config::MoraConfig, Server};
fn main() -> MoraResult<()> {
    let config = MoraConfig::from_env();
    let server = Server::new(config.ok())?;
    server.run()
}
