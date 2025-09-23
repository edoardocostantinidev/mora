use std::default;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use mora_core::entities::cluster_status::{ClusterStatus, ClusterStatusData};
use mora_core::entities::connections_info::ConnectionsInfo;
use mora_core::result::MoraError;
use ratatui::buffer::Buffer;
use ratatui::layout::{self, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Axis, Block, Borders, Chart, Dataset, GraphType, LegendPosition, List, ListDirection,
    Paragraph, Widget,
};
use ratatui::{symbols, Frame};

use mora_client::MoraClient;

#[derive(Debug, Clone)]
pub struct ConnectionPanelWidget {
    mora_client: MoraClient,
    state: Arc<RwLock<ConnectionsInfoState>>,
}

impl ConnectionPanelWidget {
    pub fn new(mora_client: &MoraClient) -> Self {
        let initial_state = ConnectionsInfoState::default();

        Self {
            mora_client: mora_client.clone(),
            state: Arc::new(RwLock::new(initial_state)),
        }
    }
}

#[derive(Debug, Default)]
struct ConnectionsInfoState {
    loading_state: LoadingState,
    connections_by_second: Vec<(u64, i64)>,
    already_fetched_once: bool,
}

#[derive(Debug, Clone, Default)]
enum LoadingState {
    #[default]
    Idle,
    Loading,
    Loaded(ConnectionsInfo),
    Error(String),
}

impl ConnectionPanelWidget {
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
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        self.state.write().unwrap().already_fetched_once = true;

        let random_between_100_and_500 = rand::random::<u64>() % 400 + 100;
        self.on_load(ConnectionsInfo {
            clients_connected: random_between_100_and_500,
        });
    }

    fn on_load(&self, connections_info: ConnectionsInfo) {
        let time = chrono::Utc::now()
            .timestamp_nanos_opt()
            .expect("are we seriously in year 2262?");
        let mut state = self.state.write().unwrap();

        if state.connections_by_second.len() >= 500 {
            state.connections_by_second.remove(0);
        }

        state
            .connections_by_second
            .push((connections_info.clients_connected, time));
    }

    fn on_err(&self, err: &MoraError) {
        self.set_loading_state(LoadingState::Error(err.to_string()));
    }

    fn set_loading_state(&self, state: LoadingState) {
        self.state.write().unwrap().loading_state = state;
    }
}

impl Widget for &ConnectionPanelWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let state = self.state.write().unwrap();
        let color = ratatui::style::Color::LightMagenta;

        let block = Block::bordered()
            .border_style(Style::default().fg(color))
            .title("Connections Info Panel");

        match &state.loading_state {
            LoadingState::Idle | LoadingState::Loading => {
                let items = [format!("Loading data... 🟡")];
                let list = List::new(items)
                    .block(Block::bordered().title("List"))
                    .style(Style::new().white())
                    .highlight_style(Style::new().italic())
                    .highlight_symbol(">>")
                    .repeat_highlight_symbol(true)
                    .direction(ListDirection::TopToBottom);

                list.render(area, buf);
            }
            LoadingState::Error(err) => {
                let items = [
                    format!("Server Offline! 🔴"),
                    "Can't retrieve connections info!".to_string(),
                    format!("Error: {err}"),
                ];
                let list = List::new(items)
                    .block(Block::bordered().title("List"))
                    .style(Style::new().white())
                    .highlight_style(Style::new().italic())
                    .highlight_symbol(">>")
                    .repeat_highlight_symbol(true)
                    .direction(ListDirection::TopToBottom);

                list.render(area, buf);
            }
            LoadingState::Loaded(_) => {
                connections_info_chart(
                    area,
                    buf,
                    self.state.read().unwrap().connections_by_second.clone(),
                );
            }
        }

        block.render(area, buf);
        return;
    }
}

fn connections_info_chart(area: Rect, buf: &mut Buffer, connections_by_second: Vec<(u64, i64)>) {
    let data = connections_by_second
        .iter()
        .map(|(x, y)| (*x as f64, *y as f64))
        .collect::<Vec<(f64, f64)>>();

    let datasets = vec![Dataset::default()
        .name("Line from only 2 points".italic())
        .marker(symbols::Marker::Braille)
        .style(Style::default().fg(Color::Yellow))
        .graph_type(GraphType::Line)
        .data(&data)];

    let chart = Chart::new(datasets)
        .block(Block::bordered().title(Line::from("Line chart").cyan().bold().centered()))
        .x_axis(
            Axis::default()
                .title("X Axis")
                .style(Style::default().gray())
                .bounds([0.0, 5.0])
                .labels(["0".bold(), "2.5".into(), "5.0".bold()]),
        )
        .y_axis(
            Axis::default()
                .title("Y Axis")
                .style(Style::default().gray())
                .bounds([0.0, 5.0])
                .labels(["0".bold(), "2.5".into(), "5.0".bold()]),
        )
        .legend_position(Some(LegendPosition::TopLeft))
        .hidden_legend_constraints((Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)));
    chart.render(area, buf);
}
