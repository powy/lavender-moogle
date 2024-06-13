// modifier.rs
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum Modifier {
    Other = 0,
    None = 1,
    Alt = 2,
    Shift = 3,
    Ctrl = 4,
}

impl Modifier {
    pub fn to_char(self) -> char {
        match self {
            Modifier::Other => '0',
            Modifier::None => '1',
            Modifier::Alt => '2',
            Modifier::Shift => '3',
            Modifier::Ctrl => '4',
        }
    }
}
