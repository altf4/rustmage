//! Actual Star Trek T–Z card rules against the public catalogue.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn modal(game: &mut Game, spell: CardId, mode: u16, target: Option<Target>) {
    act(game, Action::AnnounceCast { card: spell });
    act(game, Action::ChooseMode { mode });
    if let Some(target) = target {
        act(game, Action::ChooseTarget { target });
    }
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
    pay(game);
}

#[test]
fn test_of_identity_restricts_klingon_target_and_checks_actual_discard() {
    let mut boost = scenario(Player::P0);
    let klingon = put(
        &mut boost,
        Player::P1,
        "Cha'DIch Investigator",
        Zone::Battlefield,
    );
    let other = put(&mut boost, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let spell = put(&mut boost, Player::P0, "Test of Identity", Zone::Hand);
    act(&mut boost, Action::AnnounceCast { card: spell });
    act(&mut boost, Action::ChooseMode { mode: 0 });
    reject(
        &mut boost,
        Action::ChooseTarget {
            target: Target::Card(other),
        },
    );
    act(
        &mut boost,
        Action::ChooseTarget {
            target: Target::Card(klingon),
        },
    );
    if boost.legal_actions().contains(&Action::FinishTargets) {
        act(&mut boost, Action::FinishTargets);
    }
    pay(&mut boost);
    settle(&mut boost);
    assert_eq!(boost.object_view(klingon).power, 5);
    assert_eq!(boost.object_view(klingon).toughness, 5);
    assert!(
        boost
            .object_view(klingon)
            .keywords
            .contains(Keywords::DOUBLE_STRIKE)
    );

    for has_card in [false, true] {
        let mut game = scenario(Player::P0);
        let chosen = has_card.then(|| put(&mut game, Player::P0, "Forest", Zone::Hand));
        put(&mut game, Player::P0, "Island", Zone::Library);
        put(&mut game, Player::P0, "Mountain", Zone::Library);
        let spell = put(&mut game, Player::P0, "Test of Identity", Zone::Hand);
        modal(&mut game, spell, 1, None);
        if let Some(card) = chosen {
            while !matches!(game.state.pending, Pending::EffectDiscard { .. }) {
                choose(&mut game, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
            act(&mut game, Action::Discard { card });
            assert_eq!(game.object_view(card).zone, Zone::Graveyard);
        }
        settle(&mut game);
        assert_eq!(
            game.state.players[0].cards_drawn_this_turn,
            if has_card { 2 } else { 0 }
        );
    }
}

#[test]
fn vulcan_mind_meld_exiles_privately_for_the_correct_players() {
    for caster in [Player::P0, Player::P1] {
        let opponent = caster.other();
        let mut game = scenario(caster);
        let own = put(&mut game, caster, "Hill Giant", Zone::Library);
        let theirs_a = put(&mut game, opponent, "Forest", Zone::Library);
        let theirs_b = put(&mut game, opponent, "Shock", Zone::Library);
        let spell = put(&mut game, caster, "Vulcan Mind-Meld", Zone::Hand);
        cast(&mut game, spell, Some(Target::Player(opponent)));
        settle(&mut game);
        for (card, beneficiary) in [(own, opponent), (theirs_a, caster), (theirs_b, caster)] {
            assert_eq!(game.object_view(card).zone, Zone::Exile);
            let permission = game
                .state
                .exile_permissions
                .iter()
                .find(|p| p.object == game.object_ref(card))
                .unwrap();
            assert_eq!(permission.player, beneficiary);
            assert_eq!(permission.expiry, ExilePlayExpiry::WhileExiled);
            assert!(permission.spend_any_mana);
            assert!(
                game.observe(beneficiary)
                    .objects
                    .iter()
                    .any(|o| o.id == card)
            );
            assert!(
                !game
                    .observe(beneficiary.other())
                    .objects
                    .iter()
                    .any(|o| o.id == card)
            );
        }
    }
}
