//mod game;
use rand;
use rand::Rng;
use game::cards::*;
use game::solitaire::{StackId, CardGamePercept, Solitaire};
use game::problem::Problem;
use std::char;
use std;

fn card_str(&(_, ref card): &(Ident, Option<Card>)) -> String {
    match card {
        &Some(ref card) => {
            let suit = match card.suit {
                0 => '♠',
                1 => '♡',
                2 => '♣',
                3 => '♢',
                _ => '?'
            };
            let rank = match card.rank {
                0 => String::from("A "),
                10 => String::from("J "),
                11 => String::from("Q "),
                12 => String::from("K "),
                x => format!("{: <2}",x+1)
            };
            format!("{}{}",suit, rank)
        },
        &None => String::from("###")
    }
}

fn print_percept(p: &CardGamePercept) {
    let deck = p.stacks.get(&StackId(0,0)).unwrap();
    let runoff = p.stacks.get(&StackId(0,1)).unwrap();

    if let Some(card) = deck.last() {
        print!("{} ", card_str(&(Ident::new(0),None)));
    } else {
        print!("    ");
    }
    if let Some(card) = runoff.last() {
        print!("{} ", card_str(card));
    } else {
        print!("    ");
    }
    print!("   ");
    for i in 0..4 {
        if let Some(card) = p.stacks.get(&StackId(2,i)).unwrap().last() {
            print!(" {}", card_str(card));
        } else {
            print!("    ");
        }
    }
    println!();

    let mut has = true;
    let mut row = 0;
    while has {
        has = false;
        for i in 0..7 {
            if let Some(thing) = p.stacks.get(&StackId(1,i)).unwrap().get(row) {
                print!("{} ", card_str(thing));
                has = true;
            } else {
                print!("    ");
            }
        }
        row = row+1;
        println!();
    }
}

pub fn deck() -> Vec<Card> {
    let mut deck = Vec::new();
    for i in 0..4 {
        for j in 0..13 {
            deck.push(Card::new(i, i%2, j));
        }
    }
    deck
}

pub fn deal_with_it() -> Solitaire {
    let mut deck = Vec::new();
    for i in 0..4 {
        for j in 0..13 {
            deck.push(Card::new(i, i%2, j));
        }
    }
    {
        let mut rng = rand::thread_rng();
        let len = deck.len();
        for i in 0..deck.len()-1 {
            deck.swap(i, rng.gen_range(i,len));
        }
    }
    Solitaire::deal(deck, 7,4)
}

pub  fn cmd_solitare() {
    let mut deck = Vec::new();
    for i in 0..4 {
        for j in 0..13 {
            deck.push(Card::new(i, i%2, j));
        }
    }
    {
        let mut rng = rand::thread_rng();
        let len = deck.len();
        for i in 0..deck.len()-1 {
            deck.swap(i, rng.gen_range(i,len));
        }
    }
    println!("{:?}", deck);
    let mut stdin = std::io::stdin();
    let mut sol = Solitaire::deal(deck, 7,4);
    loop {
        print_percept(&sol.percept());
        println!("select an action:");
        let actions = sol.actions();
        for (i,act) in actions.iter().enumerate() {
            println!("{}{:?}", i, act);
        }
        let mut act = None;
        while act == None {
            let mut thing = String::new();
            stdin.read_line(&mut thing).unwrap();
            let selection: usize = thing.trim().parse().unwrap();
            act = actions.get(selection);
        }
        println!("{:?}",act);
        sol = sol.result(act.unwrap().clone());
    }
}
