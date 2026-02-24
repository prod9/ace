/// Events emitted by actions to report progress.
pub enum Event {
    /// Long-running operation started — display a spinner.
    Progress(String),
    /// Operation completed successfully — display ✓.
    Done(String),
    /// Non-fatal warning — display ⚠.
    Warn(String),
}

/// Sink that receives action events for rendering.
pub trait EventSink {
    fn handle(&mut self, event: Event);
    fn finish(&mut self);
}

/// Wrapper that guarantees `finish()` is called on drop.
pub struct OwnedSink(Box<dyn EventSink>);

impl OwnedSink {
    pub fn new(sink: Box<dyn EventSink>) -> Self {
        Self(sink)
    }

    pub fn handle(&mut self, event: Event) {
        self.0.handle(event);
    }
}

impl Drop for OwnedSink {
    fn drop(&mut self) {
        self.0.finish();
    }
}

/// No-op sink for tests and non-interactive contexts.
pub struct NoopSink;

impl EventSink for NoopSink {
    fn handle(&mut self, _event: Event) {}
    fn finish(&mut self) {}
}
