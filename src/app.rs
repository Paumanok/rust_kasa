use anyhow::{anyhow, Error, Result};
use crossterm::event;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Direction, Layout, Rect},
    style::Color,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListState, Paragraph, StatefulWidget, Widget},
    DefaultTerminal, Frame,
};
use std::time::Duration;

use rust_kasa::device;

#[derive(Default, Clone)]
pub struct App {
    focused: Focus,
    mode: Mode,
    devices: Devices,
}

#[derive(Default, Clone)]
pub struct Devices {
    devices: Vec<device::Device>,
    state: ListState,
    child_state: ListState,
}


impl Devices {
    pub fn new() -> Self {
        Self {
            devices: vec![],
            state: ListState::default(),
            child_state: ListState::default(),
        }
    }

    pub fn prev(&mut self) {
        self.state.select_previous();
    }
    pub fn next(&mut self) {
        self.state.select_next();
    }
    pub fn prev_child(&mut self) {
        self.child_state.select_previous();
    }
    pub fn next_child(&mut self) {
        self.child_state.select_next();
    }
    fn render_device_list(&mut self, area: Rect, buf: &mut Buffer) {
        let items: Vec<String> = self.devices.iter().map(|i| i.ip_addr.clone()).collect();
        let list = List::new(items)
            .block(Block::bordered().title("[D]evices"))
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true);
        StatefulWidget::render(list, area, buf, &mut self.state);
    }

    fn render_device_info(&mut self, area: Rect, buf: &mut Buffer) {
        let selected = if let Some(i) = self.state.selected() {
            let selected_device = &self.devices[i];
            if let Some(si) = selected_device.sysinfo() {
                format!("Name: {:}", si.alias)
            } else {
                "failed1".to_string()
            }
        } else {
            "failed".to_string()
        };

        let block = Block::new().borders(Borders::ALL).title("[I]nfo");

        let paragraph = Paragraph::new(selected).block(block).render(area, buf);
    }

    fn render_children(&mut self, area: Rect, buf :&mut Buffer) {
        let block = Block::new().borders(Borders::ALL).title("[C]hildren");
        
        if let Some(p) = self.state.selected() {
            let selected_device = &self.devices[p];
            if let Some(si) = selected_device.sysinfo() {
                if si.child_num > 0 {
                    if let Some(children) = selected_device.children() {
                
                        let items: Vec<String> = children.iter().map(|plug| plug.alias.clone()).collect();
                        let list = List::new(items)
                                .block(block)
                                .highlight_symbol(">>")
                                .repeat_highlight_symbol(true);
                            StatefulWidget::render(list, area, buf, &mut self.child_state);
                            };
            } else {
               let paragraph = Paragraph::new(format!("{:} Outlet", si.alias)).block(block).render(area, buf);
            };
        };
        }
    }

    fn toggle_selected_child_outlet(&mut self) {
        if let Some(p) = self.state.selected() {
            let selected_device = &self.devices[p];
            if let Some(si) = selected_device.sysinfo() {
                if si.child_num > 0 {
                    selected_device.toggle_relay_by_id(0);
                } else {
                    selected_device.toggle_single_relay();

                }
            }
        }
    }
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
        if self.devices.devices.is_empty() {
            let devs = device::discover_multiple();
            if let Ok(devices) = devs {
                self.devices.devices = devices;
                self.devices.state.select(Some(0));
            }
        }
        while self.is_running() {
            terminal.draw(|frame| self.draw(frame))?; //this needs an anyhow or color_eyre wrap
            self.handle_events()?;
        }
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.mode != Mode::Quit
    }

    /// Draw a single frame of the app.
    fn draw(&mut self, frame: &mut Frame) {
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
        //interaction with overall app
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.mode = Mode::Quit,
            KeyCode::Char('d') => self.focused = Focus::Devices,
            KeyCode::Char('s') => self.focused = Focus::Stats,
            KeyCode::Char('c') => self.focused = Focus::Children,
            //KeyCode::Char('h') | KeyCode::Left => self.prev_tab(),
            //KeyCode::Char('l') | KeyCode::Right => self.next_tab(),
            KeyCode::Char('k') => self.prev(),
            KeyCode::Char('j') => self.next(),
            //KeyCode::Char('d') | KeyCode::Delete => self.destroy(),
            _ => {}
        };
        //interaction with focused windows
        match self.focused {
            Focus::Devices => match key.code {
                KeyCode::Up => self.devices.prev(),
                KeyCode::Down => self.devices.next(),
                _ => {}
            },
            Focus::Children => match key.code {
                KeyCode::Char(' ') => self.devices.toggle_selected_child_outlet(),
                KeyCode::Up => self.devices.prev_child(),
                KeyCode::Down => self.devices.next_child(),
                _ => {}
            },
            Focus::Stats => match key.code {
                _ => {}
            },
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
}

/// Implement Widget for &App rather than for App as we would otherwise have to clone or copy the
/// entire app state on every frame. For this example, the app state is small enough that it doesn't
/// matter, but for larger apps this can be a significant performance improvement.
impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let top_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(layout[0]);
        self.devices.render_device_list(top_layout[0], buf);
        self.devices.render_device_info(top_layout[1], buf);
        self.devices.render_children(layout[1], buf);
        //let block = Block::new()
        //    .borders(Borders::ALL)
        //    .title(format!("[T]est"))
        //    .render(top_layout[0], buf);

        //let block2 = Block::new()
        //    .borders(Borders::ALL)
        //    .title(format!("[T]est2"))
        //    .render(top_layout[1], buf);
        //let block3 = Block::new()
        //    .borders(Borders::ALL)
        //    .title(format!("[T]est3"))
        //    .render(layout[1], buf);
    }
}
