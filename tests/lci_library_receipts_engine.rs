//! Distinct library picks and exact resolution-time receipts.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const SLOTS: LibrarySelectionSpec = LibrarySelectionSpec {
    look: 4,
    count: LibrarySelectionCount::DistinctCardTypes(1 | 2),
    filter: LibrarySelectionFilter::Any,
    optional: true,
    reveal: true,
    remainder: LibraryRemainder::Graveyard,
};
const REVEAL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: SLOTS,
            look: LibraryLook::Fixed(4),
            destination: LibraryPickDestination::Hand,
            public: true,
            maximum_mana_value: None,
        })],
    ),
    ..CardRules::EMPTY
};
const MULTITYPE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                count: LibrarySelectionCount::DistinctCardTypes(1 | 16),
                remainder: LibraryRemainder::BottomRandom,
                ..SLOTS
            },
            look: LibraryLook::Fixed(4),
            destination: LibraryPickDestination::Battlefield,
            public: true,
            maximum_mana_value: None,
        })],
    ),
    ..CardRules::EMPTY
};
const SURVEIL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::SurveilRememberTop(2),
            Instruction::IfAmountAtLeast {
                value: AmountValue::RememberedCount,
                minimum: 1,
                skip: 2,
            },
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::LoseLife(PlayerSet::Controller, 3),
            Instruction::IfAmountAtLeast {
                value: AmountValue::RememberedCount,
                minimum: 2,
                skip: 2,
            },
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::LoseLife(PlayerSet::Controller, 3),
        ],
    ),
    ..CardRules::EMPTY
};
const EXILE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ExileTopLibraryRemember {
                players: PlayerSet::Controller,
                amount: AmountValue::Fixed(2),
            },
            Instruction::GrantExiledResultPlay { next_turn: false },
            Instruction::IfExiledLibraryCard {
                land: true,
                skip: 1,
            },
            Instruction::GainLife(PlayerSet::Controller, 2),
            Instruction::IfExiledLibraryCard {
                land: false,
                skip: 1,
            },
            Instruction::GainLife(PlayerSet::Controller, 4),
        ],
    ),
    ..CardRules::EMPTY
};
const BOUNCE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::RememberTargetTapped,
            Instruction::ReturnToHand,
            Instruction::IfAmountAtLeast {
                value: AmountValue::RememberedCount,
                minimum: 1,
                skip: 1,
            },
            Instruction::GainLife(PlayerSet::Controller, 2),
        ],
    ),
    ..CardRules::EMPTY
};

fn opt_rules(rules: &'static CardRules) -> impl Drop {
    fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), rules)])
}

fn opt(game: &mut Game, player: Player) {
    let spell = put(game, player, "Opt", Zone::Hand);
    cast(game, spell, None);
}

fn pass_to_choice(game: &mut Game) {
    for _ in 0..8 {
        if game.state.pending != Pending::Priority {
            return;
        }
        act(game, Action::Pass);
    }
    panic!("expected a resolution choice");
}

#[test]
fn the_controller_can_pick_one_creature_and_one_land_from_publicly_revealed_cards() {
    let _rules = opt_rules(&REVEAL);
    let player = Player::P1;
    let mut game = scenario(player);
    let ineligible = put(&mut game, player, "Shock", Zone::Library);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Library);
    let land = put(&mut game, player, "Forest", Zone::Library);
    let other_creature = put(&mut game, player, "Bear Cub", Zone::Library);
    opt(&mut game, player);
    pass_to_choice(&mut game);
    let visible = game.observe(player.other()).library_selection.unwrap();
    assert!(visible.public);
    assert_eq!(visible.remaining.len(), 4);
    let checkpoint = game.snapshot();
    act(&mut game, Action::SelectLibrary { card: creature });
    reject(
        &mut game,
        Action::SelectLibrary {
            card: other_creature,
        },
    );
    reject(&mut game, Action::SelectLibrary { card: ineligible });
    act(&mut game, Action::SelectLibrary { card: land });
    act(&mut game, Action::FinishLibrarySelection);
    settle(&mut game);
    for card in [creature, land] {
        assert_eq!(game.object_view(card).zone, Zone::Hand);
    }
    for card in [ineligible, other_creature] {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }
    game.restore(&checkpoint).unwrap();
    act(&mut game, Action::FinishLibrarySelection);
    settle(&mut game);
    for card in [creature, land, ineligible, other_creature] {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }
}

