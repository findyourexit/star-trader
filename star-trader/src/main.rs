mod app;
mod model;
mod ui;

use std::io;
use std::time::Duration;

use anyhow::Result;
use app::App;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use env_logger::Env;
use ratatui::{Terminal, backend::CrosstermBackend};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    init_logging(&args);

    let mut terminal = setup_terminal()?;
    let result = run(&mut terminal);
    restore_terminal(&mut terminal)?;
    result
}

fn desired_log_level(args: &[String]) -> Option<String> {
    let mut iter = args.iter();
    iter.next(); // skip binary name

    while let Some(arg) = iter.next() {
        if arg == "--log" {
            return Some("info".to_string());
        }
        if let Some(rest) = arg.strip_prefix("--log-level=") {
            return Some(rest.to_string());
        }
        if arg == "--log-level"
            && let Some(next) = iter.next()
        {
            return Some(next.to_string());
        }
    }

    std::env::var("STAR_TRADER_LOG").ok()
}

fn init_logging(args: &[String]) {
    let default_level = desired_log_level(args).unwrap_or_else(|| "warn".to_string());
    let env = Env::default().filter_or("RUST_LOG", default_level);
    let mut builder = env_logger::Builder::from_env(env);
    builder.format_timestamp_secs();
    builder.format_target(false);
    let _ = builder.try_init();
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = App::boot();
    let tick_rate = Duration::from_millis(250);

    loop {
        if event::poll(tick_rate)?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            app.handle_key(key.code);
        }

        app.update_highlights();
        terminal.draw(|frame| ui::render(frame, &app))?;

        app.tick();

        if app.should_quit() {
            break;
        }
    }

    Ok(())
}
