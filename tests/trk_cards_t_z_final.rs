//! Final T–Z cards with source-exact permissions and entry obligations.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const ADD_LORE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield, kinds: 0, relation: 0,
            other: false, nonland: false, tapped: false,
            min_mana: 0, max_mana: u32::MAX,
        })),
        &[Instruction::PutCounters {
            subject: Subject::Targets, kind: CounterKind::Lore, amount: 1,
        }],
    ),
    ..CardRules::EMPTY
};

fn drain(game: &mut Game) {
    for _ in 0..48 {
        if game.state.pending == Pending::Priority && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty() && game.state.resolving.is_none() {
            return;
        }
        choose(game, |action| matches!(action, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("resolution budget exhausted");
}

fn advance_turn(game: &mut Game, current: Player) {
    game.state.step = Step::End;
    game.state.priority = current;
    for _ in 0..16 {
        if game.state.active != current { return; }
        choose(game, |action| matches!(action, Action::Pass));
    }
    panic!("turn did not advance");
}

fn drain_target(game: &mut Game, target: CardId) {
    for _ in 0..64 {
        if game.state.pending == Pending::Priority && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty() && game.state.resolving.is_none() {
            return;
        }
        let actions = game.legal_actions();
        if actions.contains(&Action::ChooseTarget { target: Target::Card(target) }) {
            act(game, Action::ChooseTarget { target: Target::Card(target) });
        } else if actions.contains(&Action::FinishStackTargets) {
            act(game, Action::FinishStackTargets);
        } else {
            choose(game, |action| matches!(action, Action::Pass | Action::PutTrigger { .. }));
        }
    }
    panic!("targeted resolution budget exhausted");
}

fn top_casts(game: &Game, card: CardId) -> Vec<Action> {
    game.legal_actions().into_iter().filter(|action| matches!(action,
        Action::AnnounceLibraryTop { object, method: CastMethod::Normal, .. }
            if *object == game.object_ref(card)
    )).collect()
}

#[test]
fn zefram_looks_privately_casts_one_top_artifact_and_adds_spacecraft_charge_counters() {
    let mut game = scenario(Player::P0);
    let zefram = put(&mut game, Player::P0, "Zefram Cochrane, Warp Pioneer", Zone::Battlefield);
    let next = put(&mut game, Player::P0, "Tricorder", Zone::Library);
    let first = put(&mut game, Player::P0, "Short Bow", Zone::Library);
    assert_eq!(game.observe(Player::P0).known_library_top[0].unwrap().object,
        game.object_ref(first));
    assert!(game.observe(Player::P1).known_library_top[0].is_none());
    let action = top_casts(&game, first).into_iter().find(|action| matches!(action,
        Action::AnnounceLibraryTop { permission, .. }
            if permission.source == game.object_ref(zefram)
    )).expect("Zefram permits one artifact cast from top");
    act(&mut game, action);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Battlefield);
    assert!(top_casts(&game, next).is_empty());
    assert_eq!(game.observe(Player::P0).known_library_top[0].unwrap().object,
        game.object_ref(next));
    assert!(game.observe(Player::P1).known_library_top[0].is_none());

    let spacecraft = put(&mut game, Player::P0, "Romulan Bird-of-Prey", Zone::Hand);
    cast(&mut game, spacecraft, None);
    settle(&mut game);
    assert_eq!(game.object_view(spacecraft).zone, Zone::Battlefield);
    assert_eq!(game.counter_count(spacecraft, CounterKind::Charge), 2);
    assert_eq!(game.counter_count(first, CounterKind::Charge), 0);

    game.move_object(zefram, Zone::Graveyard);
    assert!(game.observe(Player::P0).known_library_top[0].is_none());
    let later = put(&mut game, Player::P0, "Romulan Bird-of-Prey", Zone::Hand);
    cast(&mut game, later, None);
    settle(&mut game);
    assert_eq!(game.counter_count(later, CounterKind::Charge), 0);
}

#[test]
fn targ_wrangler_sacrifices_only_after_its_exact_attack_and_never_a_land() {
    let mut quiet = scenario(Player::P0);
    let targ = put(&mut quiet, Player::P0, "Targ Wrangler", Zone::Battlefield);
    assert!(quiet.object_view(targ).keywords.contains(Keywords::MENACE));
    quiet.begin_step(Step::End);
    quiet.prepare_triggers();
    assert!(quiet.state.pending_triggers.is_empty());

    let mut game = scenario(Player::P0);
    let targ = put(&mut game, Player::P0, "Targ Wrangler", Zone::Battlefield);
    let land = put(&mut game, Player::P0, "Forest", Zone::Battlefield);
    let artifact = put(&mut game, Player::P0, "Short Bow", Zone::Battlefield);
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: targ });
    act(&mut game, Action::FinishAttackers);
    game.begin_step(Step::End);
    game.prepare_triggers();
    for _ in 0..16 {
        if game.legal_actions().contains(&Action::Sacrifice { object: game.object_ref(artifact) }) {
            break;
        }
        choose(&mut game, |action| matches!(action, Action::Pass | Action::PutTrigger { .. }));
    }
    let land_ref = game.object_ref(land);
    reject(&mut game, Action::Sacrifice { object: land_ref });
    let artifact_ref = game.object_ref(artifact);
    act(&mut game, Action::Sacrifice { object: artifact_ref });
    drain(&mut game);
    assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
}

