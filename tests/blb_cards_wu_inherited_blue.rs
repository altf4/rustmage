mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn progress(game: &mut Game) {
    choose(game, |a| {
        matches!(a, Action::Pass | Action::PutTrigger { .. })
    });
}

fn until(game: &mut Game, action: Action) {
    for _ in 0..20 {
        if game.legal_actions().contains(&action) {
            return;
        }
        progress(game);
    }
    panic!("missing {action:?} in {:?}", game.state.pending);
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn arranger(card: CardId, destination: LibraryDestination) -> Action {
    Action::ArrangeLibrary { card, destination }
}

#[test]
fn crier_loots_and_lightshell_duo_surveils_two_and_has_prowess() {
    let player = Player::P0;
    let mut game = scenario(player);
    let discard = put(&mut game, player, "Island", Zone::Library);
    let crier = put(&mut game, player, "Bellowing Crier", Zone::Hand);
    cast(&mut game, crier, None);
    let choose_discard = Action::Discard { card: discard };
    until(&mut game, choose_discard.clone());
    act(&mut game, choose_discard);
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    let keep = put(&mut game, player, "Forest", Zone::Library);
    let mill = put(&mut game, player, "Island", Zone::Library);
    let duo = put(&mut game, player, "Lightshell Duo", Zone::Hand);
    cast(&mut game, duo, None);
    let graveyard = arranger(mill, LibraryDestination::Graveyard);
    until(&mut game, graveyard.clone());
    act(&mut game, graveyard);
    act(&mut game, arranger(keep, LibraryDestination::Top));
    settle(&mut game);
    assert_eq!(game.object_view(mill).zone, Zone::Graveyard);
    let spell = put(&mut game, player, "Divination", Zone::Hand);
    cast(&mut game, spell, None);
    settle(&mut game);
    assert_eq!(stats(&game, duo), (4, 5));
    assert_eq!(game.object_view(keep).zone, Zone::Hand);
}

#[test]
fn dire_downdraft_uses_target_dependent_cost_and_the_owner_chooses_where_it_goes() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let bear = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    game.record_control(game.object_ref(bear), player, None);
    let spell = put(&mut game, player, "Dire Downdraft", Zone::Hand);
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 2];
    reject(&mut game, Action::AnnounceCast { card: spell });
    game.state.objects[bear as usize].tapped = true;
    cast(&mut game, spell, Some(Target::Card(bear)));
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    let bottom = Action::ChooseLibraryDestination {
        object: game.object_ref(bear),
        destination: LibraryDestination::Bottom,
    };
    until(&mut game, bottom.clone());
    assert_eq!(game.actor(), Some(opponent));
    act(&mut game, bottom);
    settle(&mut game);
    assert_eq!(
        game.state.players[opponent.index()].library.first(),
        Some(&bear)
    );
}

#[test]
fn eddymurk_crab_uses_both_spell_types_for_its_discount_and_enters_tapped_off_turn() {
    let player = Player::P1;
    let other = player.other();
    let mut game = scenario(other);
    let first = put(&mut game, other, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, other, "Bear Cub", Zone::Battlefield);
    let crab = put(&mut game, player, "Eddymurk Crab", Zone::Hand);
    put(&mut game, player, "Opt", Zone::Graveyard);
    put(&mut game, player, "Divination", Zone::Graveyard);
    game.state.players[player.index()].mana = [0, 2, 0, 0, 0, 3];
    act(&mut game, Action::Pass);
    cast(&mut game, crab, None);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    until(&mut game, target(first));
    assert!(game.object_view(crab).tapped);
    act(&mut game, target(first));
    act(&mut game, target(second));
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    settle(&mut game);
    assert!(game.object_view(first).tapped && game.object_view(second).tapped);
}

#[test]
fn knightfisher_makes_fish_for_another_nontoken_controlled_bird() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Knightfisher", Zone::Hand);
    cast(&mut game, source, None);
    settle(&mut game);
    assert!(game.object_view(source).keywords.contains(Keywords::FLYING));
    let count = |game: &Game| {
        game.observe(player)
            .objects
            .into_iter()
            .filter(|o| {
                o.zone == Zone::Battlefield
                    && o.is_token()
                    && o.controller == player
                    && o.has_subtype("Fish")
                    && o.colors == 2
            })
            .count()
    };
    game.create_tokens(player, Card(5616), 1, false);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(count(&game), 0);
    let bird = put(&mut game, player, "Healer's Hawk", Zone::Hand);
    cast(&mut game, bird, None);
    settle(&mut game);
    assert_eq!(count(&game), 1);
}

