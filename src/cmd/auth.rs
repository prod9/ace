use crate::ace::Ace;

pub async fn run(ace: &Ace, name: &str) {
    ace.ui().message(&format!("auth: service={name}")).await;
}
