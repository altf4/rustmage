//! Paid outcome proofs for inherited S-name cards in Wilds of Eldraine.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const FOOD: Card = Card::from_catalogue_id(5246);
const RAT: Card = Card::from_catalogue_id(5323);

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn adventure(game: &mut Game, card: CardId, recipient: Option<CardId>) {
    act(game, Action::AnnounceAdventure { card });
    if let Some(recipient) = recipient {
        act(game, target(recipient));
    }
    pay(game);
}

fn from_adventure(game: &mut Game, card: CardId) {
    act(game, Action::AnnounceExile { card });
    pay(game);
}

fn tokens(game: &Game, player: Player, card: Card) -> Vec<ObjectView> {
    game.observe(player)
        .objects
        .into_iter()
        .filter(|object| {
            object.zone == Zone::Battlefield
                && object.controller == player
                && object.card == card
                && object.is_token()
        })
        .collect()
}

fn finish_turn(game: &mut Game) {
    let turn = game.state.turn;
    for _ in 0..32 {
        if game.state.turn != turn {
            return;
        }
        let action = match game.state.pending {
            Pending::Attackers => Action::FinishAttackers,
            Pending::Blockers => Action::FinishBlockers,
            Pending::Priority => Action::Pass,
            ref pending => panic!("unexpected turn choice {pending:?}"),
        };
        act(game, action);
    }
    panic!("turn did not finish");
}

fn reach_attackers(game: &mut Game) {
    for _ in 0..16 {
        if game.state.pending == Pending::Attackers {
            return;
        }
        act(game, Action::Pass);
    }
    panic!("attackers did not begin");
}

fn order_triggers(game: &mut Game) {
    for _ in 0..8 {
        if !matches!(game.state.pending, Pending::OrderTriggers { .. }) {
            return;
        }
        choose(game, |action| matches!(action, Action::PutTrigger { .. }));
    }
    panic!("trigger order did not finish");
}

#[test]
fn steam_clean_returns_a_nonland_to_its_owner_and_viper_punishes_cheap_opposing_casts() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let viper = put(
        &mut game,
        player,
        "Scalding Viper // Steam Clean",
        Zone::Hand,
    );
    let vault = put(&mut game, opponent, "Collector's Vault", Zone::Battlefield);
    let land = put(&mut game, opponent, "Forest", Zone::Battlefield);
    let own_spell = put(&mut game, player, "Quick Study", Zone::Hand);
    let opposing_spell = put(&mut game, opponent, "Quick Study", Zone::Hand);
    let expensive = put(&mut game, opponent, "Rowdy Research", Zone::Hand);
    game.change_controller(game.object_ref(vault), player);
    act(&mut game, Action::AnnounceAdventure { card: viper });
    reject(&mut game, target(land));
    act(&mut game, target(vault));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(vault).zone, Zone::Hand);
    assert_eq!(game.object_view(vault).owner, opponent);
    assert_eq!(game.object_view(viper).zone, Zone::Exile);

    from_adventure(&mut game, viper);
    settle(&mut game);
    assert_eq!(stats(&game, viper), (2, 1));
    cast(&mut game, own_spell, None);
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 20);

    act(&mut game, Action::Pass);
    cast(&mut game, opposing_spell, None);
    assert_eq!(game.state.stack.len(), 2);
    pass_pair(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 19);
    assert_eq!(game.object_view(opposing_spell).zone, Zone::Stack);
    settle(&mut game);

    act(&mut game, Action::Pass);
    cast(&mut game, expensive, None);
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 19);
}

#[test]
fn cleave_shadows_affects_only_enemy_creatures_and_shepherd_may_boost_itself() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let shepherd = put(
        &mut game,
        player,
        "Shrouded Shepherd // Cleave Shadows",
        Zone::Hand,
    );
    let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let small = put(&mut game, opponent, "Savannah Lions", Zone::Battlefield);
    let large = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    adventure(&mut game, shepherd, None);
    settle(&mut game);
    assert_eq!(game.object_view(shepherd).zone, Zone::Exile);
    assert_eq!(game.object_view(small).zone, Zone::Graveyard);
    assert_eq!(stats(&game, large), (2, 2));
    assert_eq!(stats(&game, own), (2, 2));

    from_adventure(&mut game, shepherd);
    pass_pair(&mut game);
    reject(&mut game, target(large));
    act(&mut game, target(shepherd));
    settle(&mut game);
    assert_eq!(stats(&game, shepherd), (4, 4));
    assert_eq!(stats(&game, own), (2, 2));
    finish_turn(&mut game);
    assert_eq!(stats(&game, shepherd), (2, 2));
    assert_eq!(stats(&game, large), (3, 3));
}

