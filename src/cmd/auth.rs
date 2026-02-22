use crate::ace::Ace;

pub async fn run(_ace: &mut Ace, name: &str) {
    println!("auth: service={name}");
}