#[test]
fn the_doctor_targets_any_creature_then_adds_one_counter_to_each_qualified_ally() {
    let mut game = scenario(Player::P0);
    let fresh = put(&mut game, Player::P0, "Bear Cub", Zone::Battlefield);
    let qualified = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, Player::P1, "Hill Giant", Zone::Battlefield);
    game.add_counter(qualified, CounterKind::Stun, 1);
    let doctor = put(&mut game, Player::P0, "The Doctor, EMH", Zone::Hand);
    cast(&mut game, doctor, None);
    for _ in 0..16 {
        if game.legal_actions().contains(&Action::ChooseTarget { target: Target::Card(fresh) }) {
            break;
        }
        choose(&mut game, |action| matches!(action, Action::Pass | Action::PutTrigger { .. }));
    }
    act(&mut game, Action::ChooseTarget { target: Target::Card(fresh) });
    if game.legal_actions().contains(&Action::FinishStackTargets) {
        act(&mut game, Action::FinishStackTargets);
    }
    drain(&mut game);
    assert_eq!(game.counter_count(fresh, CounterKind::PlusOne), 2);
    assert_eq!(game.counter_count(qualified, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(qualified, CounterKind::Stun), 1);
    assert_eq!(game.counter_count(doctor, CounterKind::PlusOne), 0);
    assert_eq!(game.counter_count(enemy, CounterKind::PlusOne), 0);
}

#[test]
fn worf_takes_one_small_opponent_creature_and_all_grants_expire_together() {
    let mut game = scenario(Player::P0);
    let victim = put(&mut game, Player::P1, "Bear Cub", Zone::Battlefield);
    game.state.objects[victim as usize].tapped = true;
    let too_large = put(&mut game, Player::P1, "Hill Giant", Zone::Battlefield);
    let worf = put(&mut game, Player::P0, "Worf, Intelligence Contractor", Zone::Hand);
    cast(&mut game, worf, None);
    for _ in 0..16 {
        if game.legal_actions().contains(&Action::ChooseTarget { target: Target::Card(victim) }) {
            break;
        }
        choose(&mut game, |action| matches!(action, Action::Pass | Action::PutTrigger { .. }));
    }
    reject(&mut game, Action::ChooseTarget { target: Target::Card(too_large) });
    act(&mut game, Action::ChooseTarget { target: Target::Card(victim) });
    if game.legal_actions().contains(&Action::FinishStackTargets) {
        act(&mut game, Action::FinishStackTargets);
    }
    drain(&mut game);
    assert_eq!(game.object_view(victim).controller, Player::P0);
    assert!(!game.object_view(victim).tapped);
    assert!(game.object_view(victim).has_subtype("Warrior"));
    assert!(game.object_view(victim).has_subtype("Bear"));
    assert!(game.object_view(victim).keywords.contains(Keywords::HASTE));
    advance_turn(&mut game, Player::P0);
    assert_eq!(game.object_view(victim).controller, Player::P1);
    assert!(!game.object_view(victim).has_subtype("Warrior"));
    assert!(!game.object_view(victim).keywords.contains(Keywords::HASTE));
}

