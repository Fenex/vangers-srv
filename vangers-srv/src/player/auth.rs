use std::collections::hash_map::DefaultHasher;
use std::ffi::CString;
use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
};

use crate::utils::convert_cp866_to_utf8;

fn get_hash_pwd(pwd: &[u8]) -> u64 {
    // TODO: use more secure algorithm to hashing passwords
    let mut hasher = DefaultHasher::new();
    pwd.hash(&mut hasher);
    hasher.finish()
}

pub struct Auth {
    name: Vec<u8>,
    #[allow(dead_code)]
    pwd: Option<u64>,
}

impl Auth {
    pub fn new(name: &[u8], pwd: &[u8]) -> Self {
        let name = match name.last() {
            Some(0) => name.to_owned(),
            Some(_) => {
                let mut name = name.to_owned();
                name.push(0);
                name
            }
            None => CString::new(format!("Player-{:05}", rand::random::<u16>()))
                .unwrap()
                .to_bytes_with_nul()
                .to_owned(),
        };

        let pwd = if pwd.len() > 0 {
            Some(get_hash_pwd(pwd))
        } else {
            None
        };

        Self { name, pwd }
    }

    pub fn name(&self) -> &[u8] {
        return &self.name;
    }

    pub fn name_utf8(&self) -> String {
        convert_cp866_to_utf8(&self.name[..self.name.len() - 1]).unwrap_or("unknown".to_string())
    }
}

impl Debug for Auth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Auth: {}", self.name_utf8())
    }
}

#[cfg(test)]
mod test {
    use super::Auth;

    #[test]
    fn test_new_with_empty_name() {
        let auth = Auth::new(&[], &[]);
        assert_eq!(b"Player-", &auth.name()[..7]);
        assert_eq!(Some(&0), auth.name().last());
    }

    #[test]
    fn test_new_without_nullterminated_name() {
        let auth = Auth::new(b"test-auth", &[]);
        assert_eq!(b"test-auth\0", &auth.name()[..]);
    }

    #[test]
    fn test_new_with_nullterminated_name() {
        let auth = Auth::new(b"test-auth\0", &[]);
        assert_eq!(b"test-auth\0", &auth.name()[..],);
    }

    #[test]
    fn test_name_utf8() {
        let auth = Auth::new(b"login\0", b"pwd\0");
        assert_eq!("login", auth.name_utf8());
    }
}
