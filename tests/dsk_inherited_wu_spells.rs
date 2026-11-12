//! Actual spell and typecycling actions for inherited white and blue Duskmourn cards.
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

fn target(target: Target) -> Action {
    Action::ChooseTarget { target }
}

fn object(card: CardId) -> Action {
    target(Target::Card(card))
}

#[test]
fn dont_make_a_sound_surveils_only_if_the_target_spells_controller_pays() {
    let mut game = scenario(OTHER);
    let original = put(&mut game, OTHER, "Hill Giant", Zone::Hand);
    let counter = put(&mut game, PLAYER, "Don't Make a Sound", Zone::Hand);
    let cards = ["Forest", "Shock"].map(|name| put(&mut game, PLAYER, name, Zone::Library));
    cast(&mut game, original, None);
    act(&mut game, Action::Pass);
    cast(&mut game, counter, Some(Target::Spell(original)));
    game.state.players[OTHER.index()].mana = [0, 0, 0, 0, 0, 2];
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(OTHER));

    let mut unpaid = game.clone();
    act(&mut unpaid, Action::DeclineResolutionMana);
    settle(&mut unpaid);
    assert_eq!(unpaid.object_view(original).zone, Zone::Graveyard);
    assert!(
        cards
            .into_iter()
            .all(|card| unpaid.object_view(card).zone == Zone::Library)
    );

    act(
        &mut game,
        Action::PayResolutionMana {
            payment: [0, 0, 0, 0, 0, 2],
        },
    );
    assert_eq!(game.actor(), Some(PLAYER));
    assert!(game.observe(PLAYER).library_choice.unwrap().surveil);
    assert!(game.observe(OTHER).library_choice.is_none());
    for (card, destination) in [
        (cards[1], LibraryDestination::Graveyard),
        (cards[0], LibraryDestination::Top),
    ] {
        act(&mut game, Action::ArrangeLibrary { card, destination });
    }
    settle(&mut game);
    assert_eq!(game.object_view(original).zone, Zone::Battlefield);
    assert_eq!(game.object_view(cards[1]).zone, Zone::Graveyard);
}

#[test]
fn emerge_from_the_cocoon_returns_only_a_creature_from_your_graveyard_and_gains_life() {
    let mut game = scenario(PLAYER);
    let yours = put(&mut game, PLAYER, "Grizzly Bears", Zone::Graveyard);
    let theirs = put(&mut game, OTHER, "Hill Giant", Zone::Graveyard);
    let enchantment = put(&mut game, PLAYER, "Nowhere to Run", Zone::Graveyard);
    let spell = put(&mut game, PLAYER, "Emerge from the Cocoon", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    for invalid in [theirs, enchantment] {
        reject(&mut game, object(invalid));
    }
    act(&mut game, object(yours));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(yours).zone, Zone::Battlefield);
    assert_eq!(game.object_view(yours).controller, PLAYER);
    assert_eq!(game.state.players[PLAYER.index()].life, 23);
}

#[test]
fn enter_the_enigma_draws_and_keeps_the_exact_creature_unblockable_for_the_turn() {
    let mut game = scenario(PLAYER);
    let creature = put(&mut game, PLAYER, "Grizzly Bears", Zone::Battlefield);
    let blocker = put(&mut game, OTHER, "Bear Cub", Zone::Battlefield);
    let spell = put(&mut game, PLAYER, "Enter the Enigma", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(creature)));
    settle(&mut game);
    assert_eq!(game.state.players[PLAYER.index()].cards_drawn_this_turn, 1);
    assert!(game.keywords(creature).contains(Keywords::CANT_BE_BLOCKED));
    game.begin_step(Step::BeginCombat);
    pass_pair(&mut game);
    act(&mut game, Action::Attack { card: creature });
    act(&mut game, Action::FinishAttackers);
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::Block {
            blocker,
            attacker: creature,
        },
    );
}

#[test]
fn ethereal_armor_counts_live_controlled_enchantments_including_itself() {
    let mut game = scenario(PLAYER);
    let creature = put(&mut game, PLAYER, "Grizzly Bears", Zone::Battlefield);
    let own = put(&mut game, PLAYER, "Nowhere to Run", Zone::Battlefield);
    put(&mut game, OTHER, "Trapped in the Screen", Zone::Battlefield);
    let aura = put(&mut game, PLAYER, "Ethereal Armor", Zone::Hand);
    cast(&mut game, aura, Some(Target::Card(creature)));
    settle(&mut game);
    assert_eq!(
        game.object_view(aura).attached_to,
        Some(game.object_ref(creature))
    );
    assert_eq!(stats(&game, creature), (4, 4));
    assert!(game.keywords(creature).contains(Keywords::FIRST_STRIKE));
    game.change_controller(game.object_ref(own), OTHER);
    assert_eq!(stats(&game, creature), (3, 3));
}

