use crate::ace::Ace;

pub async fn run(ace: &Ace) {
    ace.ui().message("learn").await;
}