#[test]
fn bear_down_reaches_both_permanent_types_and_song_enables_vanguards_restricted_attack() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let vanguard = put(
        &mut game,
        player,
        "Stormkeld Vanguard // Bear Down",
        Zone::Hand,
    );
    let song = put(&mut game, player, "Song of Totentanz", Zone::Hand);
    let artifact = put(&mut game, opponent, "Collector's Vault", Zone::Battlefield);
    let enchantment = put(&mut game, opponent, "Virtue of Strength", Zone::Battlefield);
    let small = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let large = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    act(&mut game, Action::AnnounceAdventure { card: vanguard });
    reject(&mut game, target(small));
    let mut remove_enchantment = game.clone();
    act(&mut remove_enchantment, target(enchantment));
    pay(&mut remove_enchantment);
    settle(&mut remove_enchantment);
    assert_eq!(
        remove_enchantment.object_view(enchantment).zone,
        Zone::Graveyard
    );
    assert_eq!(
        remove_enchantment.object_view(artifact).zone,
        Zone::Battlefield
    );
    act(&mut game, target(artifact));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);
    assert_eq!(game.object_view(vanguard).zone, Zone::Exile);
    from_adventure(&mut game, vanguard);
    settle(&mut game);
    assert_eq!(stats(&game, vanguard), (6, 7));
    let mut no_haste = game.clone();
    reach_attackers(&mut no_haste);
    reject(&mut no_haste, Action::Attack { card: vanguard });

    act(&mut game, Action::AnnounceCast { card: song });
    let mut zero = game.clone();
    act(&mut zero, Action::ChooseX { value: 0 });
    pay(&mut zero);
    settle(&mut zero);
    assert!(tokens(&zero, player, RAT).is_empty());
    assert!(
        zero.object_view(vanguard)
            .keywords
            .contains(Keywords::HASTE)
    );

    act(&mut game, Action::ChooseX { value: 2 });
    pay(&mut game);
    settle(&mut game);
    let rats = tokens(&game, player, RAT);
    assert_eq!(rats.len(), 2);
    for rat in &rats {
        assert_eq!((rat.power, rat.toughness, rat.colors), (1, 1, 4));
        assert!(rat.has_subtype("Rat"));
        assert!(
            rat.keywords
                .contains(Keywords::CANT_BLOCK.union(Keywords::HASTE))
        );
    }
    assert!(
        game.object_view(vanguard)
            .keywords
            .contains(Keywords::HASTE)
    );
    assert!(!game.object_view(small).keywords.contains(Keywords::HASTE));

    reach_attackers(&mut game);
    act(&mut game, Action::Attack { card: vanguard });
    act(&mut game, Action::Attack { card: rats[0].id });
    act(&mut game, Action::FinishAttackers);
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::Block {
            blocker: small,
            attacker: vanguard,
        },
    );
    act(
        &mut game,
        Action::Block {
            blocker: large,
            attacker: vanguard,
        },
    );
    finish_turn(&mut game);
    assert!(
        !game
            .object_view(vanguard)
            .keywords
            .contains(Keywords::HASTE)
    );
    assert!(
        !game
            .object_view(rats[0].id)
            .keywords
            .contains(Keywords::HASTE)
    );
}

