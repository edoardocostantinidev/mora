use std::sync::{Arc, RwLock};
use std::time::Duration;

use mora_core::models::queues::{ListQueuesResponse, Queue};
use mora_core::result::MoraError;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::widgets::{Block, List, ListDirection, Widget};

use mora_client::MoraClient;

use crate::selectable::Selectable;

#[derive(Debug, Clone)]
pub struct QueuePanelWidget {
    mora_client: MoraClient,
    state: Arc<RwLock<QueuesState>>,
    selected: bool,
}

impl QueuePanelWidget {
    pub fn new(mora_client: &MoraClient) -> Self {
        let initial_state = QueuesState::default();

        Self {
            mora_client: mora_client.clone(),
            state: Arc::new(RwLock::new(initial_state)),
            selected: false,
        }
    }
}

impl Selectable for QueuePanelWidget {
    fn is_selected(&self) -> bool {
        self.selected
    }

    fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }
}

#[derive(Debug, Default)]
struct QueuesState {
    loading_state: LoadingState,
    queues: Vec<Queue>,
    already_fetched_once: bool,
}

#[derive(Debug, Clone, Default)]
enum LoadingState {
    #[default]
    Idle,
    Loading,
    Loaded(Vec<Queue>),
    Error(String),
}

const REFRESH_INTERVAL_IN_MSEC: u64 = 500;

impl QueuePanelWidget {
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

        let queues_result = self.mora_client.get_queues().await;
        match queues_result {
            Ok(ListQueuesResponse { queues }) => {
                self.on_load(queues);
            }
            Err(err) => {
                self.on_err(&err);
            }
        }
    }

    fn on_load(&self, queues: Vec<Queue>) {
        let mut state = self.state.write().unwrap();
        state.already_fetched_once = true;
        state.queues = queues.clone();
        state.loading_state = LoadingState::Loaded(queues);
    }

    fn on_err(&self, err: &MoraError) {
        let mut state = self.state.write().unwrap();
        state.already_fetched_once = true;
        state.loading_state = LoadingState::Error(err.to_string());
    }
}

impl Widget for &QueuePanelWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let state = self.state.read().unwrap();
        let color = ratatui::style::Color::LightBlue;

        let modifier = if self.is_selected() {
            Modifier::BOLD
        } else {
            Modifier::empty()
        };

        let block = Block::bordered()
            .title("Queues")
            .border_style(Style::default().fg(color))
            .border_type(ratatui::widgets::BorderType::Rounded)
            .add_modifier(modifier);

        let items = match &state.loading_state {
            LoadingState::Idle => vec![format!("Initializing...")],
            LoadingState::Loading => vec![format!("Loading data... 🟡")],
            LoadingState::Error(err) => vec![
                format!("Server Offline! 🔴"),
                "Can't retrieve queues!".to_string(),
                format!("Error: {err}"),
            ],
            LoadingState::Loaded(queues) => queues
                .iter()
                .map(|queue| format!("{}: {}", queue.id, queue.pending_events_count))
                .collect::<Vec<String>>(),
        };

        let list = List::new(items)
            .block(block)
            .style(Style::new().white())
            .highlight_style(Style::new().italic())
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);

        list.render(area, buf);

        return;
    }
}
