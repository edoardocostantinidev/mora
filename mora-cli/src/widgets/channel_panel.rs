use std::sync::{Arc, RwLock};
use std::time::Duration;

use mora_core::models::channels::{Channel, ListChannelsResponse};
use mora_core::result::MoraError;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::widgets::{Block, List, ListDirection, Paragraph, Widget, Wrap};

use crate::selectable::Selectable;

use mora_client::MoraClient;

#[derive(Debug, Clone)]
pub struct ChannelPanelWidget {
    mora_client: MoraClient,
    state: Arc<RwLock<ChannelsState>>,
    selected: bool,
}

impl ChannelPanelWidget {
    pub fn new(mora_client: &MoraClient) -> Self {
        let initial_state = ChannelsState::default();

        Self {
            mora_client: mora_client.clone(),
            state: Arc::new(RwLock::new(initial_state)),
            selected: false,
        }
    }
}

impl Selectable for ChannelPanelWidget {
    fn is_selected(&self) -> bool {
        self.selected
    }

    fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }
}

#[derive(Debug, Default)]
struct ChannelsState {
    loading_state: LoadingState,
    already_fetched_once: bool,
}

#[derive(Debug, Clone, Default)]
enum LoadingState {
    #[default]
    Idle,
    Loading,
    Loaded(Vec<Channel>),
    Error(String),
}

const REFRESH_INTERVAL_IN_MSEC: u64 = 500;

impl ChannelPanelWidget {
    pub fn run(&self) {
        let this = self.clone();
        tokio::spawn(async move {
            loop {
                this.fetch_status().await;
                tokio::time::sleep(Duration::from_millis(REFRESH_INTERVAL_IN_MSEC)).await;
            }
        });
    }

    async fn fetch_status(&self) {
        if !self.state.read().unwrap().already_fetched_once {
            let mut state = self.state.write().unwrap();
            state.loading_state = LoadingState::Loading;
        }

        let channels_result = self.mora_client.get_channels().await;
        match channels_result {
            Ok(ListChannelsResponse { channels }) => {
                self.on_load(channels);
            }
            Err(err) => {
                self.on_err(&err);
            }
        }
    }

    fn on_load(&self, channels: Vec<Channel>) {
        let mut state = self.state.write().unwrap();
        state.already_fetched_once = true;
        state.loading_state = LoadingState::Loaded(channels);
    }

    fn on_err(&self, err: &MoraError) {
        let mut state = self.state.write().unwrap();
        state.already_fetched_once = true;
        state.loading_state = LoadingState::Error(err.to_string());
    }
}

impl Widget for &ChannelPanelWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let state = self.state.write().unwrap();
        let color = ratatui::style::Color::Yellow;

        let modifier = if self.is_selected() {
            Modifier::BOLD
        } else {
            Modifier::empty()
        };

        let block = Block::bordered()
            .title("Channels")
            .border_style(Style::default().fg(color))
            .border_type(ratatui::widgets::BorderType::Rounded)
            .add_modifier(modifier);

        let (items, maybe_paragraph) = match &state.loading_state {
            LoadingState::Idle => (vec![format!("Initializing...")], None),
            LoadingState::Loading => (vec![format!("Loading data... 🟡")], None),
            LoadingState::Error(err) => (
                vec![
                    format!("Server Offline! 🔴"),
                    "Can't retrieve channels!".to_string(),
                ],
                Some(Paragraph::new(format!("Error: {err}")).wrap(Wrap { trim: false })),
            ),
            LoadingState::Loaded(channels) => (
                channels
                    .iter()
                    .map(|channel| format!("{}: {}", channel.channel_id, channel.msec_from_last_op))
                    .collect::<Vec<String>>(),
                None,
            ),
        };

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(100)])
            .split(area);

        let list = List::new(items)
            .block(block)
            .style(Style::new().white())
            .highlight_style(Style::new().italic())
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);

        list.render(layout[0], buf);

        if let Some(paragraph) = maybe_paragraph {
            let error_layout_percentage = 20;
            let error_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Percentage(error_layout_percentage),
                    Constraint::Percentage(100 - error_layout_percentage),
                ])
                .margin(2)
                .split(layout[0]);

            let error_block = Block::bordered()
                .title("Error")
                .border_style(Style::default().fg(ratatui::style::Color::Red))
                .border_type(ratatui::widgets::BorderType::Rounded);

            paragraph
                .block(error_block)
                .alignment(Alignment::Center)
                .render(error_layout[1], buf);
        }

        return;
    }
}
