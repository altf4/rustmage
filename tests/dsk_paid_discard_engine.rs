//! Paid discard characteristics follow the exact successful cost event and its copies.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules, one};
use rustmage_core::{cards::*, engine::*};
use support::*;

const DRAW_DAMAGE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::Draw(PlayerSet::Controller, 2),
            Instruction::IfPaidDiscardWasNonland { skip: 1 },
            Instruction::DamagePlayers(PlayerSet::Opponent, 2),
        ],
    ),
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::Discard,
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 0,
                nonland: false,
                other: false,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            },
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const COPY: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Spell),
        &[Instruction::CopyTargetSpell {
            group: 0,
            retarget: false,
            exceptions: CopyExceptions::NONE,
        }],
    ),
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn discard(game: &Game, card: CardId) -> Action {
    Action::ChooseAdditionalCost {
        object: game.object_ref(card),
    }
}

fn receipt(game: &Game) -> PaidAdditionalCostView {
    *game
        .state
        .stack
        .last()
        .unwrap()
        .paid_additional_cost
        .as_deref()
        .expect("committed paid discard")
}

#[test]
fn the_selected_card_is_private_until_payment_and_its_frozen_type_survives_restore() {
    let _rules = fixture_rules::activate(&[(card("Seize the Spoils"), &DRAW_DAMAGE)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Seize the Spoils", Zone::Hand);
    let land = put(&mut game, player, "Forest", Zone::Hand);
    let creature = put(&mut game, player, "Bear Cub", Zone::Hand);
    let enemy = put(&mut game, opponent, "Shock", Zone::Hand);
    let anchor = put(&mut game, player, "Deeproot Pilgrimage", Zone::Battlefield);
    let foreign = put(&mut game, opponent, "Savannah Lions", Zone::Exile);
    game.state.exile_links.push(ExileLink {
        ability_origin: None,
        source: game.object_ref(anchor),
        exiled: game.object_ref(foreign),
        return_zone: Zone::Battlefield,
    });
    // An aborted cast restores the hand while retaining the spell's public identity.
    let mut expected_abort = game.clone();
    expected_abort.state.known_hand.push(game.object_ref(spell));
    act(&mut game, Action::AnnounceCast { card: spell });
    let invalid = discard(&game, enemy);
    reject(&mut game, invalid);

    let nonland = discard(&game, creature);
    let land_choice = discard(&game, land);
    let revision = game.revision();
    let saved = game.snapshot();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == nonland)
        .unwrap();
    let mut raw = game.clone();
    act(&mut raw, nonland);
    let mut aborted = raw.clone();
    assert!(aborted.state.cast_rollback.is_some());
    aborted.finish_cast_rollback(false);
    assert_eq!(aborted.snapshot(), expected_abort.snapshot());

    game.apply_index(player, revision, index).unwrap();
    assert_eq!(game.snapshot(), raw.snapshot());
    assert!(game.observe(opponent).additional_cost.is_none());
    assert!(
        game.observe(opponent)
            .objects
            .iter()
            .all(|object| object.id != creature)
    );
    assert!(
        game.state
            .announcing
            .as_ref()
            .unwrap()
            .item
            .paid_additional_cost
            .is_none()
    );

    game.restore(&saved).unwrap();
    assert_eq!(
        game.apply_index(player, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut game, land_choice);
    for (branch, paid, kinds, life) in [(&mut raw, creature, 1, 18), (&mut game, land, 2, 20)] {
        pay(branch);
        assert_eq!(branch.object_view(paid).zone, Zone::Graveyard);
        assert_eq!(receipt(branch).discarded_kinds, Some(kinds));
        branch.move_object(paid, Zone::Exile);
        settle(branch);
        assert_eq!(
            branch.state.players[player.index()].cards_drawn_this_turn,
            2
        );
        assert_eq!(branch.state.players[opponent.index()].life, life);
    }
}

#[test]
fn competing_discard_replacements_commit_the_same_types_for_exile_or_reveal_shuffle() {
    let _rules = fixture_rules::activate(&[(card("Seize the Spoils"), &DRAW_DAMAGE)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Seize the Spoils", Zone::Hand);
    let shuffler = put(&mut game, player, "Progenitus", Zone::Hand);
    put(&mut game, opponent, "Rest in Peace", Zone::Battlefield);
    act(&mut game, Action::AnnounceCast { card: spell });
    let choose_discard = discard(&game, shuffler);
    act(&mut game, choose_discard);
    pay(&mut game);
    assert_eq!(game.actor(), Some(player));
    assert!(game.observe(player).zone_replacement_choice.is_some());
    assert!(game.observe(opponent).zone_replacement_choice.is_none());

    for (effect, zone) in [
        (ZoneReplacementKind::Exile, Zone::Exile),
        (ZoneReplacementKind::RevealShuffle, Zone::Library),
    ] {
        let mut branch = game.clone();
        act(&mut branch, Action::ChooseZoneReplacement { effect });
        assert_eq!(branch.object_view(shuffler).zone, zone);
        assert_eq!(receipt(&branch).discarded_kinds, Some(1));
        if effect == ZoneReplacementKind::RevealShuffle {
            assert!(
                branch.observe(opponent).zone_reveals.iter().any(|reveal| {
                    reveal.card == card("Progenitus") && reveal.from == Zone::Hand
                })
            );
        }
        settle(&mut branch);
        assert_eq!(branch.state.players[opponent.index()].life, 18);
        assert_eq!(
            branch.state.players[player.index()].cards_drawn_this_turn,
            2
        );
    }
}

#[test]
fn a_spell_copy_keeps_its_original_discard_receipt_without_paying_it_again() {
    let _rules = fixture_rules::activate(&[
        (card("Seize the Spoils"), &DRAW_DAMAGE),
        (card("Opt"), &COPY),
    ]);
    let player = Player::P1;
    for (name, life) in [("Bear Cub", 16), ("Forest", 20)] {
        let mut game = scenario(player);
        let original = put(&mut game, player, "Seize the Spoils", Zone::Hand);
        let paid = put(&mut game, player, name, Zone::Hand);
        act(&mut game, Action::AnnounceCast { card: original });
        let choose_discard = discard(&game, paid);
        act(&mut game, choose_discard);
        pay(&mut game);
        let before = receipt(&game);

        let copy = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, copy, Some(Target::Spell(original)));
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        let item = game.state.stack.last().unwrap();
        assert!(item.is_copy);
        assert_eq!(item.cast.unwrap().payment, [0; 6]);
        assert_eq!(receipt(&game), before);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 4);
        assert_eq!(
            game.state.players[player.index()].cards_discarded_this_turn,
            1
        );
        assert_eq!(game.state.players[player.other().index()].life, life);
    }
}

#[test]
fn moving_an_unrevealed_card_to_a_hidden_destination_does_not_expose_its_types() {
    let player = Player::P0;
    let mut game = scenario(player);
    let hidden = put(&mut game, player, "Bear Cub", Zone::Hand);
    let library_move = game.move_object(hidden, Zone::Library);
    assert_eq!(game.discarded_card_kinds(&library_move), None);

    let hidden = put(&mut game, player, "Forest", Zone::Hand);
    let exile_move = game.move_object(hidden, Zone::Exile);
    assert_eq!(game.discarded_card_kinds(&exile_move), Some(2));
    game.remember_face_down_exile(exile_move.after, 1 << player.index(), None);
    assert_eq!(game.discarded_card_kinds(&exile_move), None);
}
