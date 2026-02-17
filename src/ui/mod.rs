mod stdout;

pub use stdout::StdoutUI;

use std::future::Future;
use std::pin::Pin;

pub type UIFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

pub trait UI {
    fn message(&self, text: &str) -> UIFuture<'_, ()>;
    fn confirm(&self, prompt: &str) -> UIFuture<'_, bool>;
    fn ask(&self, prompt: &str) -> UIFuture<'_, String>;
    fn select(&self, prompt: &str, options: &[&str]) -> UIFuture<'_, usize>;
}
