use std::time::Duration;
use anyhow::{anyhow, Error, Result};
use crossterm::event;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect, Direction},
    style::Color,
    text::{Line, Span},
    widgets::{Block, Tabs, Widget, BorderType, Borders},
    DefaultTerminal, Frame,
};
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct App {
    focused: Focus,
    mode: Mode,
}
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Mode {
    #[default]
    Running,
    Quit,
}
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Focus {
    #[default]
    Devices,
    Children,
    Stats,
}
impl App {
    /// Run the app until the user quits.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while self.is_running() {
            terminal
                .draw(|frame| self.draw(frame))?; //this needs an anyhow or color_eyre wrap
            self.handle_events()?;
        }
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.mode != Mode::Quit
    }

    /// Draw a single frame of the app.
    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
        //if self.mode == Mode::Destroy {
        //    destroy::destroy(frame);
        //}
    }

    /// Handle events from the terminal.
    ///
    /// This function is called once per frame, The events are polled from the stdin with timeout of
    /// 1/50th of a second. This was chosen to try to match the default frame rate of a GIF in VHS.
    fn handle_events(&mut self) -> Result<()> {
        let timeout = Duration::from_secs_f64(1.0 / 50.0);
        if !event::poll(timeout)? {
            return Ok(());
        }
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => self.handle_key_press(key),
            _ => {}
        }
        Ok(())
    }

    fn handle_key_press(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.mode = Mode::Quit,
            KeyCode::Char('d') => self.focused = Focus::Devices,
            KeyCode::Char('s') => self.focused = Focus::Stats,
            KeyCode::Char('c') => self.focused = Focus::Children,
            //KeyCode::Char('h') | KeyCode::Left => self.prev_tab(),
            //KeyCode::Char('l') | KeyCode::Right => self.next_tab(),
            KeyCode::Char('k') | KeyCode::Up => self.prev(),
            KeyCode::Char('j') | KeyCode::Down => self.next(),
            //KeyCode::Char('d') | KeyCode::Delete => self.destroy(),
            _ => {}
        };
    }

    fn prev(&mut self) {
        self.focused = match self.focused {
            Focus::Devices => Focus::Children,
            Focus::Stats => Focus::Devices,
            Focus::Children => Focus::Stats,
        }
    }

    fn next(&mut self) {
        self.focused = match self.focused {
            Focus::Devices => Focus::Stats,
            Focus::Stats => Focus::Children,
            Focus::Children => Focus::Devices,
        }
    }


    fn render_device_list(&self, area:Rect, buf: &mut Buffer) {
        //let block = 
    }
}

/// Implement Widget for &App rather than for App as we would otherwise have to clone or copy the
/// entire app state on every frame. For this example, the app state is small enough that it doesn't
/// matter, but for larger apps this can be a significant performance improvement.
impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
            Constraint::Percentage(50),
            Constraint::Percentage(50)
            ]).split(area);

        let top_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
            Constraint::Percentage(20),
            Constraint::Percentage(80)
            ]).split(layout[0]);

        let block = Block::new().borders(Borders::ALL).title(format!("[T]est")).render(top_layout[0], buf);
        let block2 = Block::new().borders(Borders::ALL).title(format!("[T]est2")).render(top_layout[1], buf);
        let block3 = Block::new().borders(Borders::ALL).title(format!("[T]est3")).render(layout[1], buf);
    }
}

