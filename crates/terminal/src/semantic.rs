//! Terminal meaning is separate from bounded syntax and output transport.
use crate::parser::{Event, Header};
use crate::{FeedOutcome, OutputEvent, Terminal, TitleTarget};

enum Action {
    Control(u8),
    Move(isize, isize, bool),
    Position(Option<usize>, Option<usize>),
    Margins(usize, usize),
    Modes(Header, bool),
    Save,
    Restore,
    Index(bool),
    ReverseIndex,
    Erase(usize),
    EraseLine,
    Query(u16),
    Unsupported,
}
fn csi(header: Header, final_byte: u8) -> Action {
    use Action::*;
    let params = header.parameters();
    if !header.intermediates().is_empty() || params.iter().any(|p| p.subparameter) {
        return Unsupported;
    }
    let value = |i: usize| params.get(i).and_then(|p| p.value).unwrap_or(0);
    if header.prefix() == Some(b'?') && matches!(final_byte, b'h' | b'l') {
        // Validate the entire list before changing any mode. Unsupported lists
        // remain observable and cannot leave a partially applied command.
        return if !params.is_empty() && params.iter().all(|p| matches!(p.value, Some(1 | 6 | 1049)))
        {
            Modes(header, final_byte == b'h')
        } else {
            Unsupported
        };
    }
    if header.prefix().is_some() {
        return Unsupported;
    }
    if params.len() <= 2 {
        match final_byte {
            b'H' | b'f' => {
                return Position(
                    Some(usize::from(value(0).max(1) - 1)),
                    Some(usize::from(value(1).max(1) - 1)),
                );
            }
            b'r' => return Margins(usize::from(value(0)), usize::from(value(1))),
            _ => {}
        }
    }
    if params.len() > 1 {
        return Unsupported;
    }
    let n = value(0);
    let count = n.max(1) as isize;
    match (n, final_byte) {
        (_, b'A') => Move(-count, 0, false),
        (_, b'B') => Move(count, 0, false),
        (_, b'C') => Move(0, count, false),
        (_, b'D') => Move(0, -count, false),
        (_, b'E') => Move(count, 0, true),
        (_, b'F') => Move(-count, 0, true),
        (_, b'G') => Position(None, Some(usize::from(n.max(1) - 1))),
        (_, b'd') => Position(Some(usize::from(n.max(1) - 1)), None),
        (_, b'X') => Erase(usize::from(n.max(1))),
        (0, b'K') => EraseLine,
        (n @ (5 | 6), b'n') => Query(n),
        _ => Unsupported,
    }
}
fn esc(header: Header, final_byte: u8) -> Action {
    if header.prefix().is_some()
        || !header.parameters().is_empty()
        || !header.intermediates().is_empty()
    {
        return Action::Unsupported;
    }
    match final_byte {
        b'7' => Action::Save,
        b'8' => Action::Restore,
        b'D' => Action::Index(false),
        b'E' => Action::Index(true),
        b'M' => Action::ReverseIndex,
        _ => Action::Unsupported,
    }
}
impl Terminal {
    pub(crate) fn dispatch(&mut self, event: Event, out: &mut FeedOutcome) {
        match event {
            Event::Print(byte) => {
                for c in self.decoder.push(byte).into_iter().flatten() {
                    if c.is_control() {
                        self.end_cluster();
                        out.unsupported = true;
                    } else {
                        self.print(c, out);
                    }
                }
            }
            Event::Execute(byte) => {
                self.flush_decoder(out);
                self.apply_action(Action::Control(byte), out);
            }
            Event::Boundary => {
                self.flush_decoder(out);
                self.end_cluster();
            }
            Event::Cancelled { string } => {
                self.flush_decoder(out);
                self.end_cluster();
                out.unsupported |= string;
            }
            Event::Csi { header, final_byte } => self.apply_action(csi(header, final_byte), out),
            Event::Esc { header, final_byte } => self.apply_action(esc(header, final_byte), out),
            Event::Osc(bytes) => self.osc(bytes, out),
            Event::Limit => out.parser_limit = true,
            Event::Invalid | Event::IgnoredString(_) => out.unsupported = true,
        }
    }
    fn osc(&mut self, bytes: Vec<u8>, out: &mut FeedOutcome) {
        let Some(split) = bytes.iter().position(|b| *b == b';') else {
            out.unsupported = true;
            return;
        };
        let target = match &bytes[..split] {
            b"0" => TitleTarget::IconAndWindow,
            b"1" => TitleTarget::Icon,
            b"2" => TitleTarget::Window,
            _ => {
                out.unsupported = true;
                return;
            }
        };
        let Ok(text) = std::str::from_utf8(&bytes[split + 1..]) else {
            out.unsupported = true;
            return;
        };
        if text.chars().any(char::is_control) {
            out.unsupported = true;
            return;
        }
        self.output.emit(OutputEvent::Title {
            target,
            text: text.to_owned(),
        });
    }
    fn apply_action(&mut self, action: Action, out: &mut FeedOutcome) {
        match action {
            Action::Modes(header, enabled) => {
                self.end_cluster();
                for parameter in header.parameters() {
                    match parameter.value.unwrap() {
                        1 => self.application_cursor = enabled,
                        6 => {
                            self.active.set_origin(enabled);
                            out.changed = true;
                        }
                        1049 => self.alternate(enabled, out),
                        _ => unreachable!("validated mode list"),
                    }
                }
            }
            Action::Position(row, column) => {
                self.end_cluster();
                let current_row = self.active.cursor.row
                    - if self.active.origin {
                        self.active.top
                    } else {
                        0
                    };
                self.active.position(
                    row.unwrap_or(current_row),
                    column.unwrap_or(self.active.cursor.column),
                );
                out.changed = true;
            }
            Action::Move(vertical, horizontal, carriage_return) => {
                self.end_cluster();
                self.active.relative(vertical, horizontal);
                if carriage_return {
                    self.active.cursor.column = 0;
                }
                out.changed = true;
            }
            Action::Margins(top, bottom) => {
                self.end_cluster();
                out.changed |= self.active.set_margins(top, bottom);
            }
            Action::Save => {
                self.end_cluster();
                self.active.save_cursor();
            }
            Action::Restore => {
                self.end_cluster();
                self.active.restore_cursor();
                out.changed = true;
            }
            Action::Index(carriage_return) => {
                self.end_cluster();
                self.active.rows[self.active.cursor.row].soft_wrapped = false;
                self.down(out);
                if carriage_return {
                    self.active.cursor.column = 0;
                }
            }
            Action::ReverseIndex => {
                self.end_cluster();
                self.active.reverse_index(out);
            }
            Action::Unsupported => out.unsupported = true,
            Action::Query(5) => self.output.emit(OutputEvent::Reply(b"\x1b[0n".to_vec())),
            Action::Query(6) => {
                let c = self.cursor();
                self.output.emit(OutputEvent::Reply(
                    format!(
                        "\x1b[{};{}R",
                        c.row + 1
                            - if self.active.origin {
                                self.active.top
                            } else {
                                0
                            },
                        c.column + 1
                    )
                    .into_bytes(),
                ));
            }
            Action::Query(_) => out.unsupported = true,
            Action::Control(b) => {
                self.end_cluster();
                match b {
                    7 => self.output.emit(OutputEvent::Bell),
                    b'\r' => {
                        self.active.cursor.column = 0;
                        self.active.cursor.wrap_pending = false;
                        out.changed = true;
                    }
                    b'\n' => {
                        self.active.rows[self.active.cursor.row].soft_wrapped = false;
                        self.down(out);
                        self.active.cursor.wrap_pending = false;
                    }
                    8 => {
                        self.active.cursor.column = self.active.cursor.column.saturating_sub(1);
                        self.active.cursor.wrap_pending = false;
                        out.changed = true;
                    }
                    0 => {}
                    _ => out.unsupported = true,
                }
            }
            Action::Erase(n) => {
                self.end_cluster();
                self.erase(
                    self.active.cursor.row,
                    self.active.cursor.column,
                    (self.active.cursor.column + n).min(self.size.columns),
                );
                out.changed = true;
            }
            Action::EraseLine => {
                self.end_cluster();
                self.erase(
                    self.active.cursor.row,
                    self.active.cursor.column,
                    self.size.columns,
                );
                self.active.rows[self.active.cursor.row].soft_wrapped = false;
                out.changed = true;
            }
        }
    }
}
