use std::sync::{Arc, RwLock};
use std::time::Duration;

use mora_core::models::events::Event;
use mora_core::models::queues::{ListQueuesResponse, Queue};
use mora_core::result::MoraError;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::widgets::{Block, List, ListDirection, ListItem, ListState, Paragraph, Widget, Wrap};

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

    pub fn move_up(&self) {
        let mut state = self.state.write().unwrap();
        if let ViewMode::List = state.view_mode {
            // Only allow navigation when queues are loaded
            if matches!(state.loading_state, LoadingState::Loaded(_)) && !state.queues.is_empty() {
                if state.selected_index > 0 {
                    state.selected_index -= 1;
                }
            }
        }
    }

    pub fn move_down(&self) {
        let mut state = self.state.write().unwrap();
        if let ViewMode::List = state.view_mode {
            // Only allow navigation when queues are loaded
            if matches!(state.loading_state, LoadingState::Loaded(_)) && !state.queues.is_empty() {
                let max_index = state.queues.len().saturating_sub(1);
                if state.selected_index < max_index {
                    state.selected_index += 1;
                }
            }
        }
    }

    pub fn drill_in(&self) {
        let mut this = self.clone();
        tokio::spawn(async move {
            let queue_id = {
                let mut state = this.state.write().unwrap();
                // Only drill in when queues are loaded
                if let ViewMode::List = state.view_mode {
                    if matches!(state.loading_state, LoadingState::Loaded(_)) {
                        if let Some(queue) = state.queues.get(state.selected_index) {
                            let queue_id = queue.id.clone();
                            state.view_mode = ViewMode::LoadingEvents;
                            Some(queue_id)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            if let Some(queue_id) = queue_id {
                // Create a temporary channel to view events
                match this
                    .mora_client
                    .create_channel(vec![queue_id.clone()], 100, u64::MAX)
                    .await
                {
                    Ok(channel_id) => {
                        match this
                            .mora_client
                            .get_channel_events(channel_id.clone(), false)
                            .await
                        {
                            Ok(events) => {
                                let mut state = this.state.write().unwrap();
                                state.view_mode = ViewMode::ViewingEvents(queue_id.clone(), events);
                            }
                            Err(e) => {
                                let mut state = this.state.write().unwrap();
                                state.view_mode = ViewMode::EventsError(e.to_string());
                            }
                        }
                        let _ = this.mora_client.delete_channel(channel_id).await;
                    }
                    Err(e) => {
                        let mut state = this.state.write().unwrap();
                        state.view_mode = ViewMode::EventsError(e.to_string());
                    }
                }
            }
        });
    }

    pub fn go_back(&self) {
        let mut state = self.state.write().unwrap();
        if !matches!(state.view_mode, ViewMode::List) {
            state.view_mode = ViewMode::List;
        }
    }

    pub fn request_delete(&self) {
        let mut state = self.state.write().unwrap();
        // Only allow delete when queues are loaded
        if let ViewMode::List = state.view_mode {
            if matches!(state.loading_state, LoadingState::Loaded(_)) {
                if let Some(queue) = state.queues.get(state.selected_index) {
                    state.view_mode = ViewMode::ConfirmDelete(queue.id.clone());
                }
            }
        }
    }

    pub fn confirm_delete(&self) {
        let mut this = self.clone();
        tokio::spawn(async move {
            let queue_id = {
                let mut state = this.state.write().unwrap();
                if let ViewMode::ConfirmDelete(queue_id) = &state.view_mode {
                    let queue_id = queue_id.clone();
                    state.view_mode = ViewMode::DeletingQueue(queue_id.clone());
                    Some(queue_id)
                } else {
                    None
                }
            };

            if let Some(queue_id) = queue_id {
                match this.mora_client.delete_queue(queue_id.clone()).await {
                    Ok(_) => {
                        let mut state = this.state.write().unwrap();
                        // Remove the queue from the list
                        state.queues.retain(|q| q.id != queue_id);
                        // Adjust selected index if needed
                        if state.selected_index >= state.queues.len() && state.selected_index > 0 {
                            state.selected_index = state.queues.len() - 1;
                        }
                        state.view_mode = ViewMode::List;
                    }
                    Err(e) => {
                        let mut state = this.state.write().unwrap();
                        state.view_mode = ViewMode::DeleteError(e.to_string());
                    }
                }
            }
        });
    }

    pub fn cancel_delete(&self) {
        let mut state = self.state.write().unwrap();
        if matches!(state.view_mode, ViewMode::ConfirmDelete(_)) {
            state.view_mode = ViewMode::List;
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

#[derive(Debug, Clone)]
enum ViewMode {
    List,
    LoadingEvents,
    ViewingEvents(String, Vec<Event>),
    EventsError(String),
    ConfirmDelete(String), // queue_id to delete
    DeletingQueue(String), // queue_id being deleted
    DeleteError(String),   // error message
}

impl Default for ViewMode {
    fn default() -> Self {
        ViewMode::List
    }
}

#[derive(Debug, Default)]
struct QueuesState {
    loading_state: LoadingState,
    queues: Vec<Queue>,
    already_fetched_once: bool,
    selected_index: usize,
    view_mode: ViewMode,
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
    pub fn run(&mut self) {
        let mut this = self.clone();
        tokio::spawn(async move {
            loop {
                this.fetch_status().await;
                tokio::time::sleep(Duration::from_millis(REFRESH_INTERVAL_IN_MSEC)).await;
            }
        });
    }

    async fn fetch_status(&mut self) {
        let already_fetched = match self.state.read() {
            Ok(state) => state.already_fetched_once,
            Err(poisoned) => poisoned.into_inner().already_fetched_once,
        };

        if !already_fetched {
            if let Ok(mut state) = self.state.write() {
                state.loading_state = LoadingState::Loading;
            }
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
        if let Ok(mut state) = self.state.write() {
            state.already_fetched_once = true;
            state.queues = queues.clone();
            state.loading_state = LoadingState::Loaded(queues);
        }
    }

    fn on_err(&self, err: &MoraError) {
        if let Ok(mut state) = self.state.write() {
            state.already_fetched_once = true;
            state.loading_state = LoadingState::Error(err.to_string());
            // Reset selected index when entering error state
            state.selected_index = 0;
        }
    }
}

impl Widget for &QueuePanelWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Handle poisoned lock gracefully
        let state = match self.state.read() {
            Ok(state) => state,
            Err(poisoned) => poisoned.into_inner(),
        };
        let color = ratatui::style::Color::LightBlue;

        let modifier = if self.is_selected() {
            Modifier::BOLD
        } else {
            Modifier::empty()
        };

        match &state.view_mode {
            ViewMode::List => {
                let block = Block::bordered()
                    .title("Queues (j/k: navigate, Enter: view events, Backspace: delete)")
                    .border_style(Style::default().fg(color))
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .add_modifier(modifier);

                let (items, maybe_paragraph) = match &state.loading_state {
                    LoadingState::Idle => (vec![ListItem::new("Initializing...")], None),
                    LoadingState::Loading => (vec![ListItem::new("Loading data... 🟡")], None),
                    LoadingState::Error(err) => (
                        vec![
                            ListItem::new("Server Offline! 🔴"),
                            ListItem::new("Can't retrieve queues!"),
                        ],
                        Some(Paragraph::new(format!("Error: {err}")).wrap(Wrap { trim: false })),
                    ),
                    LoadingState::Loaded(queues) => (
                        queues
                            .iter()
                            .enumerate()
                            .map(|(idx, queue)| {
                                let text = format!(
                                    "{} - {}: {} events",
                                    idx + 1,
                                    queue.id,
                                    queue.pending_events_count
                                );
                                let (prefix, bg, fg) = if idx == state.selected_index {
                                    (
                                        "*",
                                        ratatui::style::Color::White,
                                        ratatui::style::Color::Black,
                                    )
                                } else {
                                    (
                                        " ",
                                        ratatui::style::Color::Black,
                                        ratatui::style::Color::White,
                                    )
                                };

                                ListItem::new(format!("{prefix}{text}")).bg(bg).fg(fg)
                            })
                            .collect::<Vec<ListItem>>(),
                        None,
                    ),
                };

                let layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(vec![Constraint::Percentage(100)])
                    .split(area);

                let mut list_state = ListState::default();
                // Only set selection when we have loaded queues and the panel is selected
                if self.is_selected() && matches!(state.loading_state, LoadingState::Loaded(_)) {
                    // Ensure selected_index is within bounds
                    if state.selected_index < state.queues.len() {
                        list_state.select(Some(state.selected_index));
                    }
                }

                let list = List::new(items)
                    .block(block)
                    .style(Style::new().white())
                    .highlight_style(
                        Style::default()
                            .bg(ratatui::style::Color::DarkGray)
                            .fg(ratatui::style::Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol(">> ")
                    .direction(ListDirection::TopToBottom);

                ratatui::widgets::StatefulWidget::render(list, layout[0], buf, &mut list_state);

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
            }
            ViewMode::LoadingEvents => {
                let block = Block::bordered()
                    .title("Loading Events...")
                    .border_style(Style::default().fg(color))
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .add_modifier(modifier);

                let paragraph = Paragraph::new("Fetching events... 🟡")
                    .block(block)
                    .alignment(Alignment::Center);

                paragraph.render(area, buf);
            }
            ViewMode::ViewingEvents(queue_id, events) => {
                let block = Block::bordered()
                    .title(format!(
                        "Events in '{}' ({} total, Esc: back)",
                        queue_id,
                        events.len()
                    ))
                    .border_style(Style::default().fg(color))
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .add_modifier(modifier);

                let items: Vec<ListItem> = if events.is_empty() {
                    vec![ListItem::new("No events found")]
                } else {
                    events
                        .iter()
                        .enumerate()
                        .map(|(idx, event)| {
                            let text = format!(
                                "{}. [ts: {}] {}",
                                idx + 1,
                                event.timestamp,
                                event.data.chars().take(60).collect::<String>()
                            );
                            ListItem::new(text)
                        })
                        .collect()
                };

                let list = List::new(items)
                    .block(block)
                    .style(Style::new().white())
                    .highlight_style(
                        Style::default()
                            .bg(ratatui::style::Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol(">> ")
                    .direction(ListDirection::TopToBottom);

                list.render(area, buf);
            }
            ViewMode::EventsError(err) => {
                let block = Block::bordered()
                    .title("Error (Esc: back)")
                    .border_style(Style::default().fg(ratatui::style::Color::Red))
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .add_modifier(modifier);

                let paragraph = Paragraph::new(format!("Error loading events: {err}"))
                    .block(block)
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: false });

                paragraph.render(area, buf);
            }
            ViewMode::ConfirmDelete(queue_id) => {
                let block = Block::bordered()
                    .title("Confirm Delete")
                    .border_style(Style::default().fg(ratatui::style::Color::Red))
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .add_modifier(modifier);

                let text = format!(
                    "Are you sure you want to delete queue '{}'?\n\n\
                     This action cannot be undone.\n\n\
                     Press 'y' to confirm or 'n' to cancel",
                    queue_id
                );

                let paragraph = Paragraph::new(text)
                    .block(block)
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: false });

                paragraph.render(area, buf);
            }
            ViewMode::DeletingQueue(queue_id) => {
                let block = Block::bordered()
                    .title("Deleting Queue...")
                    .border_style(Style::default().fg(ratatui::style::Color::Yellow))
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .add_modifier(modifier);

                let paragraph = Paragraph::new(format!("Deleting queue '{}'... 🟡", queue_id))
                    .block(block)
                    .alignment(Alignment::Center);

                paragraph.render(area, buf);
            }
            ViewMode::DeleteError(err) => {
                let block = Block::bordered()
                    .title("Delete Error (Esc: back)")
                    .border_style(Style::default().fg(ratatui::style::Color::Red))
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .add_modifier(modifier);

                let paragraph = Paragraph::new(format!("Error deleting queue: {err}"))
                    .block(block)
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: false });

                paragraph.render(area, buf);
            }
        }
    }
}
