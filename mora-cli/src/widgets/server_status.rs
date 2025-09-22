use std::sync::{Arc, RwLock};
use std::vec;

use mora_core::result::MoraError;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, HighlightSpacing, Row, StatefulWidget, Table, TableState, Widget};

#[derive(Debug, Clone, Default)]
pub struct ServerStatusWidget {
    state: Arc<RwLock<ServerStatusListState>>,
}

#[derive(Debug, Default)]
struct ServerStatusListState {
    loading_state: LoadingState,
    table_state: TableState,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
enum LoadingState {
    #[default]
    Idle,
    Loading,
    Loaded,
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ServerStatus {
    Online,
    Degraded,
    Offline,
}

impl ServerStatusWidget {
    pub fn run(&self) {
        let this = self.clone();
        tokio::spawn(async move {
            this.fetch_status().await;
        });
    }

    async fn fetch_status(self) {
        self.set_loading_state(LoadingState::Loading);
        // simulate a network request
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        self.on_load(&ServerStatus::Online);
    }

    fn on_load(&self, status: &ServerStatus) {
        let mut state = self.state.write().unwrap();
        match status {
            ServerStatus::Online => {
                state.loading_state = LoadingState::Loaded;
            }
            ServerStatus::Degraded => {
                state.loading_state = LoadingState::Error(format!("{:?}", ServerStatus::Degraded));
            }
            ServerStatus::Offline => {
                state.loading_state = LoadingState::Error(format!("{:?}", ServerStatus::Offline));
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
            .title_bottom("j/k to scroll, q to quit");

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
                let error = Line::from(format!("Error: {err}")).centered();
                block.render(area, buf);
                buf.set_string(
                    area.x + area.width / 2 - error.width() as u16 / 2,
                    area.y + area.height / 2,
                    err.to_string(),
                    Style::default().fg(ratatui::style::Color::Red),
                );
            }
            LoadingState::Loaded => {
                let loading = Line::from("Loading...").centered();
                block.render(area, buf);
                buf.set_string(
                    area.x + area.width / 2 - loading.width() as u16 / 2,
                    area.y + area.height / 2,
                    "loaded!".to_string(),
                    Style::default().fg(ratatui::style::Color::Yellow),
                );
            }
        }

        return;
    }
}
