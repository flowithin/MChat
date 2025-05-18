use std::{
    io::{self, stdout, Error, Write},
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

fn main() -> io::Result<()> {
    // with function
    io::stdout().execute(Clear(crossterm::terminal::ClearType::All));
    io::stdout().execute(MoveTo(11, 11))?;
    //io::stdout().execute(RestorePosition)?;
    enable_raw_mode();
    print_events();
    io::stdout().execute(Clear(crossterm::terminal::ClearType::All));
    io::stdout().execute(MoveTo(0, 0))?;
    disable_raw_mode();
    Ok(())
}

fn print_events() -> std::io::Result<()> {
    let mut prompt = String::new();
    let mut chat = String::new();
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
                        chat.push_str(&(prompt.clone() + &"\n\r".repeat(2)));
                        prompt.clear();
                    }
                    KeyCode::Char(ch) => {
                        if modifiers == KeyModifiers::CONTROL {
                            return Ok(());
                        } else {
                            prompt.push(ch);
                        }
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
        stdout().queue(Print(chat.clone()));

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
