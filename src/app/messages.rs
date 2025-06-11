use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};

use super::{Mode, Model};

pub enum MoveDirection {
    Up,
    Down,
    Left,
    Right,
}
pub enum Message {
    ClearCommand,
    AppendMovement(char),
    MoveDirection(MoveDirection),
    PropagateEvent(Event),
    NextField,
    InsertTag,
    ChangeMode(Mode),
    Exit,
}
pub fn try_message(model: &Model, ev: Event) -> Option<Message> {
    match ev {
        Event::Key(key_event) => match key_event.kind {
            KeyEventKind::Press => match model.current_mode {
                Mode::Normal => normal_key_press(key_event),
                Mode::CreateTag => {
                    let res = match key_event.code {
                        KeyCode::Esc => Message::ChangeMode(Mode::Normal),
                        KeyCode::Insert | KeyCode::Tab | KeyCode::Enter => Message::NextField,
                        _ => Message::PropagateEvent(ev),
                    };
                    Some(res)
                }
                Mode::ShowTags => show_tags_key_press(key_event),
                Mode::Insert => insert_key_press(key_event),
            },
            KeyEventKind::Repeat => None,
            KeyEventKind::Release => None,
        },
        Event::Mouse(_) => None,
        _ => None,
    }
}

#[inline]
fn normal_key_press(key: KeyEvent) -> Option<Message> {
    let res = match key.code {
        KeyCode::Esc => Message::ClearCommand,
        KeyCode::Char(c) if c.is_ascii_digit() => Message::AppendMovement(c),
        KeyCode::Char('q') => Message::Exit,
        KeyCode::Char('T') => Message::ChangeMode(Mode::CreateTag),
        KeyCode::Char('t') => Message::ChangeMode(Mode::ShowTags),
        KeyCode::Char('i') => Message::ChangeMode(Mode::Insert),
        KeyCode::Char('?') => todo!(),
        KeyCode::Char('k') | KeyCode::Up => Message::MoveDirection(MoveDirection::Up),
        KeyCode::Char('j') | KeyCode::Down => Message::MoveDirection(MoveDirection::Down),
        KeyCode::Char('h') | KeyCode::Left => Message::MoveDirection(MoveDirection::Left),
        KeyCode::Char('l') | KeyCode::Right => Message::MoveDirection(MoveDirection::Right),
        _ => return None,
    };
    Some(res)
}
#[inline]
fn show_tags_key_press(key: KeyEvent) -> Option<Message> {
    let res = match key.code {
        KeyCode::Esc => Message::ChangeMode(Mode::Normal),
        _ => return None,
    };
    Some(res)
}
#[inline]
fn insert_key_press(key: KeyEvent) -> Option<Message> {
    let res = match key.code {
        KeyCode::Esc | KeyCode::Char('q') => Message::ChangeMode(Mode::Normal),
        KeyCode::Enter => Message::InsertTag,
        KeyCode::Char(c) if c.is_ascii_digit() => Message::AppendMovement(c),
        KeyCode::Char('k') | KeyCode::Up => Message::MoveDirection(MoveDirection::Up),
        KeyCode::Char('j') | KeyCode::Down => Message::MoveDirection(MoveDirection::Down),
        KeyCode::Char('h') | KeyCode::Left => Message::MoveDirection(MoveDirection::Left),
        KeyCode::Char('l') | KeyCode::Right => Message::MoveDirection(MoveDirection::Right),
        _ => return None,
    };
    Some(res)
}