#[test]
fn nightwhorl_hermit_rechecks_threshold_and_pearl_reduces_only_once() {
    let player = Player::P0;
    let mut game = scenario(player);
    let hermit = put(&mut game, player, "Nightwhorl Hermit", Zone::Battlefield);
    for _ in 0..6 {
        put(&mut game, player, "Forest", Zone::Graveyard);
    }
    assert_eq!(stats(&game, hermit), (1, 4));
    assert!(
        !game
            .object_view(hermit)
            .keywords
            .contains(Keywords::CANT_BE_BLOCKED)
    );
    let seventh = put(&mut game, player, "Forest", Zone::Graveyard);
    assert_eq!(stats(&game, hermit), (2, 4));
    assert!(
        game.object_view(hermit)
            .keywords
            .contains(Keywords::VIGILANCE.union(Keywords::CANT_BE_BLOCKED))
    );
    game.move_object(seventh, Zone::Exile);
    assert_eq!(stats(&game, hermit), (1, 4));
    assert!(
        game.object_view(hermit)
            .keywords
            .contains(Keywords::VIGILANCE)
    );
    assert!(
        !game
            .object_view(hermit)
            .keywords
            .contains(Keywords::CANT_BE_BLOCKED)
    );

    let pearl = put(&mut game, player, "Pearl of Wisdom", Zone::Hand);
    put(
        &mut game,
        player.other(),
        "Thieving Otter",
        Zone::Battlefield,
    );
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 1];
    reject(&mut game, Action::AnnounceCast { card: pearl });
    put(&mut game, player, "Thieving Otter", Zone::Battlefield);
    put(&mut game, player, "Thieving Otter", Zone::Battlefield);
    let mut cannot_double = game.clone();
    cannot_double.state.players[player.index()].mana = [0, 1, 0, 0, 0, 0];
    reject(&mut cannot_double, Action::AnnounceCast { card: pearl });
    cast(&mut game, pearl, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
}

#[test]
fn plumecreed_can_flash_in_to_make_its_controllers_creature_an_illegal_target() {
    let player = Player::P1;
    let other = player.other();
    let mut game = scenario(other);
    let own = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, other, "Bear Cub", Zone::Battlefield);
    let escort = put(&mut game, player, "Plumecreed Escort", Zone::Hand);
    let removal = put(&mut game, other, "Murder", Zone::Hand);
    cast(&mut game, removal, Some(Target::Card(own)));
    if game.actor() != Some(player) {
        act(&mut game, Action::Pass);
    }
    cast(&mut game, escort, None);
    until(&mut game, target(own));
    reject(&mut game, target(enemy));
    act(&mut game, target(own));
    settle(&mut game);
    assert_eq!(game.object_view(own).zone, Zone::Battlefield);
    assert!(game.object_view(own).keywords.contains(Keywords::HEXPROOF));
    assert!(game.object_view(escort).keywords.contains(Keywords::FLYING));
}

#[test]
fn spellgyre_can_counter_or_choose_surveil_before_drawing() {
    let player = Player::P0;
    for counter in [false, true] {
        let opponent = player.other();
        let mut game = scenario(if counter { opponent } else { player });
        let source = put(&mut game, player, "Spellgyre", Zone::Hand);
        let lower = put(&mut game, player, "Forest", Zone::Library);
        let higher = put(&mut game, player, "Island", Zone::Library);
        let enemy_spell = put(&mut game, opponent, "Grizzly Bears", Zone::Hand);
        if counter {
            cast(&mut game, enemy_spell, None);
            if game.actor() != Some(player) {
                act(&mut game, Action::Pass);
            }
        }
        act(&mut game, Action::AnnounceCast { card: source });
        act(
            &mut game,
            Action::ChooseMode {
                mode: u16::from(!counter),
            },
        );
        if counter {
            act(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Spell(enemy_spell),
                },
            );
        }
        pay(&mut game);
        if counter {
            settle(&mut game);
            assert_eq!(game.object_view(enemy_spell).zone, Zone::Graveyard);
        } else {
            let graveyard = arranger(higher, LibraryDestination::Graveyard);
            until(&mut game, graveyard.clone());
            act(&mut game, graveyard);
            act(&mut game, arranger(lower, LibraryDestination::Top));
            settle(&mut game);
            assert_eq!(game.object_view(higher).zone, Zone::Graveyard);
            assert_eq!(game.object_view(lower).zone, Zone::Hand);
            assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
        }
    }
}

#[test]
fn thieving_otter_draws_for_actual_damage_to_an_opponent_in_or_out_of_combat() {
    let player = Player::P0;
    for combat in [false, true] {
        let mut game = scenario(player);
        let otter = put(&mut game, player, "Thieving Otter", Zone::Battlefield);
        let opponent_creature = put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        let source = game.damage_source(otter);
        let damage = DamageEvent {
            preventable: true,
            source,
            target: Target::Card(opponent_creature),
            amount: 1,
            combat,
        };
        game.apply_damage_batch(&[damage]);
        game.prepare_triggers();
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
        game.apply_damage_batch(&[DamageEvent {
            target: Target::Player(player.other()),
            ..damage
        }]);
        game.prepare_triggers();
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    }
}

#[test]
fn mind_spring_draws_exactly_the_paid_x() {
    let player = Player::P0;
    for value in [0, 3] {
        let mut game = scenario(player);
        let source = put(&mut game, player, "Mind Spring", Zone::Hand);
        game.state.players[player.index()].mana = [0, 2, 0, 0, 0, value];
        act(&mut game, Action::AnnounceCast { card: source });
        act(&mut game, Action::ChooseX { value });
        pay(&mut game);
        settle(&mut game);
        assert_eq!(
            game.state.players[player.index()].cards_drawn_this_turn,
            u64::from(value)
        );
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    }
}
