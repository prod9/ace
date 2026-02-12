use crate::ace::Ace;

pub async fn run(ace: &Ace) {
    ace.ui().message("config: effective config").await;
}
