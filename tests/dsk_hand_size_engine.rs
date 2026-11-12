//! Dynamic hand-size setters use live graveyard card types and retain their timestamps.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const HAND_SETTER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[
            StaticAbilityFragment::OpponentMaximumHandSizeByGraveyardTypes {
                base: 7,
                minimum: 4,
            },
        ],
    }],
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn hand_size(game: &Game, viewer: Player) -> HandSizeTimelineView {
    game.observe(viewer).hand_size_timeline.unwrap()
}

fn assert_cleanup_limit(game: &Game, player: Player, maximum: usize) {
    let mut cleanup = game.clone();
    for _ in 0..8 {
        put(&mut cleanup, player, "Forest", Zone::Hand);
    }
    cleanup.state.step = Step::End;
    assert_eq!(cleanup.hand_count(player), 8);
    assert!(
        !cleanup
            .legal_actions()
            .iter()
            .any(|action| matches!(action, Action::Discard { .. }))
    );
    act(&mut cleanup, Action::Pass);
    act(&mut cleanup, Action::Pass);
    assert_eq!(cleanup.observe(player).step, Step::Cleanup);
    assert_eq!(cleanup.actor(), Some(player));

    let actions = cleanup.legal_actions();
    let index = actions
        .iter()
        .position(|action| matches!(action, Action::Discard { .. }))
        .unwrap();
    let checkpoint = cleanup.snapshot();
    let mut indexed = cleanup.clone();
    act(&mut cleanup, actions[index].clone());
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    assert_eq!(indexed.snapshot(), cleanup.snapshot());
    indexed.restore(&checkpoint).unwrap();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    assert_eq!(indexed.snapshot(), cleanup.snapshot());

    for _ in 1..8 - maximum {
        choose(&mut cleanup, |action| {
            matches!(action, Action::Discard { .. })
        });
    }
    assert_eq!(cleanup.hand_count(player), maximum);
    assert_eq!(cleanup.observe(player).active, player.other());
}

#[test]
fn only_the_current_controllers_distinct_physical_graveyard_types_set_the_opponents_limit() {
    let _rules = fixture_rules::activate(&[(named("Bear Cub"), &HAND_SETTER)]);
    for controller in [Player::P0, Player::P1] {
        let opponent = controller.other();
        let mut game = scenario(opponent);
        let source = put(&mut game, controller, "Bear Cub", Zone::Battlefield);
        put(&mut game, controller, "Gingerbrute", Zone::Graveyard);
        put(&mut game, controller, "Forest", Zone::Graveyard);
        put(&mut game, controller, "Forest", Zone::Graveyard);
        put(&mut game, opponent, "Unsummon", Zone::Graveyard);
        assert_eq!(hand_size(&game, controller).maximum, [Some(7), Some(7)]);
        assert_cleanup_limit(&game, opponent, 7);

        // Both Room names describe one enchantment card, contributing one type.
        let room = spawn(
            &mut game,
            controller,
            named("Roaring Furnace // Steaming Sauna"),
            Zone::Hand,
        );
        game.move_object(room, Zone::Graveyard);
        assert_eq!(game.object_view(room).kinds, 32);
        assert_eq!(
            hand_size(&game, opponent).maximum[opponent.index()],
            Some(3)
        );
        assert_cleanup_limit(&game, opponent, 3);

        put(&mut game, controller, "Divination", Zone::Graveyard);
        assert_eq!(
            hand_size(&game, opponent).maximum[opponent.index()],
            Some(2)
        );
        let kindred = put(&mut game, controller, "Nameless Inversion", Zone::Graveyard);
        assert_eq!(game.object_view(kindred).kinds, 8 | 128);
        assert_eq!(
            hand_size(&game, opponent).maximum[opponent.index()],
            Some(0)
        );
        put(
            &mut game,
            controller,
            "Chandra, Flameshaper",
            Zone::Graveyard,
        );
        assert_eq!(
            hand_size(&game, opponent).maximum[opponent.index()],
            Some(0)
        );
        assert_eq!(
            hand_size(&game, controller).maximum[controller.index()],
            Some(7)
        );
        assert_cleanup_limit(&game, opponent, 0);

        put(&mut game, opponent, "Gingerbrute", Zone::Graveyard);
        put(&mut game, opponent, "Forest", Zone::Graveyard);
        game.record_control(game.object_ref(source), opponent, None);
        let sizes = hand_size(&game, controller).maximum;
        assert_eq!(sizes[controller.index()], Some(3));
        assert_eq!(sizes[opponent.index()], Some(7));
    }
}

#[test]
fn turning_the_effect_off_and_back_on_keeps_its_order_against_an_unlimited_setter() {
    let _rules = fixture_rules::activate(&[(named("Bear Cub"), &HAND_SETTER)]);
    let controller = Player::P0;
    let opponent = controller.other();
    for limited_newer in [false, true] {
        let mut game = scenario(opponent);
        let unlimited =
            |game: &mut Game| put(game, opponent, "Ms. Marvel, Kamala Khan", Zone::Battlefield);
        let first = limited_newer.then(|| unlimited(&mut game));
        let source = put(&mut game, controller, "Bear Cub", Zone::Battlefield);
        let other = first.unwrap_or_else(|| unlimited(&mut game));
        put(&mut game, controller, "Gingerbrute", Zone::Graveyard);
        put(&mut game, controller, "Forest", Zone::Graveyard);
        assert_eq!(hand_size(&game, opponent).maximum[opponent.index()], None);

        let timestamp = |game: &Game, source: CardId| {
            hand_size(game, opponent)
                .sources
                .iter()
                .find(|row| row.source.card == source)
                .unwrap()
                .timestamp
        };
        let original = timestamp(&game, source);
        assert_eq!(original > timestamp(&game, other), limited_newer);
        let instant = put(&mut game, controller, "Unsummon", Zone::Graveyard);
        let expected = limited_newer.then_some(3);
        assert_eq!(
            hand_size(&game, opponent).maximum[opponent.index()],
            expected
        );

        let aura = put(&mut game, opponent, "Eaten by Piranhas", Zone::Hand);
        cast(&mut game, aura, Some(Target::Card(source)));
        settle(&mut game);
        assert_eq!(hand_size(&game, opponent).maximum[opponent.index()], None);
        assert_eq!(timestamp(&game, source), original);
        game.move_object(aura, Zone::Graveyard);
        assert_eq!(
            hand_size(&game, opponent).maximum[opponent.index()],
            expected
        );

        game.move_object(instant, Zone::Exile);
        assert_eq!(hand_size(&game, opponent).maximum[opponent.index()], None);
        game.move_object(instant, Zone::Graveyard);
        assert_eq!(
            hand_size(&game, opponent).maximum[opponent.index()],
            expected
        );
        assert_eq!(timestamp(&game, source), original);
    }
}
