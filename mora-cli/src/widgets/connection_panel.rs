use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use color_eyre::owo_colors::OwoColorize;
use mora_core::entities::connections_info::ConnectionsInfo;
use mora_core::result::MoraError;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::symbols;
use ratatui::text::Line;
use ratatui::widgets::{
    Axis, Block, Chart, Dataset, GraphType, LegendPosition, List, ListDirection, Widget,
};

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

type ConnectionsBySecond = VecDeque<(u64, i64)>; // (connections, timestamp)

#[derive(Debug, Default)]
struct ConnectionsInfoState {
    loading_state: LoadingState,
    connections_by_second: ConnectionsBySecond,
    already_fetched_once: bool,
}

#[derive(Debug, Clone, Default)]
enum LoadingState {
    #[default]
    Idle,
    Loading,
    Loaded(ConnectionsBySecond),
    Error(String),
}

const MAX_POINTS_IN_CHART: usize = 50;
impl ConnectionPanelWidget {
    pub fn run(&self) {
        let this = self.clone();
        tokio::spawn(async move {
            loop {
                this.fetch_status().await;
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        });
    }

    async fn fetch_status(&self) {
        if !self.state.read().unwrap().already_fetched_once {
            self.set_loading_state(LoadingState::Loading);
        }

        let random_num = rand::random::<u64>() % 400 + 100;
        self.on_load(ConnectionsInfo {
            clients_connected: random_num,
        });
    }

    fn on_load(&self, connections_info: ConnectionsInfo) {
        let time = chrono::Utc::now()
            .timestamp_nanos_opt()
            .expect("are we seriously in year 2262?");
        let mut state = self.state.write().unwrap();

        if state.connections_by_second.len() >= MAX_POINTS_IN_CHART {
            state.connections_by_second.pop_front();
        }

        state.already_fetched_once = true;
        state
            .connections_by_second
            .push_back((connections_info.clients_connected, time));
        state.loading_state = LoadingState::Loaded(state.connections_by_second.clone());
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
            LoadingState::Loaded(connections_info) => {
                connections_info_chart(area, buf, connections_info);
            }
        }

        block.render(area, buf);
        return;
    }
}

fn connections_info_chart(
    area: Rect,
    buf: &mut Buffer,
    connections_by_second: &ConnectionsBySecond,
) {
    let mut max_connections = 0.0;

    let data = connections_by_second
        .iter()
        .map(|(connections, timestamp)| (*connections as f64, *timestamp as f64))
        .collect::<Vec<(f64, f64)>>();

    data.iter().for_each(|(x, _)| {
        if *x > max_connections {
            max_connections = *x;
        }
    });

    let min_time = data.first().map(|(_, t)| *t as f64).unwrap_or(0.0);
    let max_time = data.last().map(|(_, t)| *t as f64).unwrap_or(0.0);
    let max_connections_label = format!("{:.0}", max_connections * 1.25);

    let adjusted_data = data
        .iter()
        .map(|(x, y)| (*x, ((*y) / 1e9).round()))
        .collect::<Vec<(f64, f64)>>();

    let bound_min_time = adjusted_data.first().map(|(_, t)| *t).unwrap_or(0.0) - 400.0;
    let bound_max_time = adjusted_data.last().map(|(_, t)| *t).unwrap_or(0.0) + 100.0;
    let bound_min_connections = 0.0;
    let bound_max_connections = 1000.0;

    let min_time_label = chrono::DateTime::from_timestamp_nanos(min_time as i64)
        .format("%H:%M:%S")
        .to_string();
    let max_time_label = chrono::DateTime::from_timestamp_nanos(max_time as i64)
        .format("%H:%M:%S")
        .to_string();

    let datasets = vec![Dataset::default()
        .marker(symbols::Marker::Braille)
        .style(Style::default().fg(Color::Magenta))
        .graph_type(GraphType::Line)
        .data(&adjusted_data)];

    let chart = Chart::new(datasets)
        .block(Block::bordered())
        .x_axis(
            Axis::default()
                .style(Style::default().hidden())
                .bounds([bound_min_time, bound_max_time])
                .labels([min_time_label, max_time_label])
                .labels_alignment(Alignment::Center),
        )
        .y_axis(
            Axis::default()
                .style(Style::default().hidden())
                .bounds([bound_min_connections, bound_max_connections])
                .labels(["0".to_string(), max_connections_label])
                .labels_alignment(Alignment::Center),
        )
        .legend_position(None);

    let items = adjusted_data
        .iter()
        .map(|(x, y)| {
            format!(
                "Bounds X({},{}) Bounds y({},{}) Connections: {:.0} at {:.0}",
                bound_min_connections, bound_max_connections, bound_min_time, bound_max_time, x, y
            )
        })
        .collect::<Vec<String>>();
    let list = List::new(items)
        .block(Block::bordered().title("List"))
        .style(Style::new().white())
        .highlight_style(Style::new().italic())
        .highlight_symbol(">>")
        .repeat_highlight_symbol(true)
        .direction(ListDirection::TopToBottom);

    list.render(area, buf);
    //chart.render(area, buf);
}