#[test]
fn exorcise_accepts_all_three_target_categories_and_checks_live_creature_power() {
    let mut game = scenario(PLAYER);
    let artifact = put(&mut game, OTHER, "Bear Trap", Zone::Battlefield);
    let enchantment = put(&mut game, PLAYER, "Nowhere to Run", Zone::Battlefield);
    let four = put(&mut game, OTHER, "Cautious Survivor", Zone::Battlefield);
    let three = put(&mut game, OTHER, "Hill Giant", Zone::Battlefield);
    let land = put(&mut game, OTHER, "Forest", Zone::Battlefield);
    let spell = put(&mut game, PLAYER, "Exorcise", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    for invalid in [three, land] {
        reject(&mut game, object(invalid));
    }
    for card in [artifact, enchantment, four] {
        let mut branch = game.clone();
        act(&mut branch, object(card));
        pay(&mut branch);
        settle(&mut branch);
        assert_eq!(branch.object_view(card).zone, Zone::Exile);
    }
}

#[test]
fn get_out_counters_the_allowed_spell_and_bounces_permanents_by_ownership() {
    let mut counter = scenario(OTHER);
    let creature = put(&mut counter, OTHER, "Grizzly Bears", Zone::Hand);
    let get_out = put(&mut counter, PLAYER, "Get Out", Zone::Hand);
    cast(&mut counter, creature, None);
    act(&mut counter, Action::Pass);
    act(&mut counter, Action::AnnounceCast { card: get_out });
    act(&mut counter, Action::ChooseMode { mode: 0 });
    act(&mut counter, target(Target::Spell(creature)));
    pay(&mut counter);
    settle(&mut counter);
    assert_eq!(counter.object_view(creature).zone, Zone::Graveyard);

    let mut game = scenario(PLAYER);
    let yours = put(&mut game, PLAYER, "Grizzly Bears", Zone::Battlefield);
    let theirs = put(&mut game, OTHER, "Bear Cub", Zone::Battlefield);
    let enchantment = put(&mut game, PLAYER, "Nowhere to Run", Zone::Battlefield);
    game.change_controller(game.object_ref(yours), OTHER);
    game.change_controller(game.object_ref(theirs), PLAYER);
    let get_out = put(&mut game, PLAYER, "Get Out", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: get_out });
    act(&mut game, Action::ChooseMode { mode: 1 });
    reject(&mut game, object(theirs));
    act(&mut game, object(yours));
    let mut single = game.clone();
    act(&mut single, Action::FinishTargets);
    pay(&mut single);
    settle(&mut single);
    assert_eq!(single.object_view(yours).zone, Zone::Hand);
    assert_eq!(single.object_view(enchantment).zone, Zone::Battlefield);
    act(&mut game, object(enchantment));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(yours).zone, Zone::Hand);
    assert_eq!(game.object_view(enchantment).zone, Zone::Hand);
    assert_eq!(game.object_view(theirs).zone, Zone::Battlefield);
}

#[test]
fn seized_from_slumber_reduces_only_the_tapped_creature_payment() {
    let mut game = scenario(PLAYER);
    let tapped = put(&mut game, OTHER, "Hill Giant", Zone::Battlefield);
    game.state.objects[tapped as usize].tapped = true;
    let ready = put(&mut game, OTHER, "Grizzly Bears", Zone::Battlefield);
    let spell = put(&mut game, PLAYER, "Seized from Slumber", Zone::Hand);
    game.state.players[PLAYER.index()].mana = [1, 0, 0, 0, 0, 1];
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, object(ready));
    act(&mut game, object(tapped));
    act(
        &mut game,
        Action::PayCosts {
            payment: [1, 0, 0, 0, 0, 1],
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(tapped).zone, Zone::Graveyard);
    assert_eq!(game.object_view(ready).zone, Zone::Battlefield);
}

