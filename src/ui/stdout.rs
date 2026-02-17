use super::{UIFuture, UI};

pub struct StdoutUI;

impl UI for StdoutUI {
    fn message(&self, text: &str) -> UIFuture<'_, ()> {
        println!("{text}");
        Box::pin(std::future::ready(()))
    }

    fn confirm(&self, _prompt: &str) -> UIFuture<'_, bool> {
        Box::pin(std::future::ready(false))
    }

    fn ask(&self, _prompt: &str) -> UIFuture<'_, String> {
        Box::pin(std::future::ready(String::new()))
    }

    fn select(&self, _prompt: &str, _options: &[&str]) -> UIFuture<'_, usize> {
        Box::pin(std::future::ready(0))
    }
}
