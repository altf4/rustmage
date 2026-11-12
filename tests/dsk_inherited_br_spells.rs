//! Paid card scenarios for inherited black and red Duskmourn spells and cycling.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const PLAYER: Player = Player::P0;
const OTHER: Player = Player::P1;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn cast_target(game: &mut Game, name: &str, card: CardId) {
    let spell = put(game, PLAYER, name, Zone::Hand);
    cast(game, spell, Some(Target::Card(card)));
}

#[test]
fn the_inherited_targeted_removal_spells_apply_their_printed_follow_ons() {
    for (spell, victim, zone, life) in [
        ("Murder", "Hill Giant", Zone::Graveyard, 20),
        ("Scorching Dragonfire", "Hill Giant", Zone::Exile, 20),
        (
            "Winter's Intervention",
            "Grizzly Bears",
            Zone::Graveyard,
            22,
        ),
        ("Withering Torment", "Grizzly Bears", Zone::Graveyard, 18),
        ("Withering Torment", "Ethereal Armor", Zone::Graveyard, 18),
    ] {
        let mut game = scenario(PLAYER);
        let creature = put(&mut game, OTHER, "Grizzly Bears", Zone::Battlefield);
        let victim = if victim == "Ethereal Armor" {
            let aura = put(&mut game, OTHER, victim, Zone::Battlefield);
            game.state.objects[aura as usize].attached_to = Some(game.object_ref(creature));
            aura
        } else {
            put(&mut game, OTHER, victim, Zone::Battlefield)
        };
        cast_target(&mut game, spell, victim);
        settle(&mut game);
        assert_eq!(game.object_view(victim).zone, zone, "{spell}");
        assert_eq!(game.state.players[PLAYER.index()].life, life, "{spell}");
    }
}

#[test]
fn pyroclasm_damages_both_sides_and_cackling_slasher_checks_an_actual_death() {
    let mut game = scenario(PLAYER);
    let our_creature = put(&mut game, PLAYER, "Grizzly Bears", Zone::Battlefield);
    let their_creature = put(&mut game, OTHER, "Bear Cub", Zone::Battlefield);
    let survivor = put(&mut game, OTHER, "Hill Giant", Zone::Battlefield);
    let slasher = put(&mut game, PLAYER, "Cackling Slasher", Zone::Hand);
    let spell = put(&mut game, PLAYER, "Pyroclasm", Zone::Hand);
    let mut early = game.clone();
    cast(&mut early, slasher, None);
    settle(&mut early);
    assert_eq!(early.counter_count(slasher, CounterKind::PlusOne), 0);

    cast(&mut game, spell, None);
    settle(&mut game);
    assert_eq!(game.object_view(our_creature).zone, Zone::Graveyard);
    assert_eq!(game.object_view(their_creature).zone, Zone::Graveyard);
    assert_eq!(game.object_view(survivor).damage, 2);
    assert_eq!(game.state.players[OTHER.index()].life, 20);

    cast(&mut game, slasher, None);
    settle(&mut game);
    assert_eq!(game.counter_count(slasher, CounterKind::PlusOne), 1);
    assert_eq!(stats(&game, slasher), (4, 4));
    assert!(game.keywords(slasher).contains(Keywords::DEATHTOUCH));
}

#[test]
fn commune_with_evil_takes_one_of_the_top_four_privately_and_bins_the_rest() {
    let mut game = scenario(PLAYER);
    let outside = put(&mut game, PLAYER, "Island", Zone::Library);
    let cards = ["Opt", "Mountain", "Grizzly Bears", "Shock"]
        .map(|name| put(&mut game, PLAYER, name, Zone::Library));
    let spell = put(&mut game, PLAYER, "Commune with Evil", Zone::Hand);
    cast(&mut game, spell, None);
    pass_pair(&mut game);
    let view = game.observe(PLAYER).library_selection.unwrap();
    assert_eq!(view.candidates.len(), 4);
    assert!(game.observe(OTHER).library_selection.is_none());
    reject(&mut game, Action::SelectLibrary { card: outside });
    reject(&mut game, Action::FinishLibrarySelection);
    act(&mut game, Action::SelectLibrary { card: cards[0] });
    settle(&mut game);
    assert_eq!(game.state.players[PLAYER.index()].life, 23);
    assert_eq!(game.object_view(cards[0]).zone, Zone::Hand);
    for &card in &cards[1..] {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }
    assert_eq!(game.object_view(outside).zone, Zone::Library);
    assert!(
        !game
            .observe(OTHER)
            .known_hand
            .contains(&game.object_ref(cards[0]))
    );
}

