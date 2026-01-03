use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table, Wrap},
};

use crate::app::{ActivePanel, App, Overlay, Screen};
use crate::model::{Merchandise, StarClass};

const LAUNCH_ART: &str = include_str!("launch_art.txt");

fn footer_lines_needed(width: u16, key_hints: &[(String, String)]) -> u16 {
    let width = width.max(1) as usize;

    let mut total = 0usize;
    for (idx, (key, desc)) in key_hints.iter().enumerate() {
        if idx > 0 {
            total += " • ".len();
        }
        total += key.len();
        total += 1; // space before description
        total += desc.len();
    }

    let lines = total.div_ceil(width);
    lines.clamp(1, 3) as u16
}

fn abbrev(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len > 1 {
        let mut truncated = s.chars().take(max_len - 1).collect::<String>();
        truncated.push('…');
        truncated
    } else {
        s.chars().take(max_len).collect()
    }
}

// -----------------------------------------------------------------------------
// Map helpers
// -----------------------------------------------------------------------------

#[derive(Clone)]
struct CellMark {
    ch: char,
    color: Option<Color>,
}

fn color_for_class(class: StarClass) -> Color {
    match class {
        StarClass::I => Color::Green,
        StarClass::II => Color::Cyan,
        StarClass::III => Color::Yellow,
        StarClass::IV => Color::Magenta,
    }
}

fn ship_color(idx: usize) -> Color {
    const PALETTE: [Color; 6] = [
        Color::LightCyan,
        Color::LightMagenta,
        Color::LightGreen,
        Color::LightYellow,
        Color::White,
        Color::Gray,
    ];
    PALETTE[idx % PALETTE.len()]
}

fn plot_line(
    grid: &mut [Vec<CellMark>],
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    ch: char,
    color: Option<Color>,
) {
    let (mut x0, mut y0, x1, y1) = (x0, y0, x1, y1);
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if !(x0 == x1 && y0 == y1)
            && let Some(row) = grid.get_mut(y0 as usize)
            && let Some(cell) = row.get_mut(x0 as usize)
            && (cell.ch == ' ' || cell.ch == '.')
        {
            cell.ch = ch;
            cell.color = color.or(cell.color);
        }

        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

fn panel_block<'a>(title: &'a str, key: &'a str, active: bool) -> Block<'a> {
    let mut block = Block::default()
        .borders(Borders::ALL)
        .title(format!("{title} [{key}]"));
    if active {
        block = block
            .border_style(Style::default().fg(Color::Cyan))
            .title_alignment(Alignment::Left)
            .title_style(Style::default().add_modifier(Modifier::BOLD));
    } else {
        block = block.border_style(Style::default().fg(Color::DarkGray));
    }
    block
}

// -----------------------------------------------------------------------------
// Main render
// -----------------------------------------------------------------------------

pub fn render(frame: &mut Frame, app: &App) {
    match app.screen() {
        Screen::Launch => render_launch_screen(frame, app),
        Screen::Menu => render_menu(frame, app),
        Screen::Instructions => render_instructions_screen(frame, app),
        Screen::About => render_about_screen(frame, app),
        Screen::Game => render_game(frame, app),
    }
}

fn render_launch_screen(frame: &mut Frame, app: &App) {
    let size = frame.area();
    let key_hints = app.key_hints();
    let footer_lines = footer_lines_needed(size.width, &key_hints);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(footer_lines)].as_ref())
        .split(size);

    let body_sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(5)].as_ref())
        .split(chunks[0]);

    let art = Paragraph::new(LAUNCH_ART)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });
    frame.render_widget(art, body_sections[0]);

    let copy_lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Star Trader",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("Rust TUI remake of the 1974 trading sim."),
        Line::from("Initializing jump lanes and markets..."),
        Line::from("Press any key to skip."),
    ];

    let copy = Paragraph::new(copy_lines)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(copy, body_sections[1]);
    draw_footer(frame, chunks[1], &key_hints);
}

fn render_game(frame: &mut Frame, app: &App) {
    let size = frame.area();
    let key_hints = app.key_hints();
    let footer_lines = footer_lines_needed(size.width, &key_hints);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),  // title bar
                Constraint::Length(5),  // header
                Constraint::Min(0),     // body
                Constraint::Length(12), // events (taller, near bottom)
                Constraint::Length(3),  // input bar
                Constraint::Length(footer_lines),
            ]
            .as_ref(),
        )
        .split(size);

    draw_title(frame, chunks[0]);
    draw_header(frame, chunks[1], app);
    draw_body(frame, chunks[2], app);
    draw_events(frame, chunks[3], app);
    draw_input_bar(frame, chunks[4], app);
    draw_footer(frame, chunks[5], &key_hints);

    if let Some(overlay) = app.overlay() {
        draw_overlay(frame, size, app, overlay);
    }
}