#[test]
fn skybeast_tracker_and_stormkeld_prowler_use_paid_spell_mana_value_including_x() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let tracker = put(&mut game, player, "Skybeast Tracker", Zone::Hand);
    let prowler = put(&mut game, player, "Stormkeld Prowler", Zone::Hand);
    let goose = put(&mut game, player, "The Goose Mother", Zone::Hand);
    let foreign = put(&mut game, opponent, "Rowdy Research", Zone::Hand);
    for card in [tracker, prowler] {
        cast(&mut game, card, None);
        settle(&mut game);
    }
    assert_eq!(stats(&game, tracker), (2, 4));
    assert!(game.object_view(tracker).keywords.contains(Keywords::REACH));
    assert_eq!(stats(&game, prowler), (2, 1));

    let mut other_cast = game.clone();
    act(&mut other_cast, Action::Pass);
    cast(&mut other_cast, foreign, None);
    settle(&mut other_cast);
    assert_eq!(other_cast.counter_count(prowler, CounterKind::PlusOne), 0);
    assert!(tokens(&other_cast, player, FOOD).is_empty());

    for (x, counters, food) in [(2, 0, 0), (3, 2, 1)] {
        let mut branch = game.clone();
        act(&mut branch, Action::AnnounceCast { card: goose });
        act(&mut branch, Action::ChooseX { value: x });
        act(
            &mut branch,
            Action::PayCosts {
                payment: [0, 1, 0, 0, 1, x],
            },
        );
        order_triggers(&mut branch);
        if x == 3 {
            assert_eq!(branch.state.stack.len(), 3);
            pass_pair(&mut branch);
            pass_pair(&mut branch);
        }
        assert_eq!(branch.object_view(goose).zone, Zone::Stack);
        assert_eq!(
            branch.counter_count(prowler, CounterKind::PlusOne),
            counters
        );
        assert_eq!(tokens(&branch, player, FOOD).len(), food);
        settle(&mut branch);
        assert_eq!(
            tokens(&branch, player, FOOD).len(),
            if x == 2 { 1 } else { 3 }
        );
    }
}

const CURSED_ROLE: Card = Card::from_catalogue_id(5263);
const WICKED_ROLE: Card = Card::from_catalogue_id(5248);
const SORCERER_ROLE: Card = Card::from_catalogue_id(5321);
const MONSTER_ROLE: Card = Card::from_catalogue_id(5330);

fn has_role(game: &Game, player: Player, role: Card, bearer: CardId) -> bool {
    tokens(game, player, role)
        .iter()
        .any(|object| object.attached_to == Some(game.object_ref(bearer)))
}

#[test]
fn hexmage_and_shatter_replace_roles_and_savior_counts_only_controlled_enchantment_deaths() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let savior = put(&mut game, player, "Savior of the Sleeping", Zone::Hand);
    let hexmage = put(&mut game, player, "Spiteful Hexmage", Zone::Hand);
    let shatter = put(&mut game, player, "Shatter the Oath", Zone::Hand);
    let disenchant = put(&mut game, player, "Disenchant", Zone::Hand);
    let bounce = put(&mut game, opponent, "Unsummon", Zone::Hand);
    let enemy = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let artifact = put(&mut game, opponent, "Collector's Vault", Zone::Battlefield);
    let enchantment = put(&mut game, opponent, "Virtue of Strength", Zone::Battlefield);
    cast(&mut game, savior, None);
    settle(&mut game);
    assert_eq!(stats(&game, savior), (2, 3));
    assert!(
        game.object_view(savior)
            .keywords
            .contains(Keywords::VIGILANCE)
    );
    cast(&mut game, hexmage, None);
    pass_pair(&mut game);
    reject(&mut game, target(enemy));
    act(&mut game, target(hexmage));
    settle(&mut game);
    assert_eq!(stats(&game, hexmage), (1, 1));
    assert!(has_role(&game, player, CURSED_ROLE, hexmage));
    let cursed = tokens(&game, player, CURSED_ROLE)[0].id;

    act(&mut game, Action::AnnounceCast { card: shatter });
    reject(&mut game, target(artifact));
    let mut destroy_only = game.clone();
    act(&mut destroy_only, target(enchantment));
    act(&mut destroy_only, Action::FinishTargets);
    pay(&mut destroy_only);
    settle(&mut destroy_only);
    assert_eq!(destroy_only.object_view(enchantment).zone, Zone::Graveyard);
    assert!(tokens(&destroy_only, player, WICKED_ROLE).is_empty());
    assert_eq!(destroy_only.counter_count(savior, CounterKind::PlusOne), 0);

    act(&mut game, target(enemy));
    reject(&mut game, target(enemy));
    act(&mut game, target(hexmage));
    pay(&mut game);
    let mut survivor = game.clone();
    act(&mut survivor, Action::Pass);
    cast(&mut survivor, bounce, Some(Target::Card(enemy)));
    settle(&mut survivor);
    assert_eq!(survivor.object_view(enemy).zone, Zone::Hand);
    assert!(has_role(&survivor, player, WICKED_ROLE, hexmage));
    assert_eq!(survivor.counter_count(savior, CounterKind::PlusOne), 1);

    settle(&mut game);
    assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
    assert_ne!(game.object_view(cursed).zone, Zone::Battlefield);
    assert!(has_role(&game, player, WICKED_ROLE, hexmage));
    assert_eq!(stats(&game, hexmage), (4, 3));
    assert_eq!(game.counter_count(savior, CounterKind::PlusOne), 1);
    let wicked = tokens(&game, player, WICKED_ROLE)[0].id;
    cast(&mut game, disenchant, Some(Target::Card(wicked)));
    pass_pair(&mut game);
    order_triggers(&mut game);
    settle(&mut game);
    assert_eq!(game.counter_count(savior, CounterKind::PlusOne), 2);
    assert_eq!(game.state.players[opponent.index()].life, 19);
}

