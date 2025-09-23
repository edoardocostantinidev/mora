use std::sync::{Arc, RwLock};
use std::time::Duration;

use mora_core::entities::cluster_status::{ClusterStatus, ClusterStatusData};
use mora_core::result::MoraError;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, List, ListDirection, Widget};

use mora_client::MoraClient;

#[derive(Debug, Clone)]
pub struct ServerStatusWidget {
    mora_client: MoraClient,
    state: Arc<RwLock<ServerStatusListState>>,
}

impl ServerStatusWidget {
    pub fn new(mora_client: &MoraClient) -> Self {
        Self {
            mora_client: mora_client.clone(),
            state: Arc::new(RwLock::new(ServerStatusListState::default())),
        }
    }
}

#[derive(Debug, Default)]
struct ServerStatusListState {
    loading_state: LoadingState,
    cluster_status: ClusterStatus,
}

#[derive(Debug, Clone, Default)]
enum LoadingState {
    #[default]
    Idle,
    Loading,
    Loaded(ClusterStatusData),
    Error(String),
}

impl ServerStatusWidget {
    pub fn run(&self) {
        let this = self.clone();
        tokio::spawn(async move {
            loop {
                this.fetch_status().await;
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
    }

    async fn fetch_status(&self) {
        self.set_loading_state(LoadingState::Loading);
        let cluster_status_result = self.mora_client.clone().get_cluster_status().await;
        match cluster_status_result {
            Ok(cluster_status) => {
                self.on_load(cluster_status);
            }
            Err(error) => {
                self.on_err(&error);
            }
        }
    }

    fn on_load(&self, status: ClusterStatus) {
        let mut state = self.state.write().unwrap();
        state.cluster_status = status.clone();
        match status {
            ClusterStatus::Online(data) => {
                state.loading_state = LoadingState::Loaded(data);
            }
            ClusterStatus::Degraded(error) => {
                state.loading_state = LoadingState::Error(format!("Degraded: {:?}", error));
            }
            ClusterStatus::Offline => {
                state.loading_state = LoadingState::Error(format!("{:?}", ClusterStatus::Offline));
            }
        }
    }

    fn on_err(&self, err: &MoraError) {
        self.set_loading_state(LoadingState::Error(err.to_string()));
    }

    fn set_loading_state(&self, state: LoadingState) {
        self.state.write().unwrap().loading_state = state;
    }
}

impl Widget for &ServerStatusWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let state = self.state.write().unwrap();
        let loading_state = Line::from(format!("{:?}", state.loading_state)).right_aligned();
        let block = Block::bordered()
            .title("Mora Control Panel")
            .title(loading_state)
            .title_bottom("q to quit");

        match &state.loading_state {
            LoadingState::Idle | LoadingState::Loading => {
                let loading = Line::from("Loading...").centered();
                block.render(area, buf);
                buf.set_string(
                    area.x + area.width / 2 - loading.width() as u16 / 2,
                    area.y + area.height / 2,
                    "loading...".to_string(),
                    Style::default().fg(ratatui::style::Color::Yellow),
                );
            }
            LoadingState::Error(err) => {
                let items = [format!("Error: {err}")];
                let list = List::new(items)
                    .block(Block::bordered().title("List"))
                    .style(Style::new().white())
                    .highlight_style(Style::new().italic())
                    .highlight_symbol(">>")
                    .repeat_highlight_symbol(true)
                    .direction(ListDirection::BottomToTop);

                list.render(area, buf);
            }
            LoadingState::Loaded(ClusterStatusData {
                version,
                current_time_in_ns,
            }) => {
                // Convert u128 to i64 safely, saturating at i64::MAX if necessary
                let current_time_in_ns_i64 = if *current_time_in_ns > i64::MAX as u128 {
                    i64::MAX
                } else {
                    *current_time_in_ns as i64
                };
                let current_time = chrono::DateTime::from_timestamp_nanos(current_time_in_ns_i64);

                let items = [
                    format!("Version: {version}"),
                    format!(
                        "Current Server Time: {}, ({} ns)",
                        current_time.to_string(),
                        current_time_in_ns
                    ),
                ];
                let list = List::new(items)
                    .block(Block::bordered().title("List"))
                    .style(Style::new().white())
                    .highlight_style(Style::new().italic())
                    .highlight_symbol(">>")
                    .repeat_highlight_symbol(true)
                    .direction(ListDirection::BottomToTop);

                list.render(area, buf);
            }
        }

        return;
    }
}
