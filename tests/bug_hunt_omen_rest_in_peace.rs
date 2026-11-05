mod support;

use rustmage as _;
use rustmage_core::engine::*;
use support::*;

fn announce_omen(game: &mut Game, player: Player, card: CardId) {
    game.state.players[player.index()].mana = [1, 0, 0, 0, 0, 1];
    act(game, Action::AnnounceOmen { card });
    pay(game);
}

#[test]
fn resolving_omen_lets_its_controller_choose_between_exile_and_shuffle() {
    let player = Player::P0;
    let mut game = scenario(player);
    let card = put(&mut game, player, "Riling Dawnbreaker", Zone::Hand);
    let initial_rng = game.state.rng;
    let mut ordinary = game.clone();
    announce_omen(&mut ordinary, player, card);
    settle(&mut ordinary);
    assert_eq!(ordinary.object_view(card).zone, Zone::Library);
    assert_ne!(ordinary.state.rng, initial_rng);
    assert!(ordinary.observe(player).zone_reveals.is_empty());

    put(
        &mut game,
        player.other(),
        "Rest in Peace",
        Zone::Battlefield,
    );
    announce_omen(&mut game, player, card);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(player));
    let actions = game.legal_actions();
    assert_eq!(
        actions
            .iter()
            .filter(|action| matches!(action, Action::ChooseZoneReplacement { .. }))
            .count(),
        2,
        "{actions:?}"
    );
    let checkpoint = game.snapshot();
    for (effect, destination, shuffled) in [
        (ZoneReplacementKind::Exile, Zone::Exile, false),
        (ZoneReplacementKind::Shuffle, Zone::Library, true),
    ] {
        let action = Action::ChooseZoneReplacement { effect };
        let index = actions.iter().position(|option| *option == action).unwrap();
        let mut raw = game.clone();
        act(&mut raw, action);
        assert_eq!(raw.object_view(card).zone, destination);
        assert_eq!(raw.state.rng != initial_rng, shuffled);
        assert!(raw.observe(player).zone_reveals.is_empty());
        assert_eq!(
            raw.observe(player)
                .objects
                .iter()
                .filter(|o| {
                    o.zone == Zone::Battlefield && o.is_token() && o.has_subtype("Soldier")
                })
                .count(),
            1
        );
        let mut indexed = game.clone();
        indexed.apply_index(player, game.revision(), index).unwrap();
        assert_eq!(indexed.snapshot(), raw.snapshot());
        indexed.restore(&checkpoint).unwrap();
        assert_eq!(indexed.snapshot(), game.snapshot());
    }
}

#[test]
fn a_stack_exit_replacement_can_apply_inside_the_omen_shuffle() {
    let player = Player::P0;
    let mut game = scenario(player);
    let card = put(&mut game, player, "Riling Dawnbreaker", Zone::Hand);
    put(
        &mut game,
        player.other(),
        "Rest in Peace",
        Zone::Battlefield,
    );
    announce_omen(&mut game, player, card);
    // Exercise the generic stack-exit replacement on the announced Omen.
    game.state.objects[card as usize].flashback = true;
    let initial_rng = game.state.rng;
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    act(
        &mut game,
        Action::ChooseZoneReplacement {
            effect: ZoneReplacementKind::Shuffle,
        },
    );
    assert_eq!(game.object_view(card).zone, Zone::Exile);
    assert_ne!(game.state.rng, initial_rng);
    assert!(game.observe(player).zone_reveals.is_empty());
}
