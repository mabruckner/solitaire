#[derive(Hash,PartialEq, Eq, Clone, Debug)]
pub struct Card {
    pub suit: usize,
    pub color: usize,
    pub rank: usize
}


#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct Ident(usize);

impl Ident {
    pub fn new(x: usize) -> Ident {
        Ident(x)
    }
    pub fn interleave(&self) -> Ident {
        Ident(self.0*2+1)
    }
    pub fn expand(&self) -> Ident {
        Ident(self.0*2)
    }
}