#[test]
fn spellbook_vendor_pays_before_targeting_and_the_role_scries_on_attack() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let vendor = put(&mut game, player, "Spellbook Vendor", Zone::Hand);
    let bearer = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let foreign = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let land = put(&mut game, player, "Island", Zone::Battlefield);
    let top = put(&mut game, player, "Opt", Zone::Library);
    cast(&mut game, vendor, None);
    settle(&mut game);
    assert!(
        game.object_view(vendor)
            .keywords
            .contains(Keywords::VIGILANCE)
    );

    pass_pair(&mut game);
    assert_eq!(game.state.step, Step::BeginCombat);
    pass_pair(&mut game);
    let offer = game.observe(player).resolution_payment.unwrap();
    assert_eq!(offer.cost.generic, 1);
    reject(&mut game, target(bearer));
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineResolutionMana);
    settle(&mut declined);
    assert!(tokens(&declined, player, SORCERER_ROLE).is_empty());

    act(&mut game, Action::TapMana { card: land });
    act(
        &mut game,
        Action::PayResolutionMana {
            payment: [0, 1, 0, 0, 0, 0],
        },
    );
    reject(&mut game, target(foreign));
    act(&mut game, target(bearer));
    settle(&mut game);
    assert!(has_role(&game, player, SORCERER_ROLE, bearer));
    assert_eq!(stats(&game, bearer), (3, 3));

    reach_attackers(&mut game);
    act(&mut game, Action::Attack { card: bearer });
    act(&mut game, Action::FinishAttackers);
    pass_pair(&mut game);
    let look = game.observe(player).library_choice.unwrap();
    assert_eq!(look.remaining.len(), 1);
    assert!(game.observe(opponent).library_choice.is_none());
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: top,
            destination: LibraryDestination::Bottom,
        },
    );
    settle(&mut game);
    assert_eq!(
        game.state.players[player.index()].library.first(),
        Some(&top)
    );
    finish_turn(&mut game);
    reach_attackers(&mut game);
    assert_eq!(game.state.active, opponent);
    assert_eq!(tokens(&game, player, SORCERER_ROLE).len(), 1);
}

#[test]
fn splashy_spellcaster_roles_choose_up_to_one_other_creature_on_a_qualifying_cast() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let caster = put(&mut game, player, "Splashy Spellcaster", Zone::Hand);
    let ally = put(&mut game, player, "Bear Cub", Zone::Hand);
    let enemy = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let foreign = put(&mut game, opponent, "Quick Study", Zone::Hand);
    let viper = put(&mut game, player, "Scalding Viper", Zone::Hand);
    let vault = put(&mut game, opponent, "Collector's Vault", Zone::Battlefield);
    for card in [caster, ally] {
        cast(&mut game, card, None);
        settle(&mut game);
    }
    assert!(tokens(&game, player, SORCERER_ROLE).is_empty());
    act(&mut game, Action::Pass);
    cast(&mut game, foreign, None);
    settle(&mut game);
    assert!(tokens(&game, player, SORCERER_ROLE).is_empty());

    adventure(&mut game, viper, Some(vault));
    reject(&mut game, target(caster));
    reject(&mut game, target(enemy));
    let mut decline = game.clone();
    act(&mut decline, Action::FinishTargets);
    settle(&mut decline);
    assert!(tokens(&decline, player, SORCERER_ROLE).is_empty());
    assert_eq!(decline.object_view(viper).zone, Zone::Exile);
    act(&mut game, target(ally));
    settle(&mut game);
    assert!(has_role(&game, player, SORCERER_ROLE, ally));
    assert_eq!(stats(&game, ally), (3, 3));
    assert_eq!(game.object_view(viper).zone, Zone::Exile);
}

