use std::io::{self, Write};

use anyhow::{Context, Result, bail};
use crossterm::cursor;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::style::{Attribute, Print, SetAttribute};
use crossterm::terminal;
use crossterm::terminal::{Clear, ClearType};

pub fn status(message: impl AsRef<str>) {
    println!("{}", message.as_ref());
}

pub fn warn(message: impl AsRef<str>) {
    println!("Warning: {}", message.as_ref());
}

pub fn error(message: impl AsRef<str>) {
    eprintln!("\nError: {}", message.as_ref());
}

pub fn blank_line() {
    println!();
}

pub fn print_qr(qr: &str) {
    println!("{qr}");
}

pub fn menu(options: &[&str]) -> Result<usize> {
    if options.is_empty() {
        bail!("menu cannot be shown without options");
    }

    blank_line();

    if options.len() <= 9 {
        match interactive_menu(options) {
            Ok(value) => return Ok(value),
            Err(error) if is_raw_mode_error(&error) => {
                warn("interactive input is unavailable; press a number and Enter.");
            }
            Err(error) => return Err(error),
        }
    }

    line_menu(options)
}

fn interactive_menu(options: &[&str]) -> Result<usize> {
    terminal::enable_raw_mode().context("failed to enable raw terminal input")?;
    let raw_mode = RawModeGuard;
    let mut stdout = io::stdout();
    let mut selected = 0;

    render_interactive_menu(&mut stdout, options, selected)?;

    loop {
        match event::read().context("failed to read keypress")? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    drop(raw_mode);
                    println!();
                    bail!("interrupted");
                }
                KeyCode::Esc => {
                    drop(raw_mode);
                    println!();
                    bail!("interrupted");
                }
                KeyCode::Up => {
                    selected = previous_selection(selected, options.len());
                    rerender_interactive_menu(&mut stdout, options, selected)?;
                }
                KeyCode::Down => {
                    selected = next_selection(selected, options.len());
                    rerender_interactive_menu(&mut stdout, options, selected)?;
                }
                KeyCode::Home => {
                    selected = 0;
                    rerender_interactive_menu(&mut stdout, options, selected)?;
                }
                KeyCode::End => {
                    selected = options.len() - 1;
                    rerender_interactive_menu(&mut stdout, options, selected)?;
                }
                KeyCode::Enter => {
                    let value = selected + 1;
                    drop(raw_mode);
                    println!("{value}");
                    return Ok(value);
                }
                KeyCode::Char(character) => {
                    if let Some(value) = selection_from_char(character, options.len()) {
                        drop(raw_mode);
                        println!("{value}");
                        return Ok(value);
                    }

                    print!("\x07");
                    io::stdout().flush().context("failed to flush stdout")?;
                }
                _ => {
                    print!("\x07");
                    io::stdout().flush().context("failed to flush stdout")?;
                }
            },
            _ => {}
        }
    }
}

fn render_interactive_menu<W: Write>(
    writer: &mut W,
    options: &[&str],
    selected: usize,
) -> Result<()> {
    for (index, option) in options.iter().enumerate() {
        execute!(writer, Clear(ClearType::CurrentLine))?;

        if index == selected {
            execute!(
                writer,
                SetAttribute(Attribute::Reverse),
                Print(format!("{}. {option}", index + 1)),
                SetAttribute(Attribute::Reset),
                Print("\r\n")
            )?;
        } else {
            execute!(writer, Print(format!("{}. {option}\r\n", index + 1)))?;
        }
    }

    execute!(
        writer,
        Clear(ClearType::CurrentLine),
        Print(format!(
            "Choose 1-{} (number key, or ↑↓ + Enter): ",
            options.len()
        ))
    )?;
    writer.flush().context("failed to flush stdout")?;

    Ok(())
}

fn rerender_interactive_menu<W: Write>(
    writer: &mut W,
    options: &[&str],
    selected: usize,
) -> Result<()> {
    execute!(
        writer,
        cursor::MoveUp(options.len() as u16),
        cursor::MoveToColumn(0)
    )?;
    render_interactive_menu(writer, options, selected)
}

fn line_menu(options: &[&str]) -> Result<usize> {
    loop {
        print_options(options);
        let label = format!("Choose 1-{}", options.len());
        let input = prompt(&label)?;

        match input.trim().parse::<usize>() {
            Ok(value) if (1..=options.len()).contains(&value) => return Ok(value),
            _ => status("Please enter one of the numbered options."),
        }
    }
}

fn print_options(options: &[&str]) {
    for (index, option) in options.iter().enumerate() {
        println!("{}. {option}", index + 1);
    }
}

fn selection_from_char(character: char, option_count: usize) -> Option<usize> {
    let value = character.to_digit(10)? as usize;

    if (1..=option_count).contains(&value) {
        Some(value)
    } else {
        None
    }
}

fn previous_selection(selected: usize, option_count: usize) -> usize {
    if selected == 0 {
        option_count - 1
    } else {
        selected - 1
    }
}

fn next_selection(selected: usize, option_count: usize) -> usize {
    (selected + 1) % option_count
}

fn is_raw_mode_error(error: &anyhow::Error) -> bool {
    error
        .chain()
        .any(|cause| cause.to_string().contains("raw terminal input"))
}

pub fn prompt(label: &str) -> Result<String> {
    print!("{label}: ");
    io::stdout().flush().context("failed to flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("failed to read input")?;

    Ok(input)
}

pub fn prompt_required(label: &str) -> Result<String> {
    loop {
        let input = prompt(label)?;
        let input = input.trim();

        if !input.is_empty() {
            return Ok(input.to_string());
        }

        status("Please enter a value.");
    }
}

struct RawModeGuard;

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_digit_to_selection() {
        assert_eq!(selection_from_char('1', 2), Some(1));
        assert_eq!(selection_from_char('2', 2), Some(2));
        assert_eq!(selection_from_char('3', 2), None);
        assert_eq!(selection_from_char('x', 2), None);
    }

    #[test]
    fn wraps_arrow_selection() {
        assert_eq!(previous_selection(0, 3), 2);
        assert_eq!(previous_selection(2, 3), 1);
        assert_eq!(next_selection(2, 3), 0);
        assert_eq!(next_selection(0, 3), 1);
    }
}