#[test]
fn a_multitype_card_can_fill_either_slot_but_cannot_be_selected_twice() {
    let _rules = opt_rules(&MULTITYPE);
    let player = Player::P0;
    let mut game = scenario(player);
    let artifact = put(&mut game, player, "Swiftfoot Boots", Zone::Library);
    let both = put(&mut game, player, "Gleaming Barrier", Zone::Library);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Library);
    let remaining = put(&mut game, player, "Forest", Zone::Library);
    opt(&mut game, player);
    pass_to_choice(&mut game);
    let checkpoint = game.snapshot();
    for chosen in [artifact, creature] {
        game.restore(&checkpoint).unwrap();
        act(&mut game, Action::SelectLibrary { card: both });
        reject(&mut game, Action::SelectLibrary { card: both });
        act(&mut game, Action::SelectLibrary { card: chosen });
        act(&mut game, Action::FinishLibrarySelection);
        settle(&mut game);
        assert_eq!(game.object_view(both).zone, Zone::Battlefield);
        assert_eq!(game.object_view(chosen).zone, Zone::Battlefield);
        assert_eq!(game.object_view(remaining).zone, Zone::Library);
    }
}

#[test]
fn retained_surveillance_cards_trigger_one_draw_and_loss_each() {
    let _rules = opt_rules(&SURVEIL);
    for kept in 0..=2 {
        let player = Player::P0;
        let mut game = scenario(player);
        let second = put(&mut game, player, "Bear Cub", Zone::Library);
        let first = put(&mut game, player, "Grizzly Bears", Zone::Library);
        opt(&mut game, player);
        pass_to_choice(&mut game);
        assert!(game.observe(player).library_choice.is_some());
        assert!(game.observe(player.other()).library_choice.is_none());
        for (index, card) in [first, second].into_iter().enumerate() {
            let destination = if index < kept {
                LibraryDestination::Top
            } else {
                LibraryDestination::Graveyard
            };
            act(&mut game, Action::ArrangeLibrary { card, destination });
            let expected = if index < kept {
                Zone::Hand
            } else {
                Zone::Graveyard
            };
            if index == 1 {
                settle(&mut game);
                assert_eq!(game.object_view(card).zone, expected);
            }
        }
        let expected = if kept == 0 {
            Zone::Graveyard
        } else {
            Zone::Hand
        };
        assert_eq!(game.object_view(first).zone, expected);
        assert_eq!(
            game.state.players[player.index()].cards_drawn_this_turn,
            kept as u64
        );
        assert_eq!(
            game.state.players[player.index()].life,
            STARTING_LIFE - 3 * kept as i128
        );
    }
}

#[test]
fn land_and_nonland_exile_receipts_are_independent() {
    let _rules = opt_rules(&EXILE);
    let player = Player::P0;
    for (names, bonus) in [
        (["Forest", "Island"], 2),
        (["Forest", "Grizzly Bears"], 6),
        (["Hill Giant", "Grizzly Bears"], 4),
    ] {
        let mut game = scenario(player);
        let cards = names.map(|name| put(&mut game, player, name, Zone::Library));
        opt(&mut game, player);
        settle(&mut game);
        assert_eq!(
            game.state.players[player.index()].life,
            STARTING_LIFE + bonus
        );
        for card in cards {
            assert_eq!(game.object_view(card).zone, Zone::Exile);
        }
    }
}

#[test]
fn tapped_status_is_sampled_before_the_exact_target_returns_to_its_owner() {
    let _rules = opt_rules(&BOUNCE);
    let player = Player::P0;
    for tapped in [false, true] {
        let mut game = scenario(player);
        let target = put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        let spell = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(target)));
        game.state.objects[target as usize].tapped = tapped;
        settle(&mut game);
        assert_eq!(game.object_view(target).zone, Zone::Hand);
        assert_eq!(game.object_view(target).owner, player.other());
        assert_eq!(
            game.state.players[player.index()].life,
            STARTING_LIFE + 2 * i128::from(tapped)
        );
    }
}
