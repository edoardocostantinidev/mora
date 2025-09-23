use std::time::Duration;

use crate::widgets::server_status::ServerStatusWidget;
use color_eyre::Result;
use crossterm::event::{Event, EventStream, KeyCode};
use mora_client::MoraClient;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::{DefaultTerminal, Frame};
use tokio_stream::StreamExt;

#[derive(Debug)]
pub struct App {
    should_quit: bool,
    server_status: ServerStatusWidget,
}

impl App {
    const FRAMES_PER_SECOND: f32 = 60.0;

    pub fn new(mora_client: &MoraClient) -> Self {
        Self {
            should_quit: false,
            server_status: ServerStatusWidget::new(mora_client),
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.server_status.run();

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
        let layout = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]);
        let body_area = frame.area();
        let title_area = layout.split(body_area)[0];
        let title = Line::from("Mora Control Panel").centered().bold();
        frame.render_widget(title, title_area);
        frame.render_widget(&self.server_status, body_area);
    }

    fn handle_event(&mut self, event: &Event) {
        if let Some(key) = event.as_key_press_event() {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                _ => {}
            }
        }
    }
}
