use game::cards::Card;

pub struct CardData<CardId> {
    pub pos: [f64; 3],
    pub display: Option<Card>,
    pub drag_children: Option<Vec<CardId>>
}

pub enum MouseAction<CardId> {
    Drop(CardId, CardId),
    Tap(CardId)
}

pub trait Renderable {
    type CardId;
    type Action;
    fn get_cards(&self) -> Vec<Self::CardId>;
    fn get_data_for(&self, id: Self::CardId) -> Option<CardData<Self::CardId>>;
    fn get_action_for(&self, act: MouseAction<Self::CardId>) -> Option<Self::Action>;
}
