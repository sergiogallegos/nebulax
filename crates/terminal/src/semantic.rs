//! Terminal meaning is separate from bounded syntax and output transport.
use crate::parser::{Event, Header};
use crate::{FeedOutcome, OutputEvent, Terminal, TitleTarget};

enum Action {
    Control(u8),
    Left(usize),
    Erase(usize),
    EraseLine,
    Alternate(bool),
    Query(u16),
    Unsupported,
}
fn csi(header: Header, final_byte: u8) -> Action {
    use Action::*;
    let params = header.parameters();
    if !header.intermediates().is_empty()
        || params.len() > 1
        || params.iter().any(|p| p.subparameter)
    {
        return Unsupported;
    }
    let value = params.first().and_then(|p| p.value).unwrap_or(0);
    match (header.prefix(), value, final_byte) {
        (None, n, b'D') => Left(usize::from(n.max(1))),
        (None, n, b'X') => Erase(usize::from(n.max(1))),
        (None, 0, b'K') => EraseLine,
        (Some(b'?'), 1049, b'h') => Alternate(true),
        (Some(b'?'), 1049, b'l') => Alternate(false),
        (None, n @ (5 | 6), b'n') => Query(n),
        _ => Unsupported,
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
            Event::Osc(bytes) => self.osc(bytes, out),
            Event::Limit => out.parser_limit = true,
            Event::Esc { .. } | Event::Invalid | Event::IgnoredString(_) => out.unsupported = true,
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
            Action::Alternate(enable) => self.alternate(enable, out),
            Action::Unsupported => out.unsupported = true,
            Action::Query(5) => self.output.emit(OutputEvent::Reply(b"\x1b[0n".to_vec())),
            Action::Query(6) => {
                let c = self.cursor();
                self.output.emit(OutputEvent::Reply(
                    format!("\x1b[{};{}R", c.row + 1, c.column + 1).into_bytes(),
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
            Action::Left(n) => {
                self.end_cluster();
                self.active.cursor.column = self.active.cursor.column.saturating_sub(n);
                self.active.cursor.wrap_pending = false;
                out.changed = true;
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
