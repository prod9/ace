use crate::ace::Ace;

pub async fn run(ace: &mut Ace) {
    ace.session().ui.message("ace").await;
}
