pub trait Problem<A, P> {
    fn percept(&self) -> P;
    fn actions(&self) -> Vec<A>;
    fn result(&self, A) -> Self;
    fn is_goal(&self) -> bool;
}


