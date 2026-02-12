use crate::ace::Ace;

pub async fn run(ace: &Ace, school_name: &str, source: &str) {
    ace.ui().message(&format!("setup: school={school_name} source={source}")).await;
}