#[test]
fn tasha_dies_into_next_turn_play_and_her_grave_activation_copies_printed_haste() {
    let mut game = scenario(Player::P0);
    let top = put(&mut game, Player::P0, "Forest", Zone::Library);
    let tasha = put(&mut game, Player::P0, "Tasha Yar, Out of Time", Zone::Battlefield);
    assert!(game.object_view(tasha).keywords.contains(Keywords::HASTE));
    game.move_object(tasha, Zone::Graveyard);
    drain(&mut game);
    assert_eq!(game.object_view(top).zone, Zone::Exile);
    assert!(game.observe(Player::P0).exile_permissions.iter().any(|entry|
        entry.object == game.object_ref(top)));
    assert!(game.observe(Player::P1).exile_permissions.iter().any(|entry|
        entry.object == game.object_ref(top)));

    game.state.priority = Player::P0;
    act(&mut game, Action::AnnounceActivate {
        card: tasha, ability: 0, counter_payment: Counters::default(),
    });
    pay(&mut game);
    assert_eq!(game.object_view(tasha).zone, Zone::Exile);
    drain(&mut game);
    let token = game.controlled_token_ids(Player::P0)
        .find(|&id| game.object_view(id).card == Card::from_name("Tasha Yar, Out of Time").unwrap())
        .expect("printed Tasha token copy");
    assert_eq!(stats(&game, token), (2, 1));
    assert!(game.object_view(token).keywords.contains(Keywords::HASTE));
    assert!(game.observe(Player::P0).delayed_sacrifices.iter().any(|row|
        row.tokens.contains(&game.object_ref(token))));
}

#[test]
fn zero_exiles_only_a_nonland_privately_and_owner_pays_two_extra_to_cast() {
    let mut game = scenario(Player::P0);
    let victim = put(&mut game, Player::P1, "Bear Cub", Zone::Hand);
    let land = put(&mut game, Player::P1, "Forest", Zone::Hand);
    let zero = put(&mut game, Player::P0, "Zero, Incorporeal Sensate", Zone::Hand);
    cast(&mut game, zero, None);
    for _ in 0..16 {
        if game.legal_actions().contains(&Action::ChooseTarget { target: Target::Player(Player::P1) }) {
            break;
        }
        choose(&mut game, |action| matches!(action, Action::Pass | Action::PutTrigger { .. }));
    }
    act(&mut game, Action::ChooseTarget { target: Target::Player(Player::P1) });
    if game.legal_actions().contains(&Action::FinishStackTargets) {
        act(&mut game, Action::FinishStackTargets);
    }
    for _ in 0..16 {
        if game.legal_actions().iter().any(|action| matches!(action, Action::ChooseHandCard { .. })) {
            break;
        }
        choose(&mut game, |action| matches!(action, Action::Pass | Action::PutTrigger { .. }));
    }
    assert!(game.observe(Player::P1).hand_choice.is_none());
    let inspected = game.observe(Player::P0).hand_choice.unwrap();
    assert_eq!(inspected.inspected_cards.unwrap().len(), 2);
    let land_ref = game.object_ref(land);
    reject(&mut game, Action::ChooseHandCard { object: Some(land_ref) });
    let action = Action::ChooseHandCard { object: Some(game.object_ref(victim)) };
    let checkpoint = game.snapshot();
    let actor = game.actor().unwrap();
    let revision = game.revision();
    let index = game.legal_actions().iter().position(|candidate| *candidate == action).unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(actor, revision, index).unwrap();
    act(&mut game, action);
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&checkpoint).unwrap();
    assert_eq!(indexed.apply_index(actor, revision, index), Err(Error::StaleRevision));
    drain(&mut game);
    assert!(game.object_view(zero).keywords.contains(Keywords::LIFELINK));
    assert_eq!(game.object_view(victim).zone, Zone::Exile);
    assert_eq!(game.object_view(land).zone, Zone::Hand);

    game.state.active = Player::P1;
    game.state.priority = Player::P1;
    game.state.step = Step::Main1;
    let taxed = game.legal_actions().into_iter().find(|action| matches!(action,
        Action::AnnounceTaxedExile { object, surcharge, .. }
            if *object == game.object_ref(victim) && surcharge.get() == 2
    )).expect("owner can cast exact exiled creature for two extra");
    act(&mut game, taxed);
    assert!(game.legal_actions().iter().any(|action| matches!(action,
        Action::PayCosts { payment } if payment.iter().copied().sum::<u32>() == 4
    )));
    pay(&mut game);
    drain(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Battlefield);
    assert_eq!(game.object_view(victim).controller, Player::P1);
}

