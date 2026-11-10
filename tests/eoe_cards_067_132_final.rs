//! Paid card scenarios for Edge of Eternities collectors 123–132.
mod support;

use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..80 {
        if ready(g) {
            return;
        }
        if let Some(range) = g.damage_assignment_range() {
            act(
                g,
                Action::AssignDamage {
                    target: range.target,
                    amount: range.maximum,
                },
            );
        } else {
            choose(g, |a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                        | Action::ChooseEntryTimestamp { .. }
                )
            });
        }
    }
    panic!(
        "advance budget exhausted at {:?} {:?}",
        g.state.step, g.state.pending
    );
}

fn quiet(g: &mut Game) {
    g.prepare_triggers();
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
    });
}

fn target(target: Target) -> Action {
    Action::ChooseTarget { target }
}

fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn life(g: &Game, player: Player) -> i128 {
    g.state.players[player.index()].life
}

#[test]
fn umbral_collar_zealot_sacrifices_another_creature_or_artifact_to_surveil_privately() {
    let p = Player::P0;
    for name in ["Candy Trail", "Bear Cub"] {
        let mut g = scenario(p);
        let zealot = put(&mut g, p, "Umbral Collar Zealot", Zone::Battlefield);
        let cost = put(&mut g, p, name, Zone::Battlefield);
        let top = put(&mut g, p, "Island", Zone::Library);
        act(&mut g, activate(zealot, 0));
        let object = g.object_ref(zealot);
        reject(&mut g, Action::ChooseAdditionalCost { object });
        let object = g.object_ref(cost);
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        assert_eq!(g.object_view(cost).zone, Zone::Graveyard);
        assert!(!g.object_view(zealot).tapped);
        until(&mut g, |g| g.observe(p).library_choice.is_some());
        assert!(g.observe(p.other()).library_choice.is_none());
        act(
            &mut g,
            Action::ArrangeLibrary {
                card: top,
                destination: LibraryDestination::Graveyard,
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(top).zone, Zone::Graveyard);
    }
}

#[test]
fn virus_beetle_and_starfield_vocalist_give_the_opponent_two_private_discards() {
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Starfield Vocalist", Zone::Battlefield);
    let beetle = put(&mut g, p, "Virus Beetle", Zone::Hand);
    let first = put(&mut g, p.other(), "Island", Zone::Hand);
    let second = put(&mut g, p.other(), "Swamp", Zone::Hand);
    let third = put(&mut g, p.other(), "Forest", Zone::Hand);
    cast(&mut g, beetle, None);
    for discard in [second, first] {
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectDiscard { .. })
        });
        assert_eq!(g.actor(), Some(p.other()));
        assert!(g.observe(p).objects.iter().all(|o| o.id != discard));
        reject(&mut g, Action::ChooseOptional { accept: false });
        act(&mut g, Action::Discard { card: discard });
    }
    quiet(&mut g);
    for discard in [first, second] {
        assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
    }
    assert_eq!(g.object_view(third).zone, Zone::Hand);
    let view = g.object_view(beetle);
    assert_eq!(view.kinds & 17, 17);
    assert_eq!(stats(&g, beetle), (1, 1));
}

#[test]
fn voidforged_titan_only_triggers_on_its_own_void_end_step_and_survives_source_departure() {
    let p = Player::P0;
    let mut g = scenario(p);
    let titan = put(&mut g, p, "Voidforged Titan", Zone::Hand);
    let witness = put(&mut g, p.other(), "Candy Trail", Zone::Battlefield);
    cast(&mut g, titan, None);
    settle(&mut g);
    let mut no_void = g.clone();
    no_void.begin_step(Step::End);
    quiet(&mut no_void);
    assert_eq!(no_void.hand_count(p), 0);
    assert_eq!(life(&no_void, p), 20);

    g.move_object(witness, Zone::Graveyard);
    let mut wrong_turn = g.clone();
    wrong_turn.state.active = p.other();
    wrong_turn.begin_step(Step::End);
    quiet(&mut wrong_turn);
    assert_eq!(wrong_turn.hand_count(p), 0);

    g.begin_step(Step::End);
    g.prepare_triggers();
    g.move_object(titan, Zone::Exile);
    quiet(&mut g);
    assert_eq!(g.hand_count(p), 1);
    assert_eq!(life(&g, p), 19);
}