fn render_menu(frame: &mut Frame, app: &App) {
    let size = frame.area();
    let key_hints = app.key_hints();
    let footer_lines = footer_lines_needed(size.width, &key_hints);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(3),
                Constraint::Length(footer_lines),
            ]
            .as_ref(),
        )
        .split(size);

    draw_title(frame, chunks[0]);

    let mut lines: Vec<Line> = Vec::new();
    for (idx, item) in app.menu_items().iter().enumerate() {
        let selected = idx == app.menu_index();
        let prefix = if selected { "> " } else { "  " };
        let label_style = if selected {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let detail_style = Style::default().fg(Color::DarkGray);
        lines.push(Line::from(vec![
            Span::styled(format!("{prefix}{}", item.label()), label_style),
            Span::raw("  "),
            Span::styled(item.detail(), detail_style),
        ]));
    }

    let menu = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .title("Main Menu (Up/Down, Enter)")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(menu, chunks[1]);

    let status = Paragraph::new(app.status_text().to_string())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().title("Status").borders(Borders::ALL));
    frame.render_widget(status, chunks[2]);

    draw_footer(frame, chunks[3], &key_hints);
}

fn render_instructions_screen(frame: &mut Frame, app: &App) {
    let size = frame.area();
    let key_hints = app.key_hints();
    let footer_lines = footer_lines_needed(size.width, &key_hints);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(footer_lines),
            ]
            .as_ref(),
        )
        .split(size);

    draw_title(frame, chunks[0]);
    draw_help_overlay(frame, chunks[1]);
    draw_footer(frame, chunks[2], &key_hints);
}