#[test]
fn those_old_scientists_draws_discards_returns_a_typed_card_and_shuffles_both_owners() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &ADD_LORE)]);
    let mut game = scenario(Player::P0);
    let bow = put(&mut game, Player::P0, "Short Bow", Zone::Hand);
    let shock = put(&mut game, Player::P0, "Shock", Zone::Hand);
    let invalid_land = put(&mut game, Player::P0, "Forest", Zone::Graveyard);
    let enemy_grave = put(&mut game, Player::P1, "Hill Giant", Zone::Graveyard);
    let saga = put(&mut game, Player::P0, "Those Old Scientists", Zone::Hand);
    cast(&mut game, saga, None);
    for _ in 0..16 {
        if game.legal_actions().contains(&Action::Discard { card: bow }) { break; }
        choose(&mut game, |action| matches!(action, Action::Pass | Action::PutTrigger { .. }));
    }
    act(&mut game, Action::Discard { card: bow });
    act(&mut game, Action::Discard { card: shock });
    drain(&mut game);
    assert_eq!(game.state.players[0].cards_drawn_this_turn, 2);
    assert_eq!(game.object_view(bow).zone, Zone::Graveyard);
    assert_eq!(game.object_view(shock).zone, Zone::Graveyard);

    let lore = put(&mut game, Player::P0, "Opt", Zone::Hand);
    cast(&mut game, lore, Some(Target::Card(saga)));
    for _ in 0..16 {
        if game.legal_actions().contains(&Action::ChooseTarget { target: Target::Card(bow) }) { break; }
        choose(&mut game, |action| matches!(action, Action::Pass | Action::PutTrigger { .. }));
    }
    reject(&mut game, Action::ChooseTarget { target: Target::Card(invalid_land) });
    act(&mut game, Action::ChooseTarget { target: Target::Card(bow) });
    if game.legal_actions().contains(&Action::FinishStackTargets) {
        act(&mut game, Action::FinishStackTargets);
    }
    drain(&mut game);
    assert_eq!(game.object_view(bow).zone, Zone::Hand);

    let lore = put(&mut game, Player::P0, "Opt", Zone::Hand);
    cast(&mut game, lore, Some(Target::Card(saga)));
    for _ in 0..16 {
        if game.legal_actions().contains(&Action::ChooseTarget { target: Target::Card(shock) }) { break; }
        choose(&mut game, |action| matches!(action, Action::Pass | Action::PutTrigger { .. }));
    }
    act(&mut game, Action::ChooseTarget { target: Target::Card(shock) });
    act(&mut game, Action::ChooseTarget { target: Target::Card(enemy_grave) });
    if game.legal_actions().contains(&Action::FinishStackTargets) {
        act(&mut game, Action::FinishStackTargets);
    }
    drain(&mut game);
    assert_eq!(game.object_view(shock).zone, Zone::Library);
    assert_eq!(game.object_view(enemy_grave).zone, Zone::Library);
    assert!(game.state.players[0].library.contains(&shock));
    assert!(game.state.players[1].library.contains(&enemy_grave));
    assert_eq!(game.object_view(saga).zone, Zone::Graveyard);
}

#[test]
fn inner_light_returns_colored_creature_draws_combat_damage_and_pays_final_chapter() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &ADD_LORE)]);
    let mut game = scenario(Player::P0);
    let creature = put(&mut game, Player::P0, "Grizzly Bears", Zone::Graveyard);
    let saga = put(&mut game, Player::P0, "The Inner Light", Zone::Hand);
    cast(&mut game, saga, None);
    drain_target(&mut game, creature);
    assert_eq!(game.object_view(creature).zone, Zone::Battlefield);

    let lore = put(&mut game, Player::P0, "Opt", Zone::Hand);
    cast(&mut game, lore, Some(Target::Card(saga)));
    drain_target(&mut game, creature);
    assert_eq!(game.counter_count(saga, CounterKind::Lore), 2);
    assert_eq!(game.observe(Player::P0).delayed_target_events.unwrap().len(), 1);

    game.state.objects[creature as usize].summoning_sick = false;
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: creature });
    act(&mut game, Action::FinishAttackers);
    for _ in 0..32 {
        if game.state.players[1].life == 18 && game.state.pending == Pending::Priority
            && game.state.stack.is_empty() && game.state.pending_triggers.is_empty() { break; }
        choose(&mut game, |action| matches!(action,
            Action::Pass | Action::PutTrigger { .. } | Action::FinishBlockers));
    }
    assert_eq!(game.state.players[1].life, 18);
    assert_eq!(game.state.players[0].cards_drawn_this_turn, 2);

    let lore = put(&mut game, Player::P0, "Opt", Zone::Hand);
    game.state.step = Step::Main2;
    game.state.priority = Player::P0;
    game.state.players[Player::P0.index()].mana = [8; 6];
    cast(&mut game, lore, Some(Target::Card(saga)));
    drain(&mut game);
    assert_eq!(game.state.players[0].life, 22);
    assert_eq!(game.state.objects.iter().filter(|object|
        object.zone == Zone::Battlefield && object.card == Card(5302)).count(), 1);
    assert_eq!(game.object_view(saga).zone, Zone::Graveyard);
}
