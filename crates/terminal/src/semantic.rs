//! Terminal meaning is separate from bounded syntax and output transport.
use crate::parser::{Event, Header};
use crate::query::Query;
use crate::{FeedOutcome, OutputEvent, Terminal, TitleTarget};

enum Action {
    Control(u8),
    Move(isize, isize, bool),
    Position(Option<usize>, Option<usize>),
    Margins(usize, usize),
    Modes(Header, bool),
    InsertMode(bool),
    Save,
    Restore,
    Reset(bool),
    Index(bool),
    ReverseIndex,
    Erase(usize),
    EditCharacters(usize, bool),
    EditLines(usize, bool),
    EraseLine(u16),
    EraseDisplay(u16),
    Tab(usize, bool),
    SetTab,
    ClearTabs(bool),
    Report(Query),
    Rendition(Header),
    Unsupported,
}
fn csi(header: Header, final_byte: u8) -> Action {
    use Action::*;
    if header.prefix().is_none()
        && header.parameters().is_empty()
        && header.intermediates() == b"!"
        && final_byte == b'p'
    {
        return Reset(false);
    }
    if let Some(query) = Query::parse(header, final_byte) {
        return Report(query);
    }
    let params = header.parameters();
    if header.prefix().is_none() && header.intermediates().is_empty() && final_byte == b'm' {
        return Rendition(header);
    }
    if !header.intermediates().is_empty() || params.iter().any(|p| p.subparameter) {
        return Unsupported;
    }
    let value = |i: usize| params.get(i).and_then(|p| p.value).unwrap_or(0);
    if header.prefix() == Some(b'?') && matches!(final_byte, b'h' | b'l') {
        // Validate the entire list before changing any mode. Unsupported lists
        // remain observable and cannot leave a partially applied command.
        return if !params.is_empty()
            && params
                .iter()
                .all(|p| matches!(p.value, Some(1 | 6 | 7 | 25 | 1049 | 2004)))
        {
            Modes(header, final_byte == b'h')
        } else {
            Unsupported
        };
    }
    if header.prefix().is_some() {
        return Unsupported;
    }
    if matches!(final_byte, b'h' | b'l') {
        return if !params.is_empty() && params.iter().all(|p| p.value == Some(4)) {
            InsertMode(final_byte == b'h')
        } else {
            Unsupported
        };
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
        (_, b'@') => EditCharacters(usize::from(n.max(1)), true),
        (_, b'P') => EditCharacters(usize::from(n.max(1)), false),
        (_, b'L') => EditLines(usize::from(n.max(1)), true),
        (_, b'M') => EditLines(usize::from(n.max(1)), false),
        (mode @ 0..=2, b'K') => EraseLine(mode),
        (mode @ 0..=3, b'J') => EraseDisplay(mode),
        (_, b'I') => Tab(usize::from(n.max(1)), true),
        (_, b'Z') => Tab(usize::from(n.max(1)), false),
        (0, b'g') => ClearTabs(false),
        (3, b'g') => ClearTabs(true),
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
        b'H' => Action::SetTab,
        b'Z' => Action::Report(Query::PrimaryAttributes),
        b'c' => Action::Reset(true),
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
            Event::EscapeBoundary => self.flush_decoder(out),
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
            Event::Osc(bytes) => {
                self.end_cluster();
                self.osc(bytes, out);
            }
            Event::Limit => {
                self.end_cluster();
                out.parser_limit = true;
            }
            Event::Invalid | Event::IgnoredString(_) => {
                self.end_cluster();
                out.unsupported = true;
            }
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
        if !matches!(action, Action::Rendition(_)) {
            self.end_cluster();
        }
        match action {
            Action::Reset(hard) => self.reset(hard, out),
            Action::InsertMode(enabled) => self.insert_mode = enabled,
            Action::Modes(header, enabled) => {
                self.end_cluster();
                for parameter in header.parameters() {
                    match parameter.value.unwrap() {
                        1 => self.application_cursor = enabled,
                        2004 => self.bracketed_paste = enabled,
                        6 => {
                            self.active.set_origin(enabled);
                            out.changed = true;
                        }
                        7 => {
                            self.active.autowrap = enabled;
                            self.active.cursor.wrap_pending = false;
                            out.changed = true;
                        }
                        25 => {
                            out.changed |= self.cursor_visible != enabled;
                            self.cursor_visible = enabled;
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
            Action::Tab(count, forward) => {
                self.active.cursor.column = self.tabs.destination(
                    self.active.cursor.column,
                    self.size.columns,
                    count,
                    forward,
                );
                self.active.cursor.wrap_pending = false;
                out.changed = true;
            }
            Action::SetTab => self.tabs.set(self.active.cursor.column, true),
            Action::ClearTabs(all) => {
                if all {
                    self.tabs.clear();
                } else {
                    self.tabs.set(self.active.cursor.column, false);
                }
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
            Action::Rendition(header) => {
                if let Some(style) = self.current_style().sgr(header) {
                    out.style_limit |= !self.set_rendition(style);
                } else {
                    self.end_cluster();
                    out.unsupported = true;
                }
            }
            Action::Unsupported => {
                self.end_cluster();
                out.unsupported = true;
            }
            Action::Report(query) => self.output.emit(OutputEvent::Reply(query.reply(self))),
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
                    b'\t' => self.apply_action(Action::Tab(1, true), out),
                    0 => {}
                    _ => out.unsupported = true,
                }
            }
            Action::EditCharacters(count, insert) => {
                self.active.edit_characters(count, insert);
                out.changed = true;
            }
            Action::EditLines(count, insert) => {
                if self.active.edit_lines(count, insert) {
                    out.changed = true;
                    out.scrolled_without_history = true;
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
            Action::EraseLine(mode) => {
                self.active.erase_line(self.active.cursor.row, mode);
                out.changed = true;
            }
            Action::EraseDisplay(mode) => {
                self.active.erase_display(mode);
                out.changed = true;
            }
        }
    }
}
