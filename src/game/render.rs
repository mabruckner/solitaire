use game::cards::Card;

pub struct CardData<CardId> {
    pub pos: [f64; 3],
    pub display: Option<Card>,
    pub drag_children: Option<Vec<CardId>>
}

pub trait Renderable {
    type CardId;
    fn get_cards(&self) -> Vec<Self::CardId>;
    fn get_data_for(&self, id: Self::CardId) -> Option<CardData<Self::CardId>>;
    //fn get_action_tap(&self, CardId) > (
}
