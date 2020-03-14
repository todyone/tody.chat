use protocol::Key;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

pub fn alphanumeric(len: usize) -> String {
    thread_rng().sample_iter(&Alphanumeric).take(len).collect()
}

pub fn generate_key() -> Key {
    alphanumeric(30)
}
