use iroh::SecretKey;
fn main() {
    let sk = SecretKey::generate(rand::rngs::OsRng);
    println!("ID: {}", sk.public());
}
