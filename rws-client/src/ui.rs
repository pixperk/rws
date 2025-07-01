use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Terminal,
};
use std::io;
use tokio::sync::mpsc;

use crate::{app::App, client};

pub async fn run(app: &mut App) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (ui_tx, mut ui_rx) = mpsc::unbounded_channel();
    let (ws_tx, ws_rx) = mpsc::unbounded_channel();
    app.tx = Some(ws_tx.clone());

    let username = app.username.clone();
    let server_url = app.server_url.clone();
    tokio::spawn(async move {
        if let Err(e) = client::connect_and_handle(username, server_url, ui_tx, ws_rx).await {
            eprintln!("WebSocket error: {}", e);
        }
    });

    loop {
        terminal.draw(|f| draw_ui(f, app))?;

        while let Ok(msg) = ui_rx.try_recv() {
            app.add_message(msg, true);
        }

        if event::poll(std::time::Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    match key_event.code {
                        KeyCode::Char('q') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.quit();
                        }
                        KeyCode::Enter => {
                            if !app.input.is_empty() {
                                if let Some(tx) = &app.tx {
                                    let _ = tx.send(app.input.clone());
                                }
                                app.clear_input();
                            }
                        }
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}


fn draw_ui(f: &mut ratatui::Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
        .split(f.size());

    let title = Paragraph::new(format!(
        "🚀 RWS Chat - {} {}",
        app.username,
        app.current_room
            .as_ref()
            .map(|r| format!("(Room: {})", r))
            .unwrap_or_default()
    ))
    .block(Block::default().borders(Borders::ALL).title("Real-time WebSocket Chat"))
    .style(Style::default().fg(Color::Cyan));
    f.render_widget(title, chunks[0]);

    let messages: Vec<ListItem> = app
        .messages
        .iter()
        .map(|m| {
            let style = if m.is_system {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{} ", m.timestamp.format("%H:%M:%S")),
                    Style::default().fg(Color::Gray),
                ),
                Span::styled(&m.content, style),
            ]))
        })
        .collect();

    let messages_list = List::new(messages)
        .block(Block::default().borders(Borders::ALL).title("Messages"))
        .style(Style::default().fg(Color::White));
    f.render_widget(messages_list, chunks[1]);

    let input = Paragraph::new(app.input.as_str())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Type your message (Ctrl+Q to quit, /create <name> for room)"),
        )
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(input, chunks[2]);

    f.set_cursor(chunks[2].x + app.input.len() as u16 + 1, chunks[2].y + 1);
}