#[test]
fn vote_out_convoke_uses_black_and_generic_creatures_even_if_summoning_sick() {
    let p = Player::P0;
    let mut g = scenario(p);
    let spell = put(&mut g, p, "Vote Out", Zone::Hand);
    let victim = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let black = put(&mut g, p, "Hullcarver", Zone::Battlefield);
    let green = [
        put(&mut g, p, "Bear Cub", Zone::Battlefield),
        put(&mut g, p, "Bear Cub", Zone::Battlefield),
        put(&mut g, p, "Bear Cub", Zone::Battlefield),
    ];
    g.state.players[p.index()].mana = [0; 6];
    for card in [black, green[0], green[1], green[2]] {
        g.state.objects[card as usize].summoning_sick = true;
    }
    act(&mut g, Action::AnnounceCast { card: spell });
    act(&mut g, target(Target::Card(victim)));
    act(&mut g, Action::BeginConvoke);
    let object = g.object_ref(green[0]);
    reject(
        &mut g,
        Action::ChooseConvoker {
            object,
            contribution: convoke::ConvokeContribution::Black,
        },
    );
    let object = g.object_ref(black);
    act(
        &mut g,
        Action::ChooseConvoker {
            object,
            contribution: convoke::ConvokeContribution::Black,
        },
    );
    for card in green {
        let object = g.object_ref(card);
        act(
            &mut g,
            Action::ChooseConvoker {
                object,
                contribution: convoke::ConvokeContribution::Generic,
            },
        );
    }
    pay(&mut g);
    assert_eq!(g.state.stack.last().unwrap().cast.unwrap().payment, [0; 6]);
    settle(&mut g);
    assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
    assert!(
        [black, green[0], green[1], green[2]]
            .into_iter()
            .all(|id| g.object_view(id).tapped)
    );
}

#[test]
fn xu_ifit_returns_an_exact_skeleton_without_its_etb_or_keyword() {
    let p = Player::P0;
    let mut g = scenario(p);
    let xu = put(&mut g, p, "Xu-Ifit, Osteoharmonist", Zone::Battlefield);
    let whale = put(&mut g, p, "Starbreach Whale", Zone::Graveyard);
    let other = put(&mut g, p.other(), "Starbreach Whale", Zone::Graveyard);
    act(&mut g, activate(xu, 0));
    reject(&mut g, target(Target::Card(other)));
    act(&mut g, target(Target::Card(whale)));
    if matches!(g.state.pending, Pending::PayCosts { .. }) {
        pay(&mut g);
    }
    assert!(g.object_view(xu).tapped);
    g.move_object(xu, Zone::Exile);
    settle(&mut g);
    let view = g.object_view(whale);
    assert_eq!(view.zone, Zone::Battlefield);
    assert!(view.has_subtype("Whale") && view.has_subtype("Skeleton"));
    assert!(!view.printed_abilities_available());
    assert!(!g.keywords(whale).contains(Keywords::FLYING));
    assert!(g.observe(p).library_choice.is_none());

    g.move_object(whale, Zone::Exile);
    g.move_object(whale, Zone::Battlefield);
    assert!(!g.object_view(whale).has_subtype("Skeleton"));
    assert!(g.keywords(whale).contains(Keywords::FLYING));
    g.prepare_triggers();
    until(&mut g, |g| g.observe(p).library_choice.is_some());
}

