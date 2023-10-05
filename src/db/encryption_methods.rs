#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EncryptionMethods {
    AES256 = 1,
}

impl From<i16> for EncryptionMethods {
    fn from(value: i16) -> Self {
        match value {
            1 => EncryptionMethods::AES256,
            _ => panic!("Invalid encryption method"),
        }
    }
}
