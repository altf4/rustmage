mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn passes(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn drain(game: &mut Game) {
    for _ in 0..30 {
        if game.state.stack.is_empty()
            && game.state.resolving.is_none()
            && game.state.pending == Pending::Priority
            && game.state.pending_triggers.is_empty()
            && game.state.ordering_triggers.is_empty()
        {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::ArrangeLibrary {
                        destination: LibraryDestination::Top,
                        ..
                    }
            )
        });
    }
    panic!("layer card effects did not settle");
}

#[test]
fn beastbinder_ignores_big_blockers_and_applies_both_layers_until_its_next_turn() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Azure Beastbinder", Zone::Battlefield);
    let elf = put(
        &mut game,
        player.other(),
        "Llanowar Elves",
        Zone::Battlefield,
    );
    let one = put(
        &mut game,
        player.other(),
        "Healer's Hawk",
        Zone::Battlefield,
    );
    let two = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    game.add_counter(elf, CounterKind::PlusOne, 1);
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: source });
    act(&mut game, Action::FinishAttackers);
    act(&mut game, target(elf));
    drain(&mut game);
    assert!(!game.object_view(source).tapped);
    assert!(!game.printed_abilities_available(elf));
    assert_eq!(stats(&game, elf), (3, 3));
    game.state.step = Step::Blockers;
    game.state.pending = Pending::Blockers;
    assert!(game.legal_actions().contains(&Action::Block {
        blocker: one,
        attacker: source
    }));
    reject(
        &mut game,
        Action::Block {
            blocker: two,
            attacker: source,
        },
    );
    game.finish_cleanup();
    assert!(!game.printed_abilities_available(elf));
    game.finish_cleanup();
    assert!(game.printed_abilities_available(elf));
    assert_eq!(stats(&game, elf), (2, 2));
}

#[test]
fn charmed_sleep_skips_ordinary_untap_but_does_not_prevent_an_effect() {
    let player = Player::P0;
    let mut game = scenario(player);
    let target = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let aura = put(&mut game, player, "Charmed Sleep", Zone::Hand);
    cast(&mut game, aura, Some(Target::Card(target)));
    drain(&mut game);
    assert!(game.object_view(target).tapped);
    game.finish_cleanup();
    game.finish_cleanup();
    assert!(game.object_view(target).tapped);
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 0];
    let trick = put(&mut game, player, "Shore Up", Zone::Hand);
    cast(&mut game, trick, Some(Target::Card(target)));
    drain(&mut game);
    assert!(!game.object_view(target).tapped);
    assert_eq!(game.object_view(aura).zone, Zone::Battlefield);
}

#[test]
fn sugar_coat_flashes_and_gives_only_foods_activation_to_the_enchanted_permanent() {
    let player = Player::P1;
    let other = player.other();
    let mut game = scenario(other);
    let elf = put(&mut game, other, "Llanowar Elves", Zone::Battlefield);
    let aura = put(&mut game, player, "Sugar Coat", Zone::Hand);
    act(&mut game, Action::Pass);
    cast(&mut game, aura, Some(Target::Card(elf)));
    drain(&mut game);
    let view = game.object_view(elf);
    assert!(view.has_subtype("Food"));
    assert!(!view.has_subtype("Elf"));
    assert_eq!(view.kinds, 16);
    assert_eq!(view.colors, 0);
    if game.actor() != Some(other) {
        act(&mut game, Action::Pass);
    }
    assert!(!game.legal_actions().iter().any(|a| matches!(a,
        Action::ActivateMana { card, .. } | Action::TapMana { card } if *card == elf
    )));
    let grantor = game.object_ref(aura);
    act(
        &mut game,
        Action::AnnounceAttachedAbility {
            grantor,
            ability: 0,
        },
    );
    pay(&mut game);
    drain(&mut game);
    assert_eq!(game.object_view(elf).zone, Zone::Graveyard);
    assert_eq!(game.object_view(aura).zone, Zone::Graveyard);
    assert_eq!(game.state.players[other.index()].life, 23);
}

#[test]
fn eluge_counts_the_new_island_and_reduces_only_the_first_actual_spell() {
    let player = Player::P0;
    let mut game = scenario(player);
    let island = put(&mut game, player, "Island", Zone::Battlefield);
    let mountain = put(&mut game, player, "Mountain", Zone::Battlefield);
    let source = put(&mut game, player, "Eluge, the Shoreless Sea", Zone::Hand);
    cast(&mut game, source, None);
    passes(&mut game);
    act(&mut game, target(mountain));
    drain(&mut game);
    assert!(game.object_view(mountain).has_subtype("Island"));
    assert!(game.object_view(mountain).has_subtype("Mountain"));
    assert_eq!(stats(&game, source), (2, 2));
    for land in [island, mountain] {
        game.state.objects[land as usize].tapped = true;
    }
    game.state.players[player.index()].mana = [0; 6];
    let first = put(&mut game, player, "Opt", Zone::Hand);
    let second = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, first, None);
    assert_eq!(
        game.state.stack.last().unwrap().cast.unwrap().payment,
        [0; 6]
    );
    drain(&mut game);
    reject(&mut game, Action::AnnounceCast { card: second });
    game.pay_counters(mountain, &Counters::of(CounterKind::Flood, 1));
    assert!(!game.object_view(mountain).has_subtype("Island"));
    assert_eq!(stats(&game, source), (1, 1));
}

#[test]
fn floodcaller_grants_flash_only_to_noncreatures_and_counts_each_valley_member_once() {
    let player = Player::P1;
    let mut game = scenario(player.other());
    let bird_frog = put(&mut game, player, "Skyskipper Duo", Zone::Battlefield);
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    for card in [bird_frog, bear] {
        game.state.objects[card as usize].tapped = true;
    }
    game.add_counter(bird_frog, CounterKind::Stun, 1);
    let caller = put(&mut game, player, "Valley Floodcaller", Zone::Hand);
    let artifact = put(&mut game, player, "Swiftfoot Boots", Zone::Hand);
    let creature = put(&mut game, player, "Bear Cub", Zone::Hand);
    act(&mut game, Action::Pass);
    reject(&mut game, Action::AnnounceCast { card: artifact });
    cast(&mut game, caller, None);
    drain(&mut game);
    if game.actor() != Some(player) {
        act(&mut game, Action::Pass);
    }
    reject(&mut game, Action::AnnounceCast { card: creature });
    cast(&mut game, artifact, None);
    drain(&mut game);
    assert_eq!(game.object_view(artifact).zone, Zone::Battlefield);
    assert_eq!(stats(&game, bird_frog), (4, 4));
    assert_eq!(stats(&game, caller), (3, 3));
    assert!(game.object_view(bird_frog).tapped);
    assert_eq!(game.counter_count(bird_frog, CounterKind::Stun), 0);
    assert_eq!(stats(&game, bear), (2, 2));
    assert!(game.object_view(bear).tapped);
}
