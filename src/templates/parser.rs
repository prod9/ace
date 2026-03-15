/// Candidate match: (open_pos, name_start, name_end). Caller validates the name.
pub type Match = (usize, usize, usize);

#[derive(Clone, Copy)]
enum Phase {
    Text,
    MaybeOpen,
    Name,
    MaybeClose,
}

pub struct Parser {
    phase: Phase,
    open_pos: usize,
    name_start: usize,
}

impl Parser {
    pub fn new() -> Self {
        Self { phase: Phase::Text, open_pos: 0, name_start: 0 }
    }

    pub fn feed(&mut self, i: usize, byte: u8) -> Option<Match> {
        match self.phase {
            Phase::Text => self.on_text(i, byte),
            Phase::MaybeOpen => self.on_maybe_open(i, byte),
            Phase::Name => self.on_name(byte),
            Phase::MaybeClose => self.on_maybe_close(i, byte),
        }
    }

    fn on_text(&mut self, i: usize, byte: u8) -> Option<Match> {
        if byte == b'{' {
            self.phase = Phase::MaybeOpen;
            self.open_pos = i;
        }
        None
    }

    fn on_maybe_open(&mut self, i: usize, byte: u8) -> Option<Match> {
        self.phase = if byte == b'{' {
            self.name_start = i + 1;
            Phase::Name
        } else {
            Phase::Text
        };
        None
    }

    fn on_name(&mut self, byte: u8) -> Option<Match> {
        if byte == b'\n' {
            self.phase = Phase::Text;
        } else if byte == b'}' {
            self.phase = Phase::MaybeClose;
        }
        None
    }

    fn on_maybe_close(&mut self, i: usize, byte: u8) -> Option<Match> {
        self.phase = Phase::Text;
        (byte == b'}').then(|| (self.open_pos, self.name_start, i - 1))
    }
}