#[test]
fn syr_armont_roles_another_creature_and_counts_each_enchanted_ally_once() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let armont = put(&mut game, player, "Syr Armont, the Redeemer", Zone::Hand);
    let bearer = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let plain = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let own_aura = put(&mut game, opponent, "Bestial Bloodline", Zone::Hand);
    let foreign_aura = put(&mut game, opponent, "Bestial Bloodline", Zone::Hand);
    let forests = [(); 4].map(|_| put(&mut game, player, "Forest", Zone::Battlefield));
    let plains = put(&mut game, player, "Plains", Zone::Battlefield);
    cast(&mut game, own_aura, Some(Target::Card(bearer)));
    settle(&mut game);
    cast(&mut game, foreign_aura, Some(Target::Card(enemy)));
    settle(&mut game);
    assert_eq!(stats(&game, bearer), (4, 4));
    finish_turn(&mut game);
    for _ in 0..4 {
        if game.state.step == Step::Main1 {
            break;
        }
        act(&mut game, Action::Pass);
    }
    assert_eq!((game.state.active, game.state.step), (player, Step::Main1));
    for card in forests.into_iter().chain([plains]) {
        act(&mut game, Action::TapMana { card });
    }
    cast(&mut game, armont, None);
    pass_pair(&mut game);
    assert_eq!(stats(&game, bearer), (5, 5));
    reject(&mut game, target(armont));
    reject(&mut game, target(enemy));
    act(&mut game, target(bearer));
    settle(&mut game);
    assert!(has_role(&game, player, MONSTER_ROLE, bearer));
    assert_eq!(stats(&game, bearer), (6, 6));
    assert!(
        game.object_view(bearer)
            .keywords
            .contains(Keywords::TRAMPLE)
    );
    assert_eq!(stats(&game, plain), (2, 2));
    assert_eq!(stats(&game, enemy), (4, 4));
    assert_eq!(game.object_view(own_aura).controller, opponent);
}

fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

#[test]
fn scarecrow_guide_pays_generic_for_each_color_once_per_turn_without_tapping() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let guide = put(&mut game, player, "Scarecrow Guide", Zone::Hand);
    let forest = put(&mut game, player, "Forest", Zone::Battlefield);
    let second = put(&mut game, player, "Forest", Zone::Battlefield);
    cast(&mut game, guide, None);
    settle(&mut game);
    let view = game.object_view(guide);
    assert!(view.keywords.contains(Keywords::REACH));
    assert_eq!((view.power, view.toughness), (2, 1));
    assert!(view.summoning_sick);
    game.state.players[player.index()].mana = [0; 6];

    let green = [0, 0, 0, 0, 1, 0];
    let blue = [0, 1, 0, 0, 0, 0];
    let produce = |production| Action::ActivateMana {
        card: guide,
        ability: 0,
        counter_payment: Counters::default(),
        production,
        payment: green,
    };
    reject(&mut game, produce(blue));
    act(&mut game, Action::TapMana { card: forest });
    reject(&mut game, produce([0, 0, 0, 0, 0, 1]));
    for index in 0..5 {
        let mut branch = game.clone();
        let mut production = [0; 6];
        production[index] = 1;
        act(&mut branch, produce(production));
        assert_eq!(branch.state.players[player.index()].mana, production);
        assert!(!branch.object_view(guide).tapped);
    }
    act(&mut game, produce(green));
    reject(&mut game, produce(blue));
    finish_turn(&mut game);
    assert_eq!(game.state.active, opponent);
    if game.actor() != Some(player) {
        act(&mut game, Action::Pass);
    }
    act(&mut game, Action::TapMana { card: second });
    act(&mut game, produce(blue));
    assert_eq!(game.state.players[player.index()].mana, blue);
}