#[test]
fn live_or_die_has_two_different_legal_target_routes() {
    let mut game = scenario(PLAYER);
    let yours = put(&mut game, PLAYER, "Grizzly Bears", Zone::Graveyard);
    let theirs = put(&mut game, OTHER, "Hill Giant", Zone::Graveyard);
    let threat = put(&mut game, OTHER, "Hill Giant", Zone::Battlefield);
    let spell = put(&mut game, PLAYER, "Live or Die", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    let mut kill = game.clone();
    act(&mut game, Action::ChooseMode { mode: 0 });
    reject(&mut game, target(theirs));
    reject(&mut game, target(threat));
    act(&mut game, target(yours));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(yours).zone, Zone::Battlefield);
    act(&mut kill, Action::ChooseMode { mode: 1 });
    reject(&mut kill, target(theirs));
    act(&mut kill, target(threat));
    pay(&mut kill);
    settle(&mut kill);
    assert_eq!(kill.object_view(threat).zone, Zone::Graveyard);
}

#[test]
fn combat_instants_grant_their_printed_keywords_and_delirium_needs_four_types() {
    let mut game = scenario(PLAYER);
    let bear = put(&mut game, PLAYER, "Grizzly Bears", Zone::Battlefield);
    put(&mut game, PLAYER, "Friendly Teddy", Zone::Graveyard);
    put(&mut game, PLAYER, "Shock", Zone::Graveyard);
    let mut violence = game.clone();
    cast_target(&mut violence, "Give In to Violence", bear);
    settle(&mut violence);
    assert_eq!(stats(&violence, bear), (4, 4));
    assert!(violence.keywords(bear).contains(Keywords::LIFELINK));

    for delirium in [false, true] {
        let mut branch = game.clone();
        if delirium {
            put(&mut branch, PLAYER, "Forest", Zone::Graveyard);
        }
        cast_target(&mut branch, "Violent Urge", bear);
        settle(&mut branch);
        assert_eq!(stats(&branch, bear), (3, 2));
        assert!(branch.keywords(bear).contains(Keywords::FIRST_STRIKE));
        assert_eq!(
            branch.keywords(bear).contains(Keywords::DOUBLE_STRIKE),
            delirium
        );
    }
}

#[test]
fn both_typecycling_cards_pay_two_and_reveal_a_matching_nonbasic() {
    for (name, matching, keyword) in [
        ("Bedhead Beastie", "Commercial District", Keywords::MENACE),
        ("Spectral Snatcher", "Shadowy Backstreet", Keywords::NONE),
    ] {
        let mut game = scenario(PLAYER);
        let source = put(&mut game, PLAYER, name, Zone::Hand);
        let found = put(&mut game, PLAYER, matching, Zone::Library);
        let wrong = put(&mut game, PLAYER, "Island", Zone::Library);
        let mut creature = game.clone();
        cast(&mut creature, source, None);
        settle(&mut creature);
        assert!(creature.keywords(source).contains(keyword));
        assert_eq!(
            stats(&creature, source),
            if name == "Bedhead Beastie" {
                (5, 6)
            } else {
                (6, 5)
            }
        );

        game.state.players[PLAYER.index()].mana = [0, 0, 0, 0, 0, 2];
        act(
            &mut game,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut game);
        assert_eq!(game.object_view(source).zone, Zone::Graveyard);
        assert_eq!(game.state.players[PLAYER.index()].mana, [0; 6]);
        pass_pair(&mut game);
        reject(&mut game, Action::ChooseSearch { card: wrong });
        act(&mut game, Action::ChooseSearch { card: found });
        if game.legal_actions().contains(&Action::FinishSearch) {
            act(&mut game, Action::FinishSearch);
        }
        assert_eq!(game.object_view(found).zone, Zone::Hand);
        assert!(
            game.observe(OTHER)
                .known_hand
                .contains(&game.object_ref(found))
        );
    }
}

#[test]
fn spectral_snatchers_ward_requires_a_discard_from_its_opposing_targeter() {
    let mut game = scenario(PLAYER);
    let snatcher = put(&mut game, OTHER, "Spectral Snatcher", Zone::Battlefield);
    let discard = put(&mut game, PLAYER, "Forest", Zone::Hand);
    cast_target(&mut game, "Murder", snatcher);
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(PLAYER));
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineEffectPayment);
    settle(&mut decline);
    assert_eq!(decline.object_view(snatcher).zone, Zone::Battlefield);
    assert_eq!(decline.object_view(discard).zone, Zone::Hand);

    act(&mut game, Action::Discard { card: discard });
    settle(&mut game);
    assert_eq!(game.object_view(snatcher).zone, Zone::Graveyard);
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
}
