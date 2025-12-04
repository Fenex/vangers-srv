#[derive(Debug, Clone, Copy)]
pub struct Bind(u8);

impl Bind {
    // TODO: returns `Result` instead of `Self`
    pub fn new(id: u8) -> Self {
        if !(1..=31).contains(&id) {
            panic!("`id` should be in range [1..32]");
        }
        Bind(id)
    }

    pub fn id(&self) -> u8 {
        self.0
    }

    #[inline(always)]
    pub fn mask(&self) -> i32 {
        1 << (self.0 - 1)
    }
}
