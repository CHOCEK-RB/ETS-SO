use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
};

use sys_probe::{Process, SysProbe};

pub struct App {
    pub sys: SysProbe,
    pub items: Vec<Process>,
    pub table_state: TableState,
    pub exit: bool,
}

impl App {
    pub fn new() -> App {
        let mut app = App {
            sys: SysProbe::new(),
            items: Vec::new(),
            table_state: TableState::default(),
            exit: false,
        };
        app.sys.init();
        app.update_processes();
        app
    }

    pub fn update_processes(&mut self) {
        self.sys.refresh_processes();
        self.items = self.sys.processes.values().cloned().collect();
        self.items.sort_by_key(|p| p.run_time);
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let tick_rate = Duration::from_secs(1);
        let mut last_tick = Instant::now();

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if crossterm::event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') => self.exit = true,
                            KeyCode::Down => self.next_row(),
                            KeyCode::Up => self.previous_row(),
                            _ => {}
                        }
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                self.update_processes();
                last_tick = Instant::now();
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let rects = Layout::default()
            .constraints([Constraint::Percentage(100)])
            .split(frame.area());

        self.render_table(frame, rects[0]);
    }

    fn render_table(&mut self, frame: &mut Frame, area: Rect) {
        let header_style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);
        let selected_style = Style::default().add_modifier(Modifier::REVERSED);

        let header = ["PID", "Name", "Nice", "Prio", "RT Prio", "RAM", "Run Time"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(header_style)
            .height(1);

        let rows = self.items.iter().map(|item| {
            let cells = vec![
                Cell::from(item.pid.to_string()),
                Cell::from(item.name.clone()),
                Cell::from(item.nice.to_string()),
                Cell::from(item.priority.to_string()),
                Cell::from(item.rt_priority.to_string()),
                Cell::from(format!("{:.1} MB", item.ram as f64 / 1024.0 / 1024.0)),
                Cell::from(item.run_time.to_string()),
            ];
            Row::new(cells).height(1)
        });

        let t = Table::new(
            rows,
            [
                Constraint::Length(8),
                Constraint::Min(20),
                Constraint::Length(6),
                Constraint::Length(6),
                Constraint::Length(9),
                Constraint::Length(9),
                Constraint::Length(12),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Monitor de Procesos Rust "),
        )
        .row_highlight_style(selected_style)
        .highlight_symbol(">> ");

        frame.render_stateful_widget(t, area, &mut self.table_state);
    }

    fn next_row(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.items.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn previous_row(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::new().run(&mut terminal);
    ratatui::restore();
    app_result
}