#[test]
fn sleep_cursed_faerie_enters_with_three_stuns_and_requires_each_untap_and_ward_payment() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let faerie = put(&mut game, player, "Sleep-Cursed Faerie", Zone::Hand);
    let bounce = put(&mut game, opponent, "Unsummon", Zone::Hand);
    cast(&mut game, faerie, None);
    settle(&mut game);
    assert_eq!(stats(&game, faerie), (3, 3));
    assert!(game.object_view(faerie).keywords.contains(Keywords::FLYING));
    assert!(game.object_view(faerie).tapped);
    assert_eq!(game.counter_count(faerie, CounterKind::Stun), 3);
    let mut natural = game.clone();
    finish_turn(&mut natural);
    finish_turn(&mut natural);
    assert!(natural.object_view(faerie).tapped);
    assert_eq!(natural.counter_count(faerie, CounterKind::Stun), 2);

    for (stun, tapped) in [(2, true), (1, true), (0, true), (0, false)] {
        act(&mut game, activate(faerie, 0));
        let domain = game.mana_payment_domain().unwrap();
        assert_eq!(domain.required, [0, 1, 0, 0, 0, 0]);
        assert_eq!(domain.generic, 1);
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.counter_count(faerie, CounterKind::Stun), stun);
        assert_eq!(game.object_view(faerie).tapped, tapped);
    }

    act(&mut game, Action::Pass);
    cast(&mut game, bounce, Some(Target::Card(faerie)));
    pass_pair(&mut game);
    assert_eq!(
        game.observe(opponent)
            .resolution_payment
            .unwrap()
            .cost
            .generic,
        2
    );
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineResolutionMana);
    settle(&mut declined);
    assert_eq!(declined.object_view(faerie).zone, Zone::Battlefield);
    assert_eq!(declined.object_view(bounce).zone, Zone::Graveyard);
    act(
        &mut game,
        Action::PayResolutionMana {
            payment: [0, 0, 0, 0, 0, 2],
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(faerie).zone, Zone::Hand);
}

#[test]
fn sleight_of_hand_keeps_one_of_the_top_two_private_and_bottoms_the_other() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Sleight of Hand", Zone::Hand);
    let below = put(&mut game, player, "Forest", Zone::Library);
    let second = put(&mut game, player, "Bear Cub", Zone::Library);
    let top = put(&mut game, player, "Opt", Zone::Library);
    let mut short = game.clone();
    let extras = short.state.players[player.index()]
        .library
        .iter()
        .copied()
        .filter(|card| *card != top)
        .collect::<Vec<_>>();
    for card in extras {
        short.move_object(card, Zone::Exile);
    }
    cast(&mut short, spell, None);
    pass_pair(&mut short);
    if short.observe(player).library_selection.is_some() {
        act(&mut short, Action::SelectLibrary { card: top });
        act(&mut short, Action::FinishLibrarySelection);
    }
    settle(&mut short);
    assert_eq!(short.object_view(top).zone, Zone::Hand);
    assert!(short.state.players[player.index()].library.is_empty());

    cast(&mut game, spell, None);
    pass_pair(&mut game);
    let view = game.observe(player).library_selection.unwrap();
    assert_eq!(
        view.remaining
            .iter()
            .map(|object| object.id)
            .collect::<Vec<_>>(),
        [top, second]
    );
    assert_eq!((view.minimum, view.maximum), (1, 1));
    assert!(game.observe(opponent).library_selection.is_none());
    reject(&mut game, Action::FinishLibrarySelection);
    reject(&mut game, Action::SelectLibrary { card: below });

    let mut other = game.clone();
    for (branch, selected, bottom) in [(&mut game, top, second), (&mut other, second, top)] {
        act(branch, Action::SelectLibrary { card: selected });
        act(branch, Action::FinishLibrarySelection);
        act(branch, Action::OrderLibraryBottom { card: bottom });
        settle(branch);
        assert_eq!(branch.object_view(selected).zone, Zone::Hand);
        assert_eq!(
            branch.state.players[player.index()].library.first(),
            Some(&bottom)
        );
        assert_eq!(branch.top_library_card(player), Some(below));
        assert!(branch.observe(opponent).zone_reveals.is_empty());
    }
    assert_eq!(
        serde_json::to_value(game.observe(opponent)).unwrap(),
        serde_json::to_value(other.observe(opponent)).unwrap()
    );
}

