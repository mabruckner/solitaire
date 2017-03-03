use game::cards::Card;
use game::grid::GridLocation;

#[derive(Eq, PartialEq)]
pub enum CardDisplay {
    Front(Card),
    Back,
    Empty
}

pub struct CardData<CardId> {
    pub pos: GridLocation,
    pub display: CardDisplay,
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
    fn get_grid_extents() -> (GridLocation, GridLocation);
}
