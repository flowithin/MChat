use crossterm::terminal;
use ratatui::text::Line;
use std::fs;
use std::os::fd::AsRawFd;
//use std::sync::mpsc::{channel, Receiver, Sender};
use std::{
    fs::File,
    io::{self, stdout, Read, Stdout, Write},
    path::Path,
    thread,
    thread::sleep,
    time::Duration,
};

use std::net::TcpStream;

use crossterm::{
    event::{
        self, poll, read, DisableBracketedPaste, DisableFocusChange, DisableMouseCapture,
        EnableBracketedPaste, EnableFocusChange, EnableMouseCapture, Event, KeyCode, KeyEvent,
        KeyModifiers, MouseEvent,
    },
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, window_size, EnterAlternateScreen, LeaveAlternateScreen,
        WindowSize,
    },
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};

struct App {
    chat: Vec<String>,
    prompt: String,
    scroll: usize,
}

impl App {
    fn new() -> Self {
        App {
            chat: Vec::new(),
            prompt: String::new(),
            scroll: 0,
        }
    }

    fn add_message(&mut self, message: String) {
        self.chat.push(message);
        // Auto-scroll to the latest message
        self.scroll = self.chat.len().saturating_sub(1);
    }

    fn render<B: Write>(&self, terminal: &mut Terminal<CrosstermBackend<B>>) -> io::Result<()> {
        terminal.draw(|f| {
            let size = f.size();

            // Define layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(size.height.saturating_sub(5)),
                        Constraint::Length(1),
                        Constraint::Length(4),
                    ]
                    .as_ref(),
                )
                .split(size);

            // Render chat window
            let chat_block = Block::default()
                .title(" Chat ")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Black).fg(Color::White));
            let chat_text = self
                .chat
                .iter()
                .enumerate()
                .map(|(i, line)| {
                    if i == self.scroll {
                        Line::from(vec![Span::styled(
                            line,
                            Style::default()
                                .fg(Color::LightCyan)
                                .add_modifier(Modifier::BOLD),
                        )])
                    } else {
                        Line::from(line.as_str())
                    }
                })
                .collect::<Vec<_>>();
            let chat_paragraph = Paragraph::new(chat_text)
                .block(chat_block)
                .wrap(Wrap { trim: true })
                .scroll((self.scroll.saturating_sub(20).try_into().unwrap_or(0), 0));
            f.render_widget(chat_paragraph, chunks[0]);

            // Render input area
            let input_block = Block::default()
                .borders(Borders::ALL)
                .title(" Input ")
                .style(Style::default().bg(Color::Black).fg(Color::White));
            let input_paragraph = Paragraph::new(self.prompt.as_str())
                .block(input_block)
                .style(Style::default().fg(Color::LightGreen))
                .wrap(Wrap { trim: true });
            f.render_widget(input_paragraph, chunks[2]);

            // Render footer
            let footer = Paragraph::new("Press Ctrl+C to exit")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Gray));
            f.render_widget(footer, chunks[1]);
        })?;
        Ok(())
    }
}

fn main() -> io::Result<()> {
    // Setup terminal
    let mut stream = TcpStream::connect("0.0.0.0:6969").expect("Couldn't connect to the server...");
    stream
        .set_nonblocking(true)
        .expect("set_nonblocking call failed");
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    // Initialize app
    let mut app = App::new();

    // Load logo
    let mut buffer = String::new();
    let logo = if Path::new("mchat.logo").exists() {
        File::open("mchat.logo")?.read_to_string(&mut buffer)?;
        buffer
    } else {
        "Welcome to MChat!".to_string()
    };

    // Display logo as a splash screen
    terminal.draw(|f| {
        let size = f.size();
        let paragraph = Paragraph::new(logo)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, size);
    })?;
    sleep(Duration::from_secs(2));
    //
    // Add logo to chat
    //app.add_message(logo);
    //terminal.backend_mut().write(logo);
    //sleep(Duration::from_secs(2));

    // Main loop
    let mut log: Vec<String> = vec![];
    loop {
        let ws: terminal::WindowSize = window_size().unwrap();
        app.render(&mut terminal)?;

        // Handle input
        let mut mes = [0; 64];
        match stream.read(&mut mes) {
            Ok(n) => {
                if n > 0 {
                    let mes = String::from_utf8(mes[0..n].to_vec()).unwrap();
                    log.push(mes.clone());
                    app.add_message(mes);
                    //stream.flush().unwrap();
                }
            }
            Err(e) => {
                //app.add_message("error".to_string());
                //if e != std::io::ErrorKind::WouldBlock {
                //    eprintln!("not would block");
                //}
            }
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(KeyEvent {
                code, modifiers, ..
            }) = read()?
            {
                match code {
                    KeyCode::Enter => {
                        let prompt = app.prompt.clone();
                        app.add_message(prompt.clone());
                        stream.write(prompt.as_bytes());
                        app.prompt.clear();
                    }
                    KeyCode::Char(c) => {
                        if modifiers.contains(KeyModifiers::CONTROL) {
                            // Exit on Ctrl+C
                            break;
                        } else {
                            app.prompt.push(c);
                        }
                    }
                    KeyCode::Backspace => {
                        app.prompt.pop();
                    }
                    _ => {}
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    //println!("log: {:#?}", log);
    let fin = String::new();
    fs::write(
        "mchat.log",
        app.chat.iter().fold(fin, |mut fin, str| {
            fin.push_str(&("user:".to_owned() + str + "\n"));
            fin
        }),
    );
    println!("chat:{:#?}", app.chat);
    Ok(())
}