fn render_about_screen(frame: &mut Frame, app: &App) {
    let size = frame.area();
    let key_hints = app.key_hints();
    let footer_lines = footer_lines_needed(size.width, &key_hints);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(footer_lines),
            ]
            .as_ref(),
        )
        .split(size);

    draw_title(frame, chunks[0]);

    let about_lines = vec![
        Line::from(vec![Span::styled(
            "Star Trader",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("Rust TUI remake of the 1974 BASIC trading sim."),
        Line::from("Fly ships between systems, haul goods, haggle prices, and bank credits."),
        Line::from("Start from the menu, then press ? in-game for controls."),
        Line::from("Saves load from savegame.json; v saves, o loads."),
        Line::from("Build/run from repo root: cargo run -p star-trader."),
    ];

    let about = Paragraph::new(about_lines)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
        .block(Block::default().title("About").borders(Borders::ALL));
    frame.render_widget(about, chunks[1]);

    draw_footer(frame, chunks[2], &key_hints);
}

fn draw_title(frame: &mut Frame, area: Rect) {
    let title = "Star Trader";
    let core = format!(" {title} ");
    let width = area.width.max(core.len() as u16);

    let line_str = if (width as usize) <= core.len() + 2 {
        title.to_string()
    } else {
        let available = width as usize - core.len();
        let left = available / 2;
        let right = available - left;
        format!("{0}{core}{1}", "=".repeat(left), "=".repeat(right))
    };

    let line = Line::from(vec![Span::styled(
        line_str,
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]);

    let para = Paragraph::new(line).alignment(Alignment::Center);
    frame.render_widget(para, area);
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let selected_ship_cash = app.state.selected_ship().map(|s| s.cash).unwrap_or(0);
    let selected_system = app
        .state
        .systems
        .get(app.state.selected_system)
        .map(|s| s.name.as_str())
        .unwrap_or("?");

    let text = vec![
        Line::from(vec![
            Span::styled("Day", Style::default().fg(Color::Gray)),
            Span::raw(" "),
            Span::styled(
                format!("{}", app.state.date.day),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" / "),
            Span::styled(
                format!("{}", app.state.date.year),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  |  Tick "),
            Span::styled(
                format!("{}", app.ticks()),
                Style::default()
                    .fg(Color::LightCyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Bank $", Style::default().fg(Color::Gray)),
            Span::raw(" "),
            Span::styled(format!("{}", app.state.bank_balance), bank_style(app)),
            Span::raw("  |  Ship $ "),
            Span::styled(format!("{}", selected_ship_cash), ship_cash_style(app)),
        ]),
        Line::from(vec![
            Span::styled("Selected System", Style::default().fg(Color::Gray)),
            Span::raw(" "),
            Span::styled(
                selected_system.to_string(),
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    let block = Block::default().borders(Borders::ALL).title("Status");
    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}

fn draw_events(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .events()
        .iter()
        .rev()
        .take(6)
        .map(|e| ListItem::new(e.clone()))
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title("Recent events (newest top)")
            .borders(Borders::ALL),
    );
    frame.render_widget(list, area);
}

fn draw_input_bar(frame: &mut Frame, area: Rect, app: &App) {
    let content = match app.input_mode {
        Some(mode) => format!("Input ({mode:?}): {}", app.input_buffer),
        None => "(no active input)".to_string(),
    };
    let p = Paragraph::new(content)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().title("Input").borders(Borders::ALL));
    frame.render_widget(p, area);
}

fn draw_footer(frame: &mut Frame, area: Rect, key_hints: &[(String, String)]) {
    let mut spans: Vec<Span> = Vec::new();
    let key_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);
    let text_style = Style::default().fg(Color::DarkGray);

    for (idx, (key, desc)) in key_hints.iter().enumerate() {
        if idx > 0 {
            spans.push(Span::styled(" • ", text_style));
        }
        spans.push(Span::styled(key.clone(), key_style));
        spans.push(Span::styled(format!(" {desc}"), text_style));
    }

    let para = Paragraph::new(Line::from(spans))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
        .style(text_style);

    frame.render_widget(para, area);
}

fn draw_body(frame: &mut Frame, area: Rect, app: &App) {
    if area.width < 120 {
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(12),
                Constraint::Length(14),
                Constraint::Min(0),
            ])
            .split(area);

        draw_systems(frame, vertical[0], app);
        draw_market(frame, vertical[1], app);
        draw_ships(frame, vertical[2], app);
    } else {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(area);

        let left = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(12), Constraint::Min(0)].as_ref())
            .split(chunks[0]);

        draw_systems(frame, left[0], app);
        draw_market(frame, left[1], app);
        draw_ships(frame, chunks[1], app);
    }
}

fn draw_systems(frame: &mut Frame, area: Rect, app: &App) {
    let compact = area.width < 64;
    let rows: Vec<Row> = app
        .state
        .systems
        .iter()
        .enumerate()
        .map(|(idx, system)| {
            let base_style = match system.class {
                StarClass::I => Style::default().fg(Color::Green),
                StarClass::II => Style::default().fg(Color::Cyan),
                StarClass::III => Style::default().fg(Color::Yellow),
                StarClass::IV => Style::default().fg(Color::Magenta),
            };

            let mut style = base_style;
            if idx == app.state.selected_system {
                style = style.add_modifier(Modifier::REVERSED);
            }

            let name = if compact {
                abbrev(&system.name, 10)
            } else {
                system.name.clone()
            };

            Row::new(vec![
                Cell::from(name).style(style),
                Cell::from(format!("{:>2}", system.class.raw_score())),
                Cell::from(format!("({:>3},{:>3})", system.coords.0, system.coords.1)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        &[
            Constraint::Length(if compact { 12 } else { 14 }),
            Constraint::Length(5),
            Constraint::Length(if compact { 12 } else { 14 }),
        ],
    )
    .header(
        Row::new(vec!["Name", "Cls", "Coords"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(panel_block(
        "Star Systems",
        "1",
        app.active_panel == ActivePanel::Systems,
    ));

    frame.render_widget(table, area);
}

fn draw_market(frame: &mut Frame, area: Rect, app: &App) {
    let Some(system) = app.state.systems.get(app.state.selected_system) else {
        frame.render_widget(Paragraph::new("No systems."), area);
        return;
    };

    let compact = area.width < 60;

    let rows: Vec<Row> = Merchandise::ALL
        .iter()
        .map(|good| {
            let idx = good.idx();
            let stock = system.stock[idx];
            let price = system.prices[idx];
            let (stock_style, price_style) = market_styles(app, idx);
            let style = if stock < 0 {
                Style::default().fg(Color::Yellow)
            } else if stock > 0 {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };

            let mut row = Row::new(vec![
                Cell::from(good.code()),
                Cell::from(format!("{:>6}", stock)).style(style.patch(stock_style)),
                Cell::from(format!("${:>6}", price)).style(price_style),
            ]);
            if idx == app.state.selected_good {
                row = row.style(Style::default().add_modifier(Modifier::REVERSED));
            }
            row
        })
        .collect();

    let market_title = format!("Market @ {}", system.name);

    let table = Table::new(
        rows,
        &[
            Constraint::Length(6),
            Constraint::Length(if compact { 7 } else { 8 }),
            Constraint::Length(if compact { 9 } else { 10 }),
        ],
    )
    .header(
        Row::new(vec!["Good", "Stock", "Price"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(panel_block(
        &market_title,
        "2",
        app.active_panel == ActivePanel::Market,
    ));

    frame.render_widget(table, area);
}

fn draw_ships(frame: &mut Frame, area: Rect, app: &App) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(9), Constraint::Min(0)].as_ref())
        .split(area);

    let compact = area.width < 80;
    let ultra_compact = area.width < 60;

    let rows: Vec<Row> = app
        .state
        .ships
        .iter()
        .enumerate()
        .map(|(idx, ship)| {
            let loc_name = app
                .state
                .systems
                .get(ship.location)
                .map(|s| s.name.as_str())
                .unwrap_or("?");
            let dest_name = app
                .state
                .systems
                .get(ship.destination)
                .map(|s| s.name.as_str())
                .unwrap_or("?");
            let (cash_style, ton_style) = ship_styles(app, idx);

            let mut row = if ultra_compact {
                let route = format!("{}→{}", abbrev(loc_name, 4), abbrev(dest_name, 4));
                Row::new(vec![
                    Cell::from(format!("{} {}", abbrev(&ship.name, 8), route)),
                    Cell::from(format!("${}", ship.cash)).style(cash_style),
                    Cell::from(format!("{}t", ship.net_tonnage)).style(ton_style),
                ])
            } else {
                let route = if compact {
                    format!("{}→{}", abbrev(loc_name, 6), abbrev(dest_name, 6))
                } else {
                    format!("{} → {}", loc_name, dest_name)
                };

                Row::new(vec![
                    Cell::from(ship.name.clone()),
                    Cell::from(route),
                    Cell::from(format!("{:>3} / {}", ship.eta.day, ship.eta.year)),
                    Cell::from(format!("${}", ship.cash)).style(cash_style),
                    Cell::from(format!("{}t", ship.net_tonnage)).style(ton_style),
                ])
            };

            if idx == app.state.selected_ship {
                row = row.style(Style::default().add_modifier(Modifier::REVERSED));
            }
            row
        })
        .collect();

    let (widths, header): (Vec<Constraint>, Vec<&str>) = if ultra_compact {
        (
            vec![
                Constraint::Length(18),
                Constraint::Length(12),
                Constraint::Length(8),
            ],
            vec!["Ship/Route", "Cash", "Tons"],
        )
    } else {
        (
            vec![
                Constraint::Length(if compact { 10 } else { 12 }),
                Constraint::Length(if compact { 16 } else { 20 }),
                Constraint::Length(12),
                Constraint::Length(12),
                Constraint::Length(8),
            ],
            vec!["Name", "Route", "ETA", "Cash", "Tons"],
        )
    };

    let table = Table::new(rows, widths)
        .header(Row::new(header).style(Style::default().add_modifier(Modifier::BOLD)))
        .block(panel_block(
            "Ships",
            "3",
            app.active_panel == ActivePanel::Ships,
        ));

    frame.render_widget(table, layout[0]);

    let (cargo_title, cargo_lines): (String, Vec<Line>) =
        if let Some(ship) = app.state.selected_ship() {
            let lines = Merchandise::ALL
                .iter()
                .map(|good| {
                    let qty = ship.cargo[good.idx()];
                    let style = cargo_style(app, good.idx());
                    Line::from(Span::styled(
                        format!("{:>4} {:>5}", good.code(), qty),
                        style,
                    ))
                })
                .collect();
            (format!("Selected Ship's Cargo ({})", ship.name), lines)
        } else {
            (
                "Selected Ship's Cargo".to_string(),
                vec![Line::from("No ships loaded")],
            )
        };

    let cargo = Paragraph::new(cargo_lines).block(
        Block::default()
            .title(cargo_title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(cargo, layout[1]);
}

fn draw_overlay(frame: &mut Frame, area: Rect, app: &App, overlay: Overlay) {
    match overlay {
        Overlay::Map => draw_map_overlay(frame, area, app),
        Overlay::Report => draw_report_overlay(frame, area, app),
        Overlay::Help => draw_help_overlay(frame, area),
    }
}

fn draw_map_overlay(frame: &mut Frame, area: Rect, app: &App) {
    frame.render_widget(Clear, area);

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(68), Constraint::Percentage(32)].as_ref())
        .split(area);

    let map_block = Block::default()
        .title("Star Map (Esc to close — Enter sets course)")
        .borders(Borders::ALL);
    let map_inner = map_block.inner(layout[0]);
    let map_lines = map_lines(app, map_inner);
    let map_para = Paragraph::new(map_lines).block(map_block);
    frame.render_widget(map_para, layout[0]);

    let info_block = Block::default().title("Details").borders(Borders::ALL);
    let info_lines = map_info_lines(app);
    let info_para = Paragraph::new(info_lines).block(info_block);
    frame.render_widget(info_para, layout[1]);
}

#[allow(clippy::needless_range_loop)]
fn map_lines(app: &App, area: Rect) -> Vec<Line<'static>> {
    if app.state.systems.is_empty() {
        return vec![Line::from("No systems loaded.".to_string())];
    }

    let map_w = area.width as usize;
    let map_h = area.height as usize;
    if map_w < 20 || map_h < 8 {
        return vec![Line::from("Map area too small.".to_string())];
    }

    let left_margin = 4usize; // room for y labels
    let bottom_margin = 2usize; // room for x labels
    let grid_w = map_w.saturating_sub(left_margin);
    let grid_h = map_h.saturating_sub(bottom_margin);

    let map_extent: f32 = 100.0; // span from -50..50
    let scale_x = (grid_w as f32 - 1.0) / map_extent.max(1.0);
    let scale_y = (grid_h as f32 - 1.0) / map_extent.max(1.0);

    let mut grid = vec![
        vec![
            CellMark {
                ch: ' ',
                color: None
            };
            grid_w
        ];
        grid_h
    ];

    let project = |coord: i16, scale: f32| -> usize {
        let shifted = (coord as f32 + 50.0).clamp(0.0, 100.0);
        (shifted * scale).round() as usize
    };

    // 10-ly cross-lines (ASCII)
    for gx in (0..=100).step_by(10) {
        let col = project(gx - 50, scale_x);
        for row in 0..grid_h {
            let cell = &mut grid[row][col.min(grid_w - 1)];
            cell.ch = match cell.ch {
                ' ' => '|',
                '-' => '+',
                _ => cell.ch,
            };
            cell.color = Some(Color::DarkGray);
        }
    }
    for gy in (0..=100).step_by(10) {
        let row = grid_h
            .saturating_sub(1)
            .saturating_sub(project(gy - 50, scale_y).min(grid_h.saturating_sub(1)));
        for col in 0..grid_w {
            let cell = &mut grid[row][col];
            cell.ch = match cell.ch {
                '|' => '+',
                ' ' => '-',
                _ => cell.ch,
            };
            cell.color = Some(Color::DarkGray);
        }
    }

    // Systems
    for (idx, system) in app.state.systems.iter().enumerate() {
        let sx = project(system.coords.0, scale_x);
        let sy = grid_h
            .saturating_sub(1)
            .saturating_sub(project(system.coords.1, scale_y).min(grid_h.saturating_sub(1)));
        let glyph = if idx == app.state.selected_system {
            'X'
        } else {
            'O'
        };
        let color = if idx == app.state.selected_system {
            Some(Color::White)
        } else {
            Some(color_for_class(system.class))
        };
        let cell = &mut grid[sy.min(grid_h - 1)][sx.min(grid_w - 1)];
        cell.ch = glyph;
        cell.color = color;
    }

    // Selected ship destination ghost marker
    if let Some(ship) = app.state.selected_ship()
        && let Some(dest) = app.state.systems.get(ship.destination)
    {
        let dx = project(dest.coords.0, scale_x);
        let dy = grid_h
            .saturating_sub(1)
            .saturating_sub(project(dest.coords.1, scale_y).min(grid_h.saturating_sub(1)));
        let cell = &mut grid[dy.min(grid_h - 1)][dx.min(grid_w - 1)];
        if !cell.ch.is_ascii_alphabetic() {
            cell.ch = '>';
            cell.color = Some(Color::Blue);
        }
    }

    // Route lines for each ship
    for ship in &app.state.ships {
        let Some(origin) = app.state.systems.get(ship.location) else {
            continue;
        };
        let Some(dest) = app.state.systems.get(ship.destination) else {
            continue;
        };
        let x0 = project(origin.coords.0, scale_x).min(grid_w - 1) as i32;
        let y0 = (grid_h
            .saturating_sub(1)
            .saturating_sub(project(origin.coords.1, scale_y).min(grid_h.saturating_sub(1)))
            .min(grid_h - 1)) as i32;
        let x1 = project(dest.coords.0, scale_x).min(grid_w - 1) as i32;
        let y1 = (grid_h
            .saturating_sub(1)
            .saturating_sub(project(dest.coords.1, scale_y).min(grid_h.saturating_sub(1)))
            .min(grid_h - 1)) as i32;
        plot_line(&mut grid, x0, y0, x1, y1, '.', Some(Color::DarkGray));
    }

    // Ships
    for (idx, ship) in app.state.ships.iter().enumerate() {
        if let Some(system) = app.state.systems.get(ship.location) {
            let sx = project(system.coords.0, scale_x);
            let sy = grid_h
                .saturating_sub(1)
                .saturating_sub(project(system.coords.1, scale_y).min(grid_h.saturating_sub(1)));
            let digit = char::from_digit((idx + 1) as u32, 10).unwrap_or('*');
            let cell = &mut grid[sy.min(grid_h - 1)][sx.min(grid_w - 1)];
            let s_color = ship_color(idx);
            if cell.ch == 'O' || cell.ch == 'X' {
                cell.ch = '@';
                cell.color = Some(s_color);
            } else if !cell.ch.is_ascii_alphabetic() {
                cell.ch = digit;
                cell.color = Some(s_color);
            }
        }
    }

    let mut lines: Vec<Line> = Vec::new();
    for row in 0..grid_h {
        let y_val =
            ((grid_h - 1 - row) as f32 / (grid_h - 1).max(1) as f32 * 100.0).round() as i32 - 50;
        let y_label = if y_val % 20 == 0 {
            format!("{:>3} ", y_val)
        } else {
            "    ".to_string()
        };
        let mut spans: Vec<Span> = Vec::with_capacity(map_w);
        spans.push(Span::raw(y_label));
        for cell in &grid[row] {
            let mut s = Span::raw(cell.ch.to_string());
            if let Some(color) = cell.color {
                s = s.style(Style::default().fg(color));
            }
            spans.push(s);
        }
        lines.push(Line::from(spans));
    }

    // X axis labels
    let mut x_label_spans: Vec<Span> = Vec::with_capacity(map_w);
    let mut x_ticks: Vec<char> = vec![' '; map_w];
    for tick in (-50..=50).step_by(20) {
        let col = left_margin + project(tick as i16, scale_x);
        let label = format!("{:>3}", tick);
        let mut start = col.saturating_sub(2);
        if start + label.len() >= map_w {
            start = map_w.saturating_sub(label.len());
        }
        for (i, c) in label.chars().enumerate() {
            let idx = start.saturating_add(i).min(map_w - 1);
            x_ticks[idx] = c;
        }
    }
    for ch in x_ticks {
        x_label_spans.push(Span::raw(ch.to_string()));
    }
    lines.push(Line::from(x_label_spans));

    let mut axis_line: Vec<Span> = Vec::with_capacity(map_w);
    let mut axis_chars = vec![' '; map_w];
    for tick in (-50..=50).step_by(10) {
        let col = left_margin + project(tick as i16, scale_x);
        axis_chars[col.min(map_w - 1)] = '|';
    }
    for ch in axis_chars {
        axis_line.push(Span::styled(
            ch.to_string(),
            Style::default().fg(Color::DarkGray),
        ));
    }
    lines.push(Line::from(axis_line));

    lines.push(Line::from(vec![
        Span::raw("Legend: "),
        Span::styled("O", Style::default().fg(Color::Green)),
        Span::raw(" system, "),
        Span::styled("X", Style::default().fg(Color::White)),
        Span::raw(" selected, "),
        Span::styled("@", Style::default().fg(Color::Cyan)),
        Span::raw(" system+ship, digits ships, "),
        Span::styled(">", Style::default().fg(Color::Blue)),
        Span::raw(" destination, '.' route, '+' grid 10 ly."),
    ]));

    lines
}

fn map_info_lines(app: &App) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    let label_style = Style::default().fg(Color::Gray);
    let value_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let key_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);

    if let Some(system) = app.state.selected_system() {
        lines.push(Line::from(vec![
            Span::styled("System", label_style),
            Span::raw(": "),
            Span::styled(system.name.clone(), value_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Class", label_style),
            Span::raw(": "),
            Span::styled(system.class.label(), value_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Coords", label_style),
            Span::raw(": "),
            Span::styled(
                format!("({}, {})", system.coords.0, system.coords.1),
                value_style,
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Last update", label_style),
            Span::raw(": day "),
            Span::styled(format!("{}", system.last_update.day), value_style),
            Span::raw(" / "),
            Span::styled(format!("{}", system.last_update.year), value_style),
        ]));
        let good = Merchandise::ALL[app.state.selected_good];
        lines.push(Line::from(vec![
            Span::styled(good.code(), label_style),
            Span::raw(" stock: "),
            Span::styled(format!("{}", system.stock[good.idx()]), value_style),
            Span::raw("  price: $"),
            Span::styled(format!("{}", system.prices[good.idx()]), value_style),
        ]));
    }

    lines.push(Line::from(String::new()));

    if let Some(ship) = app.state.selected_ship() {
        lines.push(Line::from(vec![
            Span::styled("Ship", label_style),
            Span::raw(" "),
            Span::styled(format!("{}", app.state.selected_ship + 1), value_style),
            Span::raw(": "),
            Span::styled(ship.name.clone(), value_style),
        ]));
        let loc = app
            .state
            .systems
            .get(ship.location)
            .map(|s| s.name.clone())
            .unwrap_or_else(|| "?".to_string());
        let dest = app
            .state
            .systems
            .get(ship.destination)
            .map(|s| s.name.clone())
            .unwrap_or_else(|| "?".to_string());
        lines.push(Line::from(vec![
            Span::styled("At", label_style),
            Span::raw(" "),
            Span::styled(loc, value_style),
            Span::raw(" → "),
            Span::styled(dest, value_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("ETA", label_style),
            Span::raw(" day "),
            Span::styled(format!("{}", ship.eta.day), value_style),
            Span::raw(" / "),
            Span::styled(format!("{}", ship.eta.year), value_style),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Cash $", label_style),
            Span::raw(" "),
            Span::styled(format!("{}", ship.cash), value_style),
            Span::raw("  |  Cargo tons "),
            Span::styled(format!("{}", ship.net_tonnage), value_style),
        ]));
        let cargo_summary: String = Merchandise::ALL
            .iter()
            .map(|g| format!("{}:{}", g.code(), ship.cargo[g.idx()]))
            .collect::<Vec<_>>()
            .join(" ");
        lines.push(Line::from(vec![
            Span::styled("Cargo", label_style),
            Span::raw(": "),
            Span::styled(cargo_summary, value_style),
        ]));
    }

    lines.push(Line::from(String::new()));
    lines.push(Line::from(vec![
        Span::styled("Keys", label_style),
        Span::raw(" "),
        Span::styled("arrows", key_style),
        Span::raw(" move focus, "),
        Span::styled("Tab", key_style),
        Span::raw(" ships, "),
        Span::styled("Enter", key_style),
        Span::raw(" set course"),
    ]));
    lines.push(Line::from(vec![
        Span::raw("   "),
        Span::styled("r", key_style),
        Span::raw(" recenter, "),
        Span::styled("Esc", key_style),
        Span::raw(" close"),
    ]));

    lines
}

fn draw_report_overlay(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title("Trade Report (arrows move, Esc closes)")
        .borders(Borders::ALL);
    frame.render_widget(Clear, area);

    if app.state.systems.is_empty() {
        let paragraph = Paragraph::new("No systems loaded").block(block);
        frame.render_widget(paragraph, area);
        return;
    }

    let mut rows: Vec<Row> = Vec::new();
    for system in &app.state.systems {
        for good in Merchandise::ALL.iter() {
            let stock = system.stock[good.idx()];
            let price = system.prices[good.idx()];
            let stock_style = if stock < 0 {
                Style::default().fg(Color::Yellow)
            } else if stock > 0 {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };

            rows.push(
                Row::new(vec![
                    Cell::from(system.name.clone()),
                    Cell::from(class_short(system.class)),
                    Cell::from(good.code()),
                    Cell::from(format!("{:>6}", stock)).style(stock_style),
                    Cell::from(format!("${:>6}", price)),
                ])
                .style(if system.id == app.state.selected_system {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                }),
            );
        }
    }

    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Length(10),
        ],
    )
    .header(
        Row::new(vec!["System", "Class", "Good", "Stock", "Price"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(block);

    frame.render_widget(table, area);
}

fn draw_help_overlay(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("Help (Esc to close)")
        .borders(Borders::ALL);
    frame.render_widget(Clear, area);

    let key_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let text_style = Style::default().fg(Color::Gray);

    let lines = vec![
        Line::from(vec![Span::styled(
            "Navigation",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("Arrows", key_style),
            Span::styled(" move ships/systems; ", text_style),
            Span::styled("[ / ]", key_style),
            Span::styled(" cycle goods", text_style),
        ]),
        Line::from(vec![
            Span::styled("1/2/3", key_style),
            Span::styled(" focus Systems/Market/Ships panels", text_style),
        ]),
        Line::from(vec![
            Span::styled("Tab / Shift-Tab", key_style),
            Span::styled(" cycle ships", text_style),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Travel",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("g", key_style),
            Span::styled(" set course to selected system", text_style),
        ]),
        Line::from(vec![
            Span::styled("d", key_style),
            Span::styled(" pick destination with arrows, ", text_style),
            Span::styled("Enter", key_style),
            Span::styled(" to set", text_style),
        ]),
        Line::from(vec![
            Span::styled("p", key_style),
            Span::styled(" type system #; ", text_style),
            Span::styled("n", key_style),
            Span::styled(" advance to next arrival", text_style),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Trading",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("b", key_style),
            Span::styled(" buy, ", text_style),
            Span::styled("s", key_style),
            Span::styled(" sell (enter qty then haggle price)", text_style),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Banking",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("k", key_style),
            Span::styled(" deposit, ", text_style),
            Span::styled("l", key_style),
            Span::styled(" withdraw (Class I/II only)", text_style),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Overlays",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("m", key_style),
            Span::styled(" map, ", text_style),
            Span::styled("t", key_style),
            Span::styled(" trade report, ", text_style),
            Span::styled("?", key_style),
            Span::styled(" or ", text_style),
            Span::styled("h", key_style),
            Span::styled(" help", text_style),
        ]),
        Line::from(vec![
            Span::styled(
                "Map overlay:",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(" arrows move, ", text_style),
            Span::styled("Tab/Shift-Tab", key_style),
            Span::styled(" ships", text_style),
        ]),
        Line::from(vec![
            Span::styled(" ", text_style),
            Span::styled("Enter", key_style),
            Span::styled(" sets course, ", text_style),
            Span::styled("r", key_style),
            Span::styled(" recenters, ", text_style),
            Span::styled("Esc", key_style),
            Span::styled(" closes", text_style),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Misc",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("r", key_style),
            Span::styled(" reset demo, ", text_style),
            Span::styled("q", key_style),
            Span::styled(" quit", text_style),
        ]),
        Line::from(vec![
            Span::styled("v", key_style),
            Span::styled(" save, ", text_style),
            Span::styled("o", key_style),
            Span::styled(" load", text_style),
        ]),
        Line::from(vec![
            Span::styled("Esc", key_style),
            Span::styled(" cancels input or closes overlays", text_style),
        ]),
    ];

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}

fn class_short(class: StarClass) -> &'static str {
    match class {
        StarClass::I => "I",
        StarClass::II => "II",
        StarClass::III => "III",
        StarClass::IV => "IV",
    }
}

fn bank_style(app: &App) -> Style {
    delta_style(app.bank_highlight(), app.ticks())
}

fn ship_cash_style(app: &App) -> Style {
    delta_style(
        app.ship_cash_highlight(app.state.selected_ship),
        app.ticks(),
    )
}

fn market_styles(app: &App, idx: usize) -> (Style, Style) {
    let sys_idx = app.state.selected_system;
    let stock_style = delta_style(app.stock_highlight(sys_idx, idx), app.ticks());
    let price_style = delta_style(app.price_highlight(sys_idx, idx), app.ticks());
    (stock_style, price_style)
}

fn ship_styles(app: &App, idx: usize) -> (Style, Style) {
    let cash_style = delta_style(app.ship_cash_highlight(idx), app.ticks());
    let ton_style = delta_style(app.ship_tonnage_highlight(idx), app.ticks());
    (cash_style, ton_style)
}

fn cargo_style(app: &App, good_idx: usize) -> Style {
    delta_style(
        app.cargo_highlight(app.state.selected_ship, good_idx),
        app.ticks(),
    )
}

fn delta_style(delta: Option<i64>, now: u64) -> Style {
    if let Some(d) = delta {
        let blink_on = (now / 2).is_multiple_of(2);
        if blink_on {
            if d > 0 {
                return Style::default().fg(Color::Green);
            } else {
                return Style::default().fg(Color::Red);
            }
        }
    }
    Style::default()
}
