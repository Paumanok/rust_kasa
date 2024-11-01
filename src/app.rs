use std::time::Duration;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::Color,
    text::{Line, Span},
    widgets::{Block, Tabs, Widget},
    DefaultTerminal, Frame,
};
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct App {
    focused: Focus,
}
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Mode {
    #[default]
    Running,
    Destroy,
    Quit,
}
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
                .draw(|frame| self.draw(frame))
                .wrap_err("terminal.draw")?;
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
        if self.mode == Mode::Destroy {
            destroy::destroy(frame);
        }
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
            KeyCode::Char('h') | KeyCode::Left => self.prev_tab(),
            KeyCode::Char('l') | KeyCode::Right => self.next_tab(),
            KeyCode::Char('k') | KeyCode::Up => self.prev(),
            KeyCode::Char('j') | KeyCode::Down => self.next(),
            KeyCode::Char('d') | KeyCode::Delete => self.destroy(),
            _ => {}
        };
    }

    fn prev(&mut self) {
        //match self.tab {
        //    Tab::About => self.about_tab.prev_row(),
        //    Tab::Recipe => self.recipe_tab.prev(),
        //    Tab::Email => self.email_tab.prev(),
        //    Tab::Traceroute => self.traceroute_tab.prev_row(),
        //    Tab::Weather => self.weather_tab.prev(),
        //}
    }

    fn next(&mut self) {
        //match self.tab {
        //    Tab::About => self.about_tab.next_row(),
        //    Tab::Recipe => self.recipe_tab.next(),
        //    Tab::Email => self.email_tab.next(),
        //    Tab::Traceroute => self.traceroute_tab.next_row(),
        //    Tab::Weather => self.weather_tab.next(),
        //}
    }

    //fn prev_tab(&mut self) {
    //    self.tab = self.tab.prev();
    //}

    //fn next_tab(&mut self) {
    //    self.tab = self.tab.next();
    //}

    fn destroy(&mut self) {
        self.mode = Mode::Destroy;
    }
}

/// Implement Widget for &App rather than for App as we would otherwise have to clone or copy the
/// entire app state on every frame. For this example, the app state is small enough that it doesn't
/// matter, but for larger apps this can be a significant performance improvement.
impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ]);
        let [title_bar, tab, bottom_bar] = vertical.areas(area);

        Block::new().style(THEME.root).render(area, buf);
        self.render_title_bar(title_bar, buf);
        self.render_selected_tab(tab, buf);
        App::render_bottom_bar(bottom_bar, buf);
    }
}

fn draw_main_page(frame: &mut Frame, app: &mut App, area: Rect) {}