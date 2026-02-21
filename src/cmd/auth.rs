use crate::ace::Ace;

pub async fn run(ace: &mut Ace, name: &str) {
    ace.session().ui.message(&format!("auth: service={name}")).await;
}
