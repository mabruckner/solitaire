#[derive(Hash,PartialEq, Eq, Clone, Debug)]
pub struct Card {
    pub suit: usize,
    pub color: usize,
    pub rank: usize
}
