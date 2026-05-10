use std::{
    io::{self, IsTerminal},
    time::Duration,
};

use color_eyre::eyre::{eyre, Result};
use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode},
    execute,
    style::ResetColor,
    terminal::{
        disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::data::{load_dashboard_data, DashboardData, TimeRange};

use super::{app::App, render};

pub fn run(data: DashboardData, range: TimeRange, interval_secs: u64) -> Result<()> {
    if !io::stdout().is_terminal() || !io::stdin().is_terminal() {
        return Err(eyre!("--live requires an interactive terminal"));
    }

    let (width, height) = size()?;
    if width < render::MIN_WIDTH || height < render::MIN_HEIGHT {
        return Err(eyre!(
            "terminal is {width}x{height}; tokenburn live dashboard requires at least {}x{}. Resize the window or run without --live for plain output.",
            render::MIN_WIDTH,
            render::MIN_HEIGHT
        ));
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        ResetColor,
        Clear(ClearType::All),
        MoveTo(0, 0)
    )?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    let result = run_loop(&mut terminal, App::new(data, range, interval_secs));

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, mut app: App) -> Result<()> {
    while !app.quit {
        terminal.draw(|frame| {
            render::render(
                frame,
                &app.data,
                &app.range,
                true,
                app.paused,
                app.show_help,
                Some(app.active_tab),
                app.show_claude_subagents,
            )
        })?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => app.quit = true,
                    KeyCode::Char('r') => app.cycle_range(),
                    KeyCode::Char('s') => app.toggle_claude_subagents(),
                    KeyCode::Char('p') => app.paused = !app.paused,
                    KeyCode::Char('?') => app.show_help = !app.show_help,
                    KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => app.next_tab(),
                    KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => app.prev_tab(),
                    _ => {}
                }
            }
        }

        if app.refresh_due() {
            app.data = load_dashboard_data(&app.range)?;
            app.last_refresh = std::time::Instant::now();
        }
    }

    Ok(())
}
