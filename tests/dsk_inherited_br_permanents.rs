//! Paid death, attack, and activation scenarios for inherited Duskmourn permanents.
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

fn activation(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

fn cost(game: &Game, card: CardId) -> Action {
    Action::ChooseAdditionalCost {
        object: game.object_ref(card),
    }
}

fn murder(game: &mut Game, player: Player, creature: CardId) {
    let spell = put(game, player, "Murder", Zone::Hand);
    cast(game, spell, Some(Target::Card(creature)));
    pass_pair(game);
}

fn declare_attack(game: &mut Game, attacker: CardId) {
    game.begin_step(Step::BeginCombat);
    pass_pair(game);
    act(game, Action::Attack { card: attacker });
    act(game, Action::FinishAttackers);
}

fn resolve_targeted_triggers(game: &mut Game, expected: Target) {
    for _ in 0..30 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        choose(game, |action| match action {
            Action::ChooseTarget { target } => *target == expected,
            Action::Pass | Action::PutTrigger { .. } | Action::FinishTargets => true,
            _ => false,
        });
    }
    panic!("inherited card trigger did not settle");
}

#[test]
fn final_vengeance_pays_a_creature_or_enchantment_and_triggers_the_sacrifice_rewards() {
    let mut game = scenario(PLAYER);
    let pig = put(&mut game, PLAYER, "Piggy Bank", Zone::Battlefield);
    let observer = put(&mut game, PLAYER, "Popular Egotist", Zone::Battlefield);
    let enchantment = put(
        &mut game,
        PLAYER,
        "Bottomless Pool // Locker Room",
        Zone::Battlefield,
    );
    let land = put(&mut game, PLAYER, "Forest", Zone::Battlefield);
    let victim = put(&mut game, OTHER, "Hill Giant", Zone::Battlefield);
    let spell = put(&mut game, PLAYER, "Final Vengeance", Zone::Hand);
    let new_id = game.state.objects.len() as CardId;
    game.state.players[PLAYER.index()].mana = [0, 0, 1, 0, 0, 0];
    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, target(Target::Card(victim)));
    let land_payment = cost(&game, land);
    reject(&mut game, land_payment);
    let mut alternative = game.clone();
    for (branch, payment) in [(&mut game, pig), (&mut alternative, enchantment)] {
        let pay_with = cost(branch, payment);
        act(branch, pay_with);
        pay(branch);
        assert_eq!(branch.object_view(payment).zone, Zone::Graveyard);
        assert_eq!(branch.object_view(victim).zone, Zone::Battlefield);
        resolve_targeted_triggers(branch, Target::Player(OTHER));
        assert_eq!(branch.object_view(victim).zone, Zone::Exile);
        assert_eq!(branch.state.players[PLAYER.index()].life, 21);
        assert_eq!(branch.state.players[OTHER.index()].life, 19);
        assert_eq!(stats(branch, observer), (3, 2));
    }
    let token = game.object_view(new_id);
    assert!(token.is_token() && token.has_subtype("Treasure"));
    assert!(token.card.is_artifact());
}

#[test]
fn popular_egotist_pays_another_permanent_and_taps_as_the_indestructible_effect_resolves() {
    let mut game = scenario(PLAYER);
    let egotist = put(&mut game, PLAYER, "Popular Egotist", Zone::Battlefield);
    let fodder = put(&mut game, PLAYER, "Grizzly Bears", Zone::Battlefield);
    let removal = put(&mut game, PLAYER, "Murder", Zone::Hand);
    act(&mut game, activation(egotist));
    let self_payment = cost(&game, egotist);
    reject(&mut game, self_payment);
    let pay_with = cost(&game, fodder);
    act(&mut game, pay_with);
    pay(&mut game);
    assert_eq!(game.object_view(fodder).zone, Zone::Graveyard);
    assert!(!game.object_view(egotist).tapped);
    resolve_targeted_triggers(&mut game, Target::Player(OTHER));
    assert!(game.object_view(egotist).tapped);
    assert!(game.keywords(egotist).contains(Keywords::INDESTRUCTIBLE));
    assert_eq!(game.state.players[PLAYER.index()].life, 21);
    cast(&mut game, removal, Some(Target::Card(egotist)));
    settle(&mut game);
    assert_eq!(game.object_view(egotist).zone, Zone::Battlefield);
}

#[test]
fn clockwork_percussionist_and_diversion_specialist_grant_different_play_deadlines() {
    let mut game = scenario(PLAYER);
    let clock = put(&mut game, PLAYER, "Clockwork Percussionist", Zone::Hand);
    let specialist = put(&mut game, PLAYER, "Diversion Specialist", Zone::Battlefield);
    let prey = put(&mut game, PLAYER, "Appendage Amalgam", Zone::Battlefield);
    let first = put(&mut game, PLAYER, "Shock", Zone::Library);
    let second = put(&mut game, PLAYER, "Opt", Zone::Library);
    cast(&mut game, clock, None);
    settle(&mut game);
    assert!(game.keywords(clock).contains(Keywords::HASTE));
    assert!(game.keywords(specialist).contains(Keywords::MENACE));

    murder(&mut game, PLAYER, clock);
    settle(&mut game);
    assert_eq!(game.object_view(second).zone, Zone::Exile);
    act(&mut game, activation(specialist));
    let self_payment = cost(&game, specialist);
    reject(&mut game, self_payment);
    let pay_with = cost(&game, prey);
    act(&mut game, pay_with);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Exile);
    let view = game.observe(PLAYER);
    for (card, expiry) in [
        (first, ExilePlayExpiry::Turn(game.state.turn)),
        (second, ExilePlayExpiry::EndOfNextTurn),
    ] {
        assert!(view.exile_permissions.iter().any(|row| {
            row.player == PLAYER && row.object == game.object_ref(card) && row.expiry == expiry
        }));
    }
    act(&mut game, Action::AnnounceExile { card: second });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(second).zone, Zone::Graveyard);
}