#[test]
fn soul_guide_lantern_exiles_the_chosen_grave_card_then_pays_for_either_activation() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let lantern = put(&mut game, player, "Soul-Guide Lantern", Zone::Hand);
    let own = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    let own_keep = put(&mut game, player, "Forest", Zone::Graveyard);
    let foreign = put(&mut game, opponent, "Murder", Zone::Graveyard);
    let foreign_land = put(&mut game, opponent, "Forest", Zone::Graveyard);
    let invalid = put(&mut game, opponent, "Murder", Zone::Hand);
    let top = put(&mut game, player, "Opt", Zone::Library);
    cast(&mut game, lantern, None);
    pass_pair(&mut game);
    reject(&mut game, target(invalid));
    act(&mut game, target(own));
    settle(&mut game);
    assert_eq!(game.object_view(own).zone, Zone::Exile);
    assert_eq!(game.object_view(foreign).zone, Zone::Graveyard);

    let mut exile = game.clone();
    act(
        &mut exile,
        Action::Activate {
            card: lantern,
            ability: 0,
            target: None,
            payment: [0; 6],
            counter_payment: Counters::default(),
        },
    );
    assert_eq!(exile.object_view(lantern).zone, Zone::Graveyard);
    settle(&mut exile);
    assert_eq!(exile.object_view(foreign).zone, Zone::Exile);
    assert_eq!(exile.object_view(foreign_land).zone, Zone::Exile);
    assert_eq!(exile.object_view(own_keep).zone, Zone::Graveyard);
    assert_eq!(exile.top_library_card(player), Some(top));

    act(&mut game, activate(lantern, 1));
    assert_eq!(game.mana_payment_domain().unwrap().generic, 1);
    pay(&mut game);
    assert_eq!(game.object_view(lantern).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(top).zone, Zone::Hand);
    assert_eq!(game.object_view(foreign).zone, Zone::Graveyard);
}

#[test]
fn scream_puff_makes_food_for_combat_damage_to_players_and_witch_pays_with_either_food_kind() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let puff = put(&mut game, player, "Scream Puff", Zone::Hand);
    let witch = put(&mut game, player, "Sweettooth Witch", Zone::Hand);
    let song = put(&mut game, player, "Song of Totentanz", Zone::Hand);
    let giant = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
    let nontoken = put(&mut game, player, "Gingerbrute", Zone::Battlefield);
    let vault = put(&mut game, player, "Collector's Vault", Zone::Battlefield);
    let foreign_food = spawn(&mut game, opponent, FOOD, Zone::Battlefield);
    let swamps = [(); 2].map(|_| put(&mut game, player, "Swamp", Zone::Battlefield));
    cast(&mut game, witch, None);
    settle(&mut game);
    assert_eq!(stats(&game, witch), (3, 2));
    let first_food = tokens(&game, player, FOOD)[0].id;
    cast(&mut game, puff, None);
    settle(&mut game);
    assert_eq!(stats(&game, puff), (4, 5));
    assert!(
        game.object_view(puff)
            .keywords
            .contains(Keywords::DEATHTOUCH)
    );

    act(&mut game, Action::AnnounceCast { card: song });
    act(&mut game, Action::ChooseX { value: 0 });
    pay(&mut game);
    settle(&mut game);
    reach_attackers(&mut game);
    act(&mut game, Action::Attack { card: puff });
    act(&mut game, Action::FinishAttackers);
    pass_pair(&mut game);
    let mut blocked = game.clone();
    act(
        &mut blocked,
        Action::Block {
            blocker: giant,
            attacker: puff,
        },
    );
    act(&mut blocked, Action::FinishBlockers);
    pass_pair(&mut blocked);
    settle(&mut blocked);
    assert_eq!(blocked.object_view(giant).zone, Zone::Graveyard);
    assert_eq!(blocked.object_view(puff).zone, Zone::Graveyard);
    assert_eq!(blocked.state.players[opponent.index()].life, 20);
    assert_eq!(tokens(&blocked, player, FOOD).len(), 1);

    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 16);
    assert_eq!(tokens(&game, player, FOOD).len(), 2);
    for card in swamps {
        act(&mut game, Action::TapMana { card });
    }

    let mut self_target = game.clone();
    for (branch, recipient, food, expected) in [
        (&mut game, opponent, first_food, 14),
        (&mut self_target, player, nontoken, 18),
    ] {
        act(branch, activate(witch, 0));
        act(
            branch,
            Action::ChooseTarget {
                target: Target::Player(recipient),
            },
        );
        for card in [vault, foreign_food] {
            let object = branch.object_ref(card);
            reject(branch, Action::ChooseAdditionalCost { object });
        }
        let object = branch.object_ref(food);
        act(branch, Action::ChooseAdditionalCost { object });
        let domain = branch.mana_payment_domain().unwrap();
        assert_eq!(domain.generic, 2);
        pay(branch);
        assert_ne!(branch.object_view(food).zone, Zone::Battlefield);
        assert_eq!(branch.state.players[recipient.index()].life, expected + 2);
        settle(branch);
        assert_eq!(branch.state.players[recipient.index()].life, expected);
    }
}

