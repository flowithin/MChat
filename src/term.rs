use std::{
    fs::File,
    io::{self, stdout, Error, Read, Write},
    string,
    thread::sleep,
    time::Duration,
};

//#[cfg(feature = "bracketed-paste")]
use crossterm::{
    cursor::{DisableBlinking, EnableBlinking, MoveTo, RestorePosition, SavePosition},
    event::{
        poll, read, DisableBracketedPaste, DisableFocusChange, DisableMouseCapture,
        EnableBracketedPaste, EnableFocusChange, EnableMouseCapture, Event, KeyCode, KeyEvent,
        KeyModifiers,
    },
    execute,
    style::Print,
    terminal::{self, disable_raw_mode, enable_raw_mode, window_size, Clear},
    ExecutableCommand, QueueableCommand,
};
struct Rect {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
}

fn chat_window(stdout: &mut impl Write, chat: &[String], boundary: &Rect) {
    //the linenum
    let n = chat.len();
    let m = n.checked_sub(boundary.h as usize).unwrap_or(0);
    for (dy, line) in chat.iter().skip(m).enumerate() {
        stdout
            .queue(MoveTo(boundary.x, boundary.y + dy as u16))
            .unwrap();
        stdout.write(line.as_bytes()).unwrap();
    }
}
fn main() -> io::Result<()> {
    // with function
    io::stdout().execute(Clear(crossterm::terminal::ClearType::All));
    io::stdout().execute(MoveTo(21, 11))?;
    let mut buf: [u8; 1024] = [0; 1024];

    io::stdout().write({
        File::open("mchat.logo")?.read(&mut buf)?;
        &buf
    });
    //io::stdout().execute(RestorePosition)?;

    // Wait for Enter key
    enable_raw_mode();
    loop {
        if poll(Duration::from_millis(50))? {
            if let Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) = read()?
            {
                break;
            }
        }
        // Optionally update screen or animate loading dots here
    }
    print_events();
    io::stdout().execute(Clear(crossterm::terminal::ClearType::All));
    io::stdout().execute(MoveTo(0, 0))?;
    disable_raw_mode();
    Ok(())
}

pub fn print_events() -> std::io::Result<()> {
    let mut prompt = String::new();
    let mut chat: Vec<String> = vec![];
    loop {
        let ws: terminal::WindowSize = window_size().unwrap();
        // `read()` blocks until an `Event` is available
        while poll(Duration::from_millis(0))? {
            match read()? {
                Event::FocusGained => println!("FocusGained"),
                Event::FocusLost => println!("FocusLost"),
                Event::Key(KeyEvent {
                    code,
                    modifiers,
                    kind,
                    state,
                }) => match code {
                    KeyCode::Enter => {
                        let prompt_ = prompt.clone();
                        chat.push(prompt_);
                        prompt.clear();
                    }
                    KeyCode::Char(ch) => {
                        if modifiers == KeyModifiers::CONTROL {
                            return Ok(());
                        } else {
                            prompt.push(ch);
                        }
                    }
                    KeyCode::Backspace => {
                        prompt.pop();
                    }
                    _ => {}
                },
                Event::Mouse(event) => println!("{:?}", event),
                //#[cfg(feature = "bracketed-paste")]
                Event::Paste(data) => println!("{:?}", data),
                Event::Resize(width, height) => println!("New size {}x{}", width, height),
            }
        }
        io::stdout().queue(Clear(crossterm::terminal::ClearType::All));

        stdout().queue(MoveTo(0, ws.rows - 2));
        stdout().queue(Print("═".repeat((ws.columns) as usize)));

        stdout().queue(MoveTo(0, 0));
        let boundary = Rect {
            x: 0,
            y: 0,
            w: ws.columns,
            h: ws.rows - 2,
        };
        chat_window(&mut stdout(), &chat, &boundary);

        stdout().queue(MoveTo(0, ws.rows - 1));
        stdout().queue(Print(prompt.clone()));

        stdout().flush();
        sleep(Duration::from_millis(33));
    }
    execute!(
        std::io::stdout(),
        DisableBracketedPaste,
        DisableFocusChange,
        DisableMouseCapture
    )?;
    Ok(())
}
