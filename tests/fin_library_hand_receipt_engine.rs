//! Private selections expose only actual selected hand moves to their follow-up.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::engine::zone_replacements;
use rustmage_core::{cards::*, engine::*};
use support::*;

const TAKE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::RememberCount(AmountValue::Fixed(7)),
            Instruction::SelectLibraryToHandRememberCount(LibrarySelectionEffect {
                spec: LibrarySelectionSpec {
                    look: 1,
                    count: LibrarySelectionCount::Fixed(1),
                    filter: LibrarySelectionFilter::Kinds(1 | 16),
                    optional: true,
                    reveal: true,
                    remainder: LibraryRemainder::Keep,
                },
                look: LibraryLook::Fixed(1),
                destination: LibraryPickDestination::Hand,
                public: false,
                maximum_mana_value: None,
            }),
            Instruction::GainLifeScaled(PlayerSet::Controller, AmountValue::MovedCount),
        ],
    ),
    ..CardRules::EMPTY
};

fn begin(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn staged() -> (Game, CardId) {
    let player = Player::P0;
    let mut game = scenario(player);
    let top = put(&mut game, player, "Swiftfoot Boots", Zone::Library);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, None);
    (game, top)
}

#[test]
fn ineligible_and_declined_private_looks_have_the_same_opponent_observation() {
    let opt = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(opt, &TAKE)]);
    let player = Player::P0;
    let opponent = player.other();
    let (mut eligible, top) = staged();
    let mut ineligible = eligible.clone();
    ineligible.state.objects[top as usize].card = Card::Forest;
    for game in [&mut eligible, &mut ineligible] {
        begin(game);
        assert_eq!(
            game.state.pending,
            Pending::EffectLibrarySelection { player }
        );
        assert!(game.observe(opponent).library_selection.is_none());
        assert!(game.observe(opponent).zone_reveals.is_empty());
    }
    let exact = eligible.object_ref(top);
    let eligible_view = eligible.observe(player).library_selection.unwrap();
    assert_eq!(eligible_view.candidates, [exact]);
    let ineligible_view = ineligible.observe(player).library_selection.unwrap();
    assert_eq!(ineligible_view.remaining[0].card, Card::Forest);
    assert!(ineligible_view.candidates.is_empty());
    assert_eq!(
        serde_json::to_value(eligible.observe(opponent)).unwrap(),
        serde_json::to_value(ineligible.observe(opponent)).unwrap()
    );
    reject(&mut ineligible, Action::SelectLibrary { card: top });
    for game in [&mut eligible, &mut ineligible] {
        act(game, Action::FinishLibrarySelection);
        assert_eq!(game.object_view(top).zone, Zone::Library);
        assert_eq!(game.state.players[player.index()].life, 20);
        assert_eq!(game.private_library_top(player, player), Some(exact));
        assert_eq!(game.private_library_top(opponent, player), None);
        assert!(game.observe(opponent).zone_reveals.is_empty());
    }
    assert_eq!(
        serde_json::to_value(eligible.observe(opponent)).unwrap(),
        serde_json::to_value(ineligible.observe(opponent)).unwrap()
    );
}

#[test]
fn the_receipt_counts_only_a_committed_hand_move_and_replays_an_owned_choice() {
    struct ClearZoneRules;
    impl Drop for ClearZoneRules {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|slot| slot.set(None));
            zone_replacements::TEST_EXILE_INSTEAD_OF_HAND.with(|slot| slot.set(None));
        }
    }
    let _clear = ClearZoneRules;
    let opt = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(opt, &TAKE)]);
    let player = Player::P0;
    let opponent = player.other();
    let (mut game, top) = staged();
    begin(&mut game);
    let card = game.object_view(top).card;
    let exact = game.object_ref(top);
    act(&mut game, Action::SelectLibrary { card: top });
    assert!(game.observe(opponent).zone_reveals.is_empty());
    let finish = Action::FinishLibrarySelection;
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == finish)
        .unwrap();
    let revision = game.revision();
    let saved = game.snapshot();
    let mut replay = game.clone();
    assert_eq!(game.apply(opponent, finish.clone()), Err(Error::WrongActor));
    assert_eq!(game.snapshot(), saved);
    act(&mut game, finish.clone());
    replay.apply_index(player, revision, index).unwrap();
    assert_eq!(game.snapshot(), replay.snapshot());
    assert_eq!(game.state.players[player.index()].life, 21);
    assert_eq!(game.object_view(top).zone, Zone::Hand);
    let view = game.observe(opponent);
    assert!(view.known_hand.contains(&game.object_ref(top)));
    assert!(
        view.zone_reveals
            .iter()
            .any(|event| event.card == card && event.from == Zone::Library)
    );

    for (prevent, destination) in [(true, Zone::Library), (false, Zone::Exile)] {
        replay.restore(&saved).unwrap();
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|slot| slot.set(prevent.then_some(exact)));
        zone_replacements::TEST_EXILE_INSTEAD_OF_HAND
            .with(|slot| slot.set((!prevent).then_some(exact)));
        act(&mut replay, finish.clone());
        assert_eq!(replay.object_view(top).zone, destination);
        assert_eq!(replay.state.players[player.index()].life, 20);
        let view = replay.observe(opponent);
        assert!(view.known_hand.is_empty());
        assert!(
            view.zone_reveals
                .iter()
                .any(|event| event.card == card && event.from == Zone::Library)
        );
        if prevent {
            assert_eq!(
                view.known_library_top[player.index()].unwrap().object,
                exact
            );
        }
    }
}

#[test]
fn an_empty_look_clears_the_previous_count_without_a_choice() {
    let opt = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(opt, &TAKE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let library = game.state.players[player.index()].library.clone();
    for card in library {
        game.move_object(card, Zone::Exile);
    }
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 20);
    assert!(game.observe(player).library_selection.is_none());
}
