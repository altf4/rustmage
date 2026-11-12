//! Simultaneous creation for the players tied for most controlled creatures.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const CLUE: Card = Card::from_catalogue_id(5296);
const SWEEP: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::CreateTokensFor {
                players: PlayerSet::MostCreatures,
                token: CLUE,
                amount: 1,
                tapped: false,
            },
            Instruction::MoveBattlefield {
                filters: &[BattlefieldFilter {
                    kinds: 1,
                    relation: -1,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: u32::MAX,
                }],
                exile: false,
            },
        ],
    ),
    ..CardRules::EMPTY
};

fn resolve(game: &mut Game, player: Player, spell: Card) {
    let spell = spawn(game, player, spell, Zone::Hand);
    cast(game, spell, None);
    settle(game);
}

fn clues(game: &Game) -> [usize; 2] {
    let mut counts = [0; 2];
    for object in game.observe(Player::P0).objects {
        if object.zone == Zone::Battlefield && object.card == CLUE {
            counts[object.controller.index()] += 1;
        }
    }
    counts
}

#[test]
fn ties_at_zero_and_positive_counts_reward_both_players_before_the_sweep() {
    let spell = Card::from_name("Day of Judgment").unwrap();
    let _rules = fixture_rules::activate(&[(spell, &SWEEP)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut empty = scenario(player);
    resolve(&mut empty, player, spell);
    assert_eq!(clues(&empty), [1, 1]);

    let mut tied = scenario(player);
    let first = put(&mut tied, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut tied, opponent, "Bear Cub", Zone::Battlefield);
    put(&mut tied, opponent, "Short Bow", Zone::Battlefield);
    resolve(&mut tied, player, spell);
    assert_eq!(clues(&tied), [1, 1]);
    assert_eq!(tied.object_view(first).zone, Zone::Graveyard);
    assert_eq!(tied.object_view(second).zone, Zone::Graveyard);

    let mut unequal = scenario(player);
    let borrowed = put(&mut unequal, player, "Bear Cub", Zone::Battlefield);
    let exact = unequal.object_ref(borrowed);
    unequal.record_control(exact, opponent, None);
    resolve(&mut unequal, player, spell);
    assert_eq!(clues(&unequal), [1, 0]);
    assert_eq!(unequal.object_view(borrowed).zone, Zone::Graveyard);
}