#[test]
fn zero_point_ballad_threshold_and_life_loss_precede_the_exact_return_choice() {
    let p = Player::P0;
    let mut g = scenario(p);
    let spell = put(&mut g, p, "Zero Point Ballad", Zone::Hand);
    let boundary = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    g.add_counter(boundary, CounterKind::PlusOne, 3);
    let small = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let shielded = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    g.add_counter(shielded, CounterKind::Shield, 1);
    let unrelated = put(&mut g, p.other(), "Grizzly Bears", Zone::Graveyard);
    let mut five = g.clone();
    act(&mut five, Action::AnnounceCast { card: spell });
    act(&mut five, Action::ChooseX { value: 5 });
    pay(&mut five);
    settle(&mut five);
    assert_eq!(life(&five, p), 15);
    assert_eq!(five.object_view(boundary).zone, Zone::Battlefield);
    assert_eq!(five.object_view(small).zone, Zone::Graveyard);

    act(&mut g, Action::AnnounceCast { card: spell });
    act(&mut g, Action::ChooseX { value: 6 });
    pay(&mut g);
    until(&mut g, |g| {
        g.legal_actions().iter().any(
            |a| matches!(a, Action::SelectGraveyardReturn { object } if object.card == boundary),
        )
    });
    assert_eq!(life(&g, p), 14);
    assert_eq!(g.object_view(small).zone, Zone::Graveyard);
    assert_eq!(g.object_view(shielded).zone, Zone::Battlefield);
    assert_eq!(g.counter_count(shielded, CounterKind::Shield), 0);
    let object = g.object_ref(unrelated);
    reject(&mut g, Action::SelectGraveyardReturn { object });
    reject(&mut g, Action::FinishGraveyardReturn);
    let object = g.object_ref(boundary);
    act(&mut g, Action::SelectGraveyardReturn { object });
    quiet(&mut g);
    assert_eq!(g.object_view(boundary).controller, p);
    assert_eq!(g.object_view(boundary).owner, p.other());
    assert_eq!(stats(&g, boundary), (3, 3));
}

#[test]
fn bombard_deals_four_only_to_creatures_and_shield_prevents_the_damage() {
    let p = Player::P0;
    let mut g = scenario(p);
    let spell = put(&mut g, p, "Bombard", Zone::Hand);
    let creature = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let artifact = put(&mut g, p.other(), "Candy Trail", Zone::Battlefield);
    act(&mut g, Action::AnnounceCast { card: spell });
    reject(&mut g, target(Target::Card(artifact)));
    reject(&mut g, target(Target::Player(p.other())));
    act(&mut g, target(Target::Card(creature)));
    pay(&mut g);

    let mut shielded = g.clone();
    shielded.add_counter(creature, CounterKind::Shield, 1);
    quiet(&mut shielded);
    assert_eq!(shielded.object_view(creature).zone, Zone::Battlefield);
    assert_eq!(shielded.object_view(creature).damage, 0);
    assert_eq!(shielded.counter_count(creature, CounterKind::Shield), 0);
    quiet(&mut g);
    assert_eq!(g.object_view(creature).zone, Zone::Graveyard);
}