#[test]
fn stingblade_assassin_flashes_in_and_checks_damage_history_and_opposing_control() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let assassin = put(&mut game, player, "Stingblade Assassin", Zone::Hand);
    let friendly = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let damaged = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let untouched = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let shocks = [(); 2].map(|_| put(&mut game, player, "Shock", Zone::Hand));
    for (spell, recipient) in shocks.into_iter().zip([friendly, damaged]) {
        act(&mut game, Action::Pass);
        cast(&mut game, spell, Some(Target::Card(recipient)));
        settle(&mut game);
        assert_eq!(game.object_view(recipient).damage, 2);
    }
    // Clear the mark after actual damage to distinguish the turn-history predicate.
    game.state.objects[damaged as usize].damage = 0;
    act(&mut game, Action::Pass);
    assert_eq!(game.state.active, opponent);
    cast(&mut game, assassin, None);
    pass_pair(&mut game);
    assert_eq!(stats(&game, assassin), (3, 1));
    assert!(
        game.object_view(assassin)
            .keywords
            .contains(Keywords::FLYING.union(Keywords::FLASH))
    );
    reject(&mut game, target(friendly));
    reject(&mut game, target(untouched));
    act(&mut game, target(damaged));
    settle(&mut game);
    assert_eq!(game.object_view(damaged).zone, Zone::Graveyard);
    assert_eq!(game.object_view(friendly).zone, Zone::Battlefield);
    assert_eq!(game.object_view(untouched).zone, Zone::Battlefield);
}

#[test]
fn stroke_of_midnight_creates_a_human_for_the_target_controller_on_a_legal_resolution() {
    const HUMAN: Card = Card::from_catalogue_id(5254);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let stroke = put(&mut game, player, "Stroke of Midnight", Zone::Hand);
    let shield = put(&mut game, player, "Take Up the Shield", Zone::Hand);
    let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
    let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let land = put(&mut game, opponent, "Forest", Zone::Battlefield);
    game.change_controller(game.object_ref(creature), opponent);
    act(&mut game, Action::AnnounceCast { card: stroke });
    reject(&mut game, target(land));
    act(&mut game, target(creature));
    pay(&mut game);

    let mut stale = game.clone();
    cast(&mut stale, bounce, Some(Target::Card(creature)));
    settle(&mut stale);
    assert_eq!(stale.object_view(creature).zone, Zone::Hand);
    assert!(tokens(&stale, opponent, HUMAN).is_empty());

    let mut indestructible = game.clone();
    cast(&mut indestructible, shield, Some(Target::Card(creature)));
    settle(&mut indestructible);
    assert_eq!(indestructible.object_view(creature).zone, Zone::Battlefield);
    assert_eq!(tokens(&indestructible, opponent, HUMAN).len(), 1);

    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
    assert!(tokens(&game, player, HUMAN).is_empty());
    let humans = tokens(&game, opponent, HUMAN);
    assert_eq!(humans.len(), 1);
    assert_eq!(
        (humans[0].power, humans[0].toughness, humans[0].colors),
        (1, 1, 1)
    );
    assert!(humans[0].has_subtype("Human"));
}

#[test]
fn sugar_rush_adds_only_power_and_draws_only_on_a_legal_resolution() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let rush = put(&mut game, player, "Sugar Rush", Zone::Hand);
    let creature = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let bounce = put(&mut game, opponent, "Unsummon", Zone::Hand);
    let top = put(&mut game, player, "Opt", Zone::Library);
    cast(&mut game, rush, Some(Target::Card(creature)));

    let mut stale = game.clone();
    act(&mut stale, Action::Pass);
    cast(&mut stale, bounce, Some(Target::Card(creature)));
    settle(&mut stale);
    assert_eq!(stale.object_view(creature).zone, Zone::Hand);
    assert_eq!(stale.object_view(top).zone, Zone::Library);
    assert_eq!(stale.state.players[player.index()].cards_drawn_this_turn, 0);

    settle(&mut game);
    assert_eq!(stats(&game, creature), (5, 2));
    assert_eq!(game.object_view(top).zone, Zone::Hand);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    finish_turn(&mut game);
    assert_eq!(stats(&game, creature), (2, 2));
}