#[test]
fn split_up_evaluates_each_creatures_tapped_status_for_both_modes() {
    let mut game = scenario(PLAYER);
    let tapped = put(&mut game, PLAYER, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[tapped as usize].tapped = true;
    let ready = put(&mut game, OTHER, "Hill Giant", Zone::Battlefield);
    let land = put(&mut game, PLAYER, "Forest", Zone::Battlefield);
    game.state.objects[land as usize].tapped = true;
    let spell = put(&mut game, PLAYER, "Split Up", Zone::Hand);
    for (mode, dead, live) in [(0, tapped, ready), (1, ready, tapped)] {
        let mut branch = game.clone();
        act(&mut branch, Action::AnnounceCast { card: spell });
        act(&mut branch, Action::ChooseMode { mode });
        pay(&mut branch);
        settle(&mut branch);
        assert_eq!(branch.object_view(dead).zone, Zone::Graveyard);
        assert_eq!(branch.object_view(live).zone, Zone::Battlefield);
        assert_eq!(branch.object_view(land).zone, Zone::Battlefield);
    }
}

#[test]
fn sheltered_by_ghosts_and_trapped_in_the_screen_have_exact_linked_returns() {
    for name in ["Sheltered by Ghosts", "Trapped in the Screen"] {
        let mut game = scenario(PLAYER);
        let bearer = put(&mut game, PLAYER, "Grizzly Bears", Zone::Battlefield);
        let enemy = put(&mut game, OTHER, "Bear Trap", Zone::Battlefield);
        let enemy_land = put(&mut game, OTHER, "Forest", Zone::Battlefield);
        let enemy_creature = put(&mut game, OTHER, "Hill Giant", Zone::Battlefield);
        let exact = game.object_ref(enemy);
        let spell = put(&mut game, PLAYER, name, Zone::Hand);
        let is_aura = name == "Sheltered by Ghosts";
        if is_aura {
            act(&mut game, Action::AnnounceCast { card: spell });
            reject(&mut game, object(enemy_creature));
            act(&mut game, object(bearer));
            pay(&mut game);
        } else {
            cast(&mut game, spell, None);
        }
        pass_pair(&mut game);
        reject(&mut game, object(enemy_land));
        act(&mut game, object(enemy));
        settle(&mut game);
        assert_eq!(game.object_view(enemy).zone, Zone::Exile);
        if is_aura {
            assert_eq!(stats(&game, bearer), (3, 2));
            assert!(game.keywords(bearer).contains(Keywords::LIFELINK));
        }

        let removal = put(
            &mut game,
            OTHER,
            if is_aura { "Murder" } else { "Disenchant" },
            Zone::Hand,
        );
        act(&mut game, Action::Pass);
        cast(
            &mut game,
            removal,
            Some(Target::Card(if is_aura { bearer } else { spell })),
        );
        pass_pair(&mut game);
        let mut unpaid = game.clone();
        act(&mut unpaid, Action::DeclineResolutionMana);
        settle(&mut unpaid);
        assert_eq!(unpaid.object_view(enemy).zone, Zone::Exile);
        choose(&mut game, |action| {
            matches!(action, Action::PayResolutionMana { .. })
        });
        settle(&mut game);
        let returned = game.object_view(enemy);
        assert_eq!(returned.zone, Zone::Battlefield);
        assert_ne!(returned.reference, exact);
        assert_eq!(returned.controller, OTHER);
    }
}

#[test]
fn vanish_from_sight_lets_the_owner_choose_both_library_ends_then_surveils_for_the_caster() {
    let mut game = scenario(PLAYER);
    let creature = put(&mut game, OTHER, "Hill Giant", Zone::Battlefield);
    game.change_controller(game.object_ref(creature), PLAYER);
    let top = put(&mut game, PLAYER, "Shock", Zone::Library);
    let spell = put(&mut game, PLAYER, "Vanish from Sight", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(creature)));
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(OTHER));
    let exact = game.object_ref(creature);
    for destination in [LibraryDestination::Top, LibraryDestination::Bottom] {
        let mut branch = game.clone();
        act(
            &mut branch,
            Action::ChooseLibraryDestination {
                object: exact,
                destination,
            },
        );
        let library = &branch.state.players[OTHER.index()].library;
        assert_eq!(
            if destination == LibraryDestination::Top {
                library.last()
            } else {
                library.first()
            },
            Some(&creature),
        );
        assert_eq!(branch.actor(), Some(PLAYER));
        assert!(branch.observe(OTHER).library_choice.is_none());
        act(
            &mut branch,
            Action::ArrangeLibrary {
                card: top,
                destination: LibraryDestination::Graveyard,
            },
        );
        assert_eq!(branch.object_view(top).zone, Zone::Graveyard);
    }
}

#[test]
fn the_white_and_blue_typecyclers_pay_two_and_find_typed_nonbasics() {
    for (name, land, keyword) in [
        (
            "Daggermaw Megalodon",
            "Meticulous Archive",
            Keywords::VIGILANCE,
        ),
        (
            "Shepherding Spirits",
            "Shadowy Backstreet",
            Keywords::FLYING,
        ),
    ] {
        let mut game = scenario(PLAYER);
        let creature = put(&mut game, PLAYER, name, Zone::Hand);
        let selected = put(&mut game, PLAYER, land, Zone::Library);
        let invalid = put(&mut game, PLAYER, "Mountain", Zone::Library);
        let mut normal = game.clone();
        cast(&mut normal, creature, None);
        settle(&mut normal);
        assert!(normal.keywords(creature).contains(keyword));
        game.state.players[PLAYER.index()].mana = [0, 0, 0, 0, 0, 2];
        act(
            &mut game,
            Action::AnnounceActivate {
                card: creature,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut game);
        assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
        assert_eq!(game.state.players[PLAYER.index()].mana, [0; 6]);
        pass_pair(&mut game);
        reject(&mut game, Action::ChooseSearch { card: invalid });
        act(&mut game, Action::ChooseSearch { card: selected });
        if game.legal_actions().contains(&Action::FinishSearch) {
            act(&mut game, Action::FinishSearch);
        }
        assert_eq!(game.object_view(selected).zone, Zone::Hand);
        assert!(
            game.observe(OTHER)
                .known_hand
                .contains(&game.object_ref(selected))
        );
    }
}
