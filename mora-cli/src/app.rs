use std::time::Duration;

use crate::selectable::Selectable;
use crate::widgets::channel_panel::ChannelPanelWidget;
use crate::widgets::connection_panel::ConnectionPanelWidget;
use crate::widgets::queue_panel::QueuePanelWidget;
use crate::widgets::server_status::ServerStatusWidget;
use color_eyre::Result;
use crossterm::event::{Event, EventStream, KeyCode};
use mora_client::MoraClient;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::Block;
use ratatui::{DefaultTerminal, Frame};
use tokio_stream::StreamExt;

#[derive(Debug)]
pub struct App {
    should_quit: bool,
    server_status: ServerStatusWidget,
    connection_panel: ConnectionPanelWidget,
    channel_panel: ChannelPanelWidget,
    queue_panel: QueuePanelWidget,
    selected_panel: SelectedPanel,
}

#[derive(Debug)]
enum SelectedPanel {
    Queue,
    Channel,
    Connection,
}

impl App {
    const FRAMES_PER_SECOND: f32 = 60.0;

    pub fn new(mora_client: &MoraClient) -> Self {
        Self {
            should_quit: false,
            server_status: ServerStatusWidget::new(mora_client),
            connection_panel: ConnectionPanelWidget::new(mora_client),
            channel_panel: ChannelPanelWidget::new(mora_client),
            queue_panel: QueuePanelWidget::new(mora_client),
            selected_panel: SelectedPanel::Queue,
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.server_status.run();
        self.connection_panel.run();
        self.queue_panel.run();
        self.channel_panel.run();

        let period = Duration::from_secs_f32(1.0 / Self::FRAMES_PER_SECOND);
        let mut interval = tokio::time::interval(period);
        let mut events = EventStream::new();

        while !self.should_quit {
            tokio::select! {
                _ = interval.tick() => { terminal.draw(|frame| self.render(frame))?; },
                Some(Ok(event)) = events.next() => self.handle_event(&event),
            }
        }
        Ok(())
    }

    fn render(&self, frame: &mut Frame) {
        let body_area = frame.area();
        let block = Block::bordered()
            .title("Mora Dashboard")
            .title_bottom("q to quit");

        let outer_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(100)])
            .split(body_area);

        let main_layout_percentage = 85;
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(main_layout_percentage),
                Constraint::Percentage(100 - main_layout_percentage),
            ])
            .margin(1)
            .split(outer_layout[0]);

        let central_layout_percentage = 50;
        let central_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(central_layout_percentage),
                Constraint::Percentage(100 - central_layout_percentage),
            ])
            .split(main_layout[0]);

        let lower_bar = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_layout[1]);

        frame.render_widget(block, outer_layout[0]);
        frame.render_widget(&self.server_status, lower_bar[0]);
        frame.render_widget(&self.connection_panel, lower_bar[1]);
        frame.render_widget(&self.queue_panel, central_layout[0]);
        frame.render_widget(&self.channel_panel, central_layout[1]);
    }

    fn handle_event(&mut self, event: &Event) {
        if let Some(key) = event.as_key_press_event() {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                KeyCode::Tab => {
                    self.selected_panel = match self.selected_panel {
                        //tab moves panel
                        SelectedPanel::Queue => {
                            self.queue_panel.set_selected(false);
                            self.channel_panel.set_selected(true);
                            SelectedPanel::Channel
                        }
                        SelectedPanel::Channel => {
                            self.channel_panel.set_selected(false);
                            self.connection_panel.set_selected(true);
                            SelectedPanel::Connection
                        }
                        SelectedPanel::Connection => {
                            self.connection_panel.set_selected(false);
                            self.queue_panel.set_selected(true);
                            SelectedPanel::Queue
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