#[test]
fn fear_of_lost_teeth_targets_after_death_and_gains_one_life() {
    let mut game = scenario(PLAYER);
    let fear = put(&mut game, PLAYER, "Fear of Lost Teeth", Zone::Battlefield);
    murder(&mut game, PLAYER, fear);
    assert_eq!(game.object_view(fear).zone, Zone::Graveyard);
    act(&mut game, target(Target::Player(OTHER)));
    settle(&mut game);
    assert_eq!(game.state.players[PLAYER.index()].life, 21);
    assert_eq!(game.state.players[OTHER.index()].life, 19);
}

#[test]
fn the_attack_triggers_apply_to_the_declared_group_and_appendage_surveils_its_own_attack() {
    let mut off_turn = scenario(OTHER);
    let flash = put(&mut off_turn, PLAYER, "Appendage Amalgam", Zone::Hand);
    act(&mut off_turn, Action::Pass);
    cast(&mut off_turn, flash, None);
    settle(&mut off_turn);
    assert_eq!(off_turn.object_view(flash).zone, Zone::Battlefield);

    let mut game = scenario(PLAYER);
    let appendage = put(&mut game, PLAYER, "Appendage Amalgam", Zone::Battlefield);
    let top = put(&mut game, PLAYER, "Shock", Zone::Library);
    declare_attack(&mut game, appendage);
    pass_pair(&mut game);
    assert!(game.observe(PLAYER).library_choice.unwrap().surveil);
    assert!(game.observe(OTHER).library_choice.is_none());
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: top,
            destination: LibraryDestination::Graveyard,
        },
    );
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);

    let mut game = scenario(PLAYER);
    let slayer = put(&mut game, PLAYER, "Most Valuable Slayer", Zone::Battlefield);
    let caller = put(&mut game, PLAYER, "Razorkin Hordecaller", Zone::Battlefield);
    let attacker = put(&mut game, PLAYER, "Grizzly Bears", Zone::Battlefield);
    let next = game.state.objects.len() as CardId;
    assert!(game.keywords(caller).contains(Keywords::HASTE));
    declare_attack(&mut game, attacker);
    resolve_targeted_triggers(&mut game, Target::Card(attacker));
    assert!(game.keywords(attacker).contains(Keywords::FIRST_STRIKE));
    assert_eq!(stats(&game, attacker), (3, 2));
    assert_eq!(stats(&game, slayer), (2, 4));
    let token = game.object_view(next);
    assert!(token.is_token() && token.has_subtype("Gremlin"));
    assert_eq!((token.colors, token.power, token.toughness), (8, 1, 1));
    assert!(!game.state.attackers.iter().any(|row| row.card == next));
}

#[test]
fn ripchain_and_valgavoths_faithful_pay_distinct_sacrifices() {
    let mut game = scenario(PLAYER);
    let ripchain = put(&mut game, PLAYER, "Ripchain Razorkin", Zone::Battlefield);
    let land = put(&mut game, PLAYER, "Mountain", Zone::Battlefield);
    let faithful = put(&mut game, PLAYER, "Valgavoth's Faithful", Zone::Battlefield);
    let yours = put(&mut game, PLAYER, "Grizzly Bears", Zone::Graveyard);
    let theirs = put(&mut game, OTHER, "Hill Giant", Zone::Graveyard);
    assert!(game.keywords(ripchain).contains(Keywords::REACH));
    act(&mut game, activation(ripchain));
    let invalid = cost(&game, faithful);
    reject(&mut game, invalid);
    let land_payment = cost(&game, land);
    act(&mut game, land_payment);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Graveyard);
    assert_eq!(game.state.players[PLAYER.index()].cards_drawn_this_turn, 1);

    let mut combat = game.clone();
    combat.state.step = Step::BeginCombat;
    reject(&mut combat, activation(faithful));
    act(&mut game, activation(faithful));
    reject(&mut game, target(Target::Card(theirs)));
    act(&mut game, target(Target::Card(yours)));
    pay(&mut game);
    assert_eq!(game.object_view(faithful).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(yours).zone, Zone::Battlefield);
    assert_eq!(game.object_view(theirs).zone, Zone::Graveyard);
}

#[test]
fn turn_inside_out_and_vicious_clown_interact_with_the_exact_manifested_creature() {
    let mut game = scenario(PLAYER);
    let clown = put(&mut game, PLAYER, "Vicious Clown", Zone::Hand);
    let large = put(&mut game, PLAYER, "Hill Giant", Zone::Hand);
    let marked = put(&mut game, PLAYER, "Grizzly Bears", Zone::Battlefield);
    let card = put(&mut game, PLAYER, "Island", Zone::Library);
    let spell = put(&mut game, PLAYER, "Turn Inside Out", Zone::Hand);
    for creature in [clown, large] {
        cast(&mut game, creature, None);
        settle(&mut game);
        assert_eq!(stats(&game, clown), (2, 3));
    }
    cast(&mut game, spell, Some(Target::Card(marked)));
    settle(&mut game);
    assert_eq!(stats(&game, marked), (5, 2));
    murder(&mut game, PLAYER, marked);
    pass_pair(&mut game);
    let chosen = game.object_ref(card);
    act(&mut game, Action::ChooseManifest { object: chosen });
    settle(&mut game);
    assert!(game.is_face_down(game.object_ref(card)));
    assert_eq!(stats(&game, card), (2, 2));
    assert_eq!(stats(&game, clown), (4, 3));
}
