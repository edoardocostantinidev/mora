use std::sync::{Arc, RwLock};
use std::time::Duration;

use mora_core::models::health::{ClusterStatus, ClusterStatusData};
use mora_core::result::MoraError;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListDirection, Widget};

use crate::selectable::Selectable;

use mora_client::MoraClient;

#[derive(Debug, Clone)]
pub struct ServerStatusWidget {
    mora_client: MoraClient,
    state: Arc<RwLock<ServerStatusState>>,
    selected: bool,
}

impl ServerStatusWidget {
    pub fn new(mora_client: &MoraClient) -> Self {
        let url = mora_client
            .build_url("health")
            .map(|u| u.to_string())
            .unwrap_or_else(|_| "Invalid URL".to_string());

        let initial_state = ServerStatusState::new(url);

        Self {
            mora_client: mora_client.clone(),
            state: Arc::new(RwLock::new(initial_state)),
            selected: false,
        }
    }
}

impl Selectable for ServerStatusWidget {
    fn is_selected(&self) -> bool {
        self.selected
    }

    fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }
}

#[derive(Debug, Default)]
struct ServerStatusState {
    loading_state: LoadingState,
    cluster_status: ClusterStatus,
    connection_url: String,
    already_fetched_once: bool,
}

impl ServerStatusState {
    fn new(connection_url: String) -> Self {
        Self {
            loading_state: LoadingState::Idle,
            cluster_status: ClusterStatus::Offline,
            connection_url: connection_url,
            already_fetched_once: false,
        }
    }
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
        if !self.state.read().unwrap().already_fetched_once {
            self.set_loading_state(LoadingState::Loading);
        }

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
                state.already_fetched_once = true;
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
        let color = match &state.cluster_status {
            ClusterStatus::Online(_) => ratatui::style::Color::Green,
            ClusterStatus::Degraded(_) => ratatui::style::Color::Yellow,
            ClusterStatus::Offline => ratatui::style::Color::Red,
        };

        let modifier = if self.is_selected() {
            Modifier::BOLD
        } else {
            Modifier::empty()
        };

        let block = Block::bordered()
            .border_style(Style::default().fg(color))
            .border_type(ratatui::widgets::BorderType::Rounded)
            .title("Server Status")
            .add_modifier(modifier);

        match &state.loading_state {
            LoadingState::Idle | LoadingState::Loading => {
                block.render(area, buf);
            }
            LoadingState::Error(err) => {
                let items = [
                    format!("Server Offline! 🔴"),
                    format!("URL: {}", state.connection_url),
                    format!("Error: {err}"),
                ];
                let list = List::new(items)
                    .block(block)
                    .style(Style::new().white())
                    .highlight_style(Style::new().italic())
                    .highlight_symbol(">>")
                    .repeat_highlight_symbol(true)
                    .direction(ListDirection::TopToBottom);

                list.render(area, buf);
            }
            LoadingState::Loaded(ClusterStatusData {
                version,
                current_time_in_ns,
            }) => {
                server_health_list(area, buf, version, current_time_in_ns, block);
            }
        }

        return;
    }
}

fn server_health_list(
    area: Rect,
    buf: &mut Buffer,
    version: &str,
    current_time_in_ns: &u128,
    block: Block,
) {
    // Convert u128 to i64 safely, saturating at i64::MAX if necessary
    let current_time_in_ns_i64 = if *current_time_in_ns > i64::MAX as u128 {
        i64::MAX
    } else {
        *current_time_in_ns as i64
    };

    let current_time = chrono::DateTime::from_timestamp_nanos(current_time_in_ns_i64);

    let items = vec![
        format!("Server Online! 🟢"),
        format!("Mora Version: {version}"),
        format!("Current Server Time: {}", current_time),
    ];
    let list = List::new(items)
        .block(block)
        .style(Style::new().white())
        .repeat_highlight_symbol(true)
        .direction(ListDirection::TopToBottom);

    list.render(area, buf);
}