#[test]
fn cut_propulsion_uses_the_creatures_own_power_lifelink_and_live_flying() {
    let p = Player::P0;
    for (power, flying, damage) in [(0, true, 0), (2, false, 2), (2, true, 4)] {
        let mut g = scenario(p);
        let spell = put(&mut g, p, "Cut Propulsion", Zone::Hand);
        let turtle = put(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
        if power > 0 {
            g.add_counter(turtle, CounterKind::PlusOne, power);
        }
        cast(&mut g, spell, Some(Target::Card(turtle)));
        let object = g.object_ref(turtle);
        let keywords = if flying {
            Keywords::LIFELINK.union(Keywords::FLYING)
        } else {
            Keywords::LIFELINK
        };
        g.grant_keyword_effects(&[turtle], object, keywords);
        settle(&mut g);
        assert_eq!(g.object_view(turtle).damage, damage);
        assert_eq!(life(&g, p.other()), 20 + damage);
        assert_eq!(life(&g, p), 20);
    }
}

#[test]
fn debris_field_crusher_damages_as_an_artifact_then_unlocks_its_stationed_pump() {
    let p = Player::P0;
    let mut g = scenario(p);
    let ship = put(&mut g, p, "Debris Field Crusher", Zone::Hand);
    let donor = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
    cast(&mut g, ship, None);
    let enemy = target(Target::Player(p.other()));
    until(&mut g, |g| g.legal_actions().contains(&enemy));
    assert!(!g.is_creature(ship));
    act(&mut g, enemy);
    settle(&mut g);
    assert_eq!(life(&g, p.other()), 17);
    reject(&mut g, activate(ship, 1));

    g.add_counter(ship, CounterKind::Charge, 7);
    act(&mut g, activate(ship, 0));
    let object = g.object_ref(donor);
    act(&mut g, Action::ChooseAdditionalCost { object });
    pay(&mut g);
    settle(&mut g);
    assert_eq!(stats(&g, ship), (1, 5));
    assert!(g.keywords(ship).contains(Keywords::FLYING));

    g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
    act(&mut g, activate(ship, 1));
    pay(&mut g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    g.pay_counters(ship, &Counters::of(CounterKind::Charge, 1));
    assert!(!g.is_creature(ship));
    settle(&mut g);
    g.add_counter(ship, CounterKind::Charge, 1);
    assert_eq!(stats(&g, ship), (3, 5));
    let turn = g.state.turn;
    until(&mut g, |g| g.state.turn > turn);
    assert_eq!(stats(&g, ship), (1, 5));
}

fn onslaught_copies(g: &Game, player: Player) -> Vec<CardId> {
    g.observe(player)
        .objects
        .into_iter()
        .filter(|o| o.zone == Zone::Battlefield && o.controller == player && o.is_token())
        .map(|o| o.id)
        .collect()
}

#[test]
fn devastating_onslaught_pays_double_x_and_sacrifices_the_exact_hasty_copies() {
    let p = Player::P0;
    let mut g = scenario(p);
    let spell = put(&mut g, p, "Devastating Onslaught", Zone::Hand);
    let creature = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let artifact = put(&mut g, p, "Swiftfoot Boots", Zone::Battlefield);
    let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    g.add_counter(creature, CounterKind::PlusOne, 1);
    g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 4];
    act(&mut g, Action::AnnounceCast { card: spell });
    reject(&mut g, Action::ChooseX { value: 3 });
    let mut zero = g.clone();
    act(&mut zero, Action::ChooseX { value: 0 });
    act(&mut zero, target(Target::Card(creature)));
    pay(&mut zero);
    settle(&mut zero);
    assert!(onslaught_copies(&zero, p).is_empty());

    act(&mut g, Action::ChooseX { value: 2 });
    reject(&mut g, target(Target::Card(enemy)));
    let mut art = g.clone();
    act(&mut art, target(Target::Card(artifact)));
    pay(&mut art);
    quiet(&mut art);
    let copies = onslaught_copies(&art, p);
    assert_eq!(copies.len(), 2);
    for copy in copies {
        assert!(!art.is_creature(copy));
        assert!(art.object_view(copy).has_subtype("Equipment"));
    }

    act(&mut g, target(Target::Card(creature)));
    pay(&mut g);
    assert_eq!(g.state.stack.last().unwrap().cast.unwrap().mana_spent(), 5);
    quiet(&mut g);
    let copies = onslaught_copies(&g, p);
    assert_eq!(copies.len(), 2);
    until(&mut g, |g| g.state.pending == Pending::Attackers);
    for &copy in &copies {
        assert_eq!(stats(&g, copy), (3, 3));
        assert!(g.keywords(copy).contains(Keywords::HASTE));
        act(&mut g, Action::Attack { card: copy });
    }
    act(&mut g, Action::FinishAttackers);
    let turn = g.state.turn;
    until(&mut g, |g| g.state.turn > turn);
    assert_eq!(life(&g, p.other()), 14);
    assert!(onslaught_copies(&g, p).is_empty());
    assert_eq!(stats(&g, creature), (4, 4));
    assert_eq!(g.object_view(creature).zone, Zone::Battlefield);
}
