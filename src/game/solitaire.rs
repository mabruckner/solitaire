use std::collections::HashMap;
use std::mem;
pub use game::cards::Card;
use game::problem::Problem;
use game::render::*;
use game::grid::*;

impl Card {
    pub fn new(suit: usize, color:usize, rank: usize) -> Card {
        Card {
            suit: suit,
            color: color,
            rank: rank
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Solitaire {
    deck: Vec<Card>,
    runoff: Vec<Card>,
    row: Vec<Vec<Card>>,
    visibility: Vec<usize>,
    goal: Vec<Vec<Card>>
}

impl Clone for Solitaire {
    fn clone(&self) -> Self {
        Solitaire {
            deck: self.deck.clone(),
            runoff: self.runoff.clone(),
            row: self.row.clone(),
            visibility: self.visibility.clone(),
            goal: self.goal.clone() 
        }
    }
}

impl Solitaire {
    pub fn deal(mut cards: Vec<Card>, row_count: usize, suits: usize) -> Self {
        let mut row = Vec::new();
        let mut visibility = Vec::new();
        for i in 0..row_count {
            let mut stack = Vec::new();
            for j in 0..i+1 {
                stack.push(cards.pop().unwrap());
            }
            row.push(stack);
            visibility.push(i);
        }
        let mut goal = Vec::new();
        for i in 0..suits {
            goal.push(Vec::new());
        }
        let mut deck = Vec::new();
        while let Some(card) = cards.pop() {
            deck.push(card);
        }
        Solitaire{
            deck: deck,
            runoff: Vec::new(),
            row: row,
            visibility: visibility,
            goal: goal
        }
    }
    fn get_mut_stack_for(&mut self, id: StackId) -> Option<&mut Vec<Card>> {
        match id {
            StackId(0,0) => Some(&mut self.deck),
            StackId(0,1) => Some(&mut self.runoff),
            StackId(1,x) => self.row.get_mut(x),
            StackId(2,x) => self.goal.get_mut(x),
            _ => None
        }
    }
}

impl Problem<CardGameAction, CardGamePercept> for Solitaire {
    fn percept(&self) -> CardGamePercept {
        let mut map = HashMap::new();
        let mut deck = Vec::new();
        let mut runoff = Vec::new();
        for _ in &self.deck {
            deck.push(None);
        }
        map.insert(StackId(0,0),deck);
        for x in &self.runoff {
            runoff.push(Some(x.clone()));
        }
        map.insert(StackId(0,1),runoff);
        for (i, stack) in self.row.iter().enumerate() {
            let mut thing = Vec::new();
            for (j, crd) in stack.iter().enumerate() {
                thing.push(if j < self.visibility[i] {
                    None
                } else {
                    Some(crd.clone())
                });
            }
            map.insert(StackId(1,i),thing);
        }
        for (i, stack) in self.goal.iter().enumerate() {
            let mut thing = Vec::new();
            for crd in stack {
                thing.push(Some(crd.clone()));
            }
            map.insert(StackId(2,i),thing);
        }
        CardGamePercept {
            stacks: map
        }
    }
    fn actions(&self) -> Vec<CardGameAction> {
        let mut out = Vec::new();
        let mut considerations = Vec::new();
        if !self.deck.is_empty() || !self.runoff.is_empty() {
            out.push(CardGameAction::Tap(StackId(0,0)));
        }
        if !self.runoff.is_empty() {
            considerations.push(self.runoff.last().unwrap());
        }
        for i in 0..self.row.len() {
            for j in self.visibility[i]..self.row[i].len() {
                considerations.push(&self.row[i][j]);
            }
        }
        for current in considerations.drain(..) {
            for k in 0..self.row.len() {
                if let Some(test) = self.row[k].last() {
                    if test.color != current.color && test.rank == current.rank + 1 {
                        out.push(CardGameAction::Move(current.clone(), StackId(1,k)));
                    }
                } else {
                    out.push(CardGameAction::Move(current.clone(), StackId(1,k)));
                }
            }
        }
        if !self.runoff.is_empty() {
            considerations.push(self.runoff.last().unwrap());
        }
        for i in 0..self.row.len() {
            if let Some(thing) = self.row[i].last() {
                considerations.push(thing);
            }
        }
        for current in considerations.drain(..) {
            for k in 0..self.goal.len() {
                if current.rank == self.goal[k].len() && current.suit == k {
                    out.push(CardGameAction::Move(current.clone(), StackId(2,k)));
                }
            }
        }
        out
    }
    fn result(&self, act: CardGameAction) -> Self {
        let mut out = self.clone();
        match act {
            CardGameAction::Tap(StackId(0,0)) => {
                if out.deck.is_empty() {
                    mem::swap(&mut out.deck, &mut out.runoff);
                    out.deck.reverse();
                } else {
                    for _ in 0..3 {
                        if let Some(crd) = out.deck.pop() {
                            out.runoff.push(crd);
                        }
                    }
                }
            },
            CardGameAction::Move(card, stack) => {
                let mut tmp = Vec::new();
                for i in 0..3 {
                    let mut halt = false;
                    let mut j = 0;
                    while let Some(mut test) = out.get_mut_stack_for(StackId(i,j)) {
                        for k in 0..test.len() {
                            if test[k] == card {
                                while test.len() > k {
                                    tmp.push(test.pop().unwrap());
                                }
                                halt = true;
                                break;
                            }
                        }
                        if halt {
                            break;
                        }
                        j=j+1;
                    }
                    if halt {
                        break;
                    }
                }
                if let Some(mut stack) = out.get_mut_stack_for(stack.clone()) {
                    while let Some(val) = tmp.pop() {
                        stack.push(val);
                    }
                }
                for i in 0..out.row.len() {
                    if out.row[i].len() <= out.visibility[i] && out.row[i].len() > 0 {
                        out.visibility[i] = out.visibility[i] - 1;
                    }
                }
            },
            _ => ()
        }
        out
    }
    fn is_goal(&self) -> bool {
        for v in &self.row {
            if !v.is_empty() {
                return false;
            }
        }
        self.deck.is_empty() && self.runoff.is_empty()
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct StackId(pub usize, pub usize);

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum CardGameAction {
    Move(Card, StackId),
    Tap(StackId)
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct CardGamePercept {
    pub stacks: HashMap<StackId,Vec<Option<Card>>>
}

fn get_stack_location(&StackId(i, j): &StackId) -> GridLocation {
    match i {
        0 => GridLocation::new(GridValue(j as i32*2, 0), GridValue(0, 0), 0),
        1 => GridLocation::new(GridValue(j as i32*2, 0), GridValue(2, 0), 0),
        2 | _ => GridLocation::new(GridValue(j as i32*2+5, 0), GridValue(0, 0), 0),
    }
}

impl Renderable for CardGamePercept {
    type CardId = (StackId, Option<usize>);
    type Action = CardGameAction;
    fn get_cards(&self) -> Vec<Self::CardId> {
        let mut out = Vec::new();
        for (k, val) in self.stacks.iter() {
            out.push((k.clone(), None));
            for i in 0..val.len() {
                out.push((k.clone(), Some(i)));
            }
        }
        out
    }
    fn get_data_for(&self, id: Self::CardId) -> Option<CardData<Self::CardId>> {
        if let Some(stack) = self.stacks.get(&id.0) {
            if let Some(idx) = id.1 {
                if let Some(val) = stack.get(idx) {
                    let mut children = Vec::new();
                    let mut offset = GridLocation::new(GridValue(0,0), GridValue(0,0), idx as i32);
                    if (id.0).0 == 1 {
                        if val.is_some() {
                            for i in (idx+1)..stack.len() {
                                children.push((id.0.clone(), Some(i)));
                            }
                        }
                        let mut count = 0;
                        for i in 0..idx {
                            if stack[i].is_none() {
                                count = count +1;
                            } else {
                                count = count + 2;
                            }
                        }
                        offset.y = GridValue(0, count);
                    }
                    let draggable = children.len() > 0 || match id.0 {
                        StackId(0,1) | StackId(1, _) => idx == stack.len() - 1,
                        _ => false
                    };
                    if id.0 == StackId(0,1) {
                        let mut start = stack.len() as i32 - 3;
                        if start < 0 {
                            start = 0;
                        }
                        if idx as i32 > start {
                            offset.x = GridValue(0, (idx as i32 - start)*2);
                        }
                    }
                    Some(CardData {
                        pos: get_stack_location(&id.0) + offset,
                        display: match val {
                            &Some(ref crd) => CardDisplay::Front(crd.clone()),
                            &None => CardDisplay::Back
                        },
                        drag_children: if draggable { Some(children) } else { None }
                    })
                } else {
                    None
                }
            } else {
                Some(CardData {
                    pos: get_stack_location(&id.0) - GridLocation::new(GridValue(0,0), GridValue(0,0), 1),
                    display: CardDisplay::Empty,
                    drag_children: None
                })
            }
        } else {
            None
        }
    }
    fn get_action_for(&self, act: MouseAction<Self::CardId>) -> Option<Self::Action> {
        match act {
            MouseAction::Drop(dragged, (stack, _)) => {
                if let Some(data) = self.get_data_for(dragged) {
                    if let CardDisplay::Front(crd) = data.display {
                        Some(CardGameAction::Move(crd, stack))
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            MouseAction::Tap((stack, _)) => {
                Some(CardGameAction::Tap(stack))
            }
        }
    }
    fn get_grid_extents() -> (GridLocation, GridLocation) {
        (GridLocation::new(GridValue(-1,0), GridValue(-1,0), -1), GridLocation::new(GridValue(13,0), GridValue(3, 32), 20))
    }
}



