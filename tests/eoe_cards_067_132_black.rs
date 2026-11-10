//! Paid card scenarios for Edge of Eternities collectors 095–108.
mod support;

use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..64 {
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
                    Action::Pass | Action::PutTrigger { .. } | Action::FinishBlockers
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

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn select(g: &mut Game, card: CardId) {
    act(g, target(card));
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}

fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn attack(g: &mut Game, creatures: &[CardId]) {
    until(g, |g| g.state.pending == Pending::Attackers);
    for &card in creatures {
        act(g, Action::Attack { card });
    }
    act(g, Action::FinishAttackers);
}

fn life(g: &Game, player: Player) -> i128 {
    g.state.players[player.index()].life
}

fn robot(g: &Game, player: Player) -> CardId {
    let robot = g
        .observe(player)
        .objects
        .into_iter()
        .find(|o| o.zone == Zone::Battlefield && o.controller == player && o.is_token())
        .expect("Robot token");
    assert!(robot.has_subtype("Robot"));
    assert_eq!(robot.kinds & 17, 17);
    assert_eq!(stats(g, robot.id), (2, 2));
    robot.id
}

#[test]
fn depressurize_checks_the_power_after_applying_its_reduction() {
    let p = Player::P0;
    for (name, destroyed) in [("Hill Giant", true), ("Magnigoth Sentry", false)] {
        let mut g = scenario(p);
        let creature = put(&mut g, p.other(), name, Zone::Battlefield);
        let spell = put(&mut g, p, "Depressurize", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(creature)));
        settle(&mut g);
        if destroyed {
            assert_eq!(g.object_view(creature).zone, Zone::Graveyard);
        } else {
            assert_eq!(g.object_view(creature).zone, Zone::Battlefield);
            assert_eq!(stats(&g, creature), (1, 4));
        }
    }
}

#[test]
fn dubious_delicacy_flashes_and_pays_for_either_food_ability() {
    let p = Player::P0;
    for harm in [false, true] {
        let mut g = scenario(p.other());
        let food = put(&mut g, p, "Dubious Delicacy", Zone::Hand);
        let creature = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        act(&mut g, Action::Pass);
        cast(&mut g, food, None);
        until(&mut g, |g| g.legal_actions().contains(&target(creature)));
        if harm {
            select(&mut g, creature);
        } else {
            act(&mut g, Action::FinishTargets);
        }
        quiet(&mut g);
        assert!(g.object_view(food).has_subtype("Food"));
        assert_eq!(g.object_view(creature).zone == Zone::Graveyard, harm);
        if g.actor() != Some(p) {
            act(&mut g, Action::Pass);
        }
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        act(&mut g, activate(food, u16::from(harm)));
        if harm {
            let self_target = Action::ChooseTarget {
                target: Target::Player(p),
            };
            reject(&mut g, self_target);
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
        }
        pay(&mut g);
        assert_eq!(g.object_view(food).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert_eq!(life(&g, p), if harm { 20 } else { 23 });
        assert_eq!(life(&g, p.other()), if harm { 17 } else { 20 });
    }
}

#[test]
fn elegy_acolyte_draws_once_for_multiple_dealers_and_makes_an_untapped_robot_in_void() {
    let p = Player::P0;
    let mut g = scenario(p);
    let acolyte = put(&mut g, p, "Elegy Acolyte", Zone::Battlefield);
    let bear = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let mut clean = g.clone();
    clean.begin_step(Step::End);
    quiet(&mut clean);
    assert!(!clean.observe(p).objects.iter().any(|o| o.is_token()));
    attack(&mut g, &[acolyte, bear]);
    until(&mut g, |g| life(g, p.other()) == 14);
    quiet(&mut g);
    assert_eq!(g.hand_count(p), 1);
    assert_eq!(life(&g, p), 23);
    g.move_object(bear, Zone::Graveyard);
    let mut wrong_turn = g.clone();
    wrong_turn.state.active = p.other();
    wrong_turn.begin_step(Step::End);
    quiet(&mut wrong_turn);
    assert!(!wrong_turn.observe(p).objects.iter().any(|o| o.is_token()));
    g.begin_step(Step::End);
    quiet(&mut g);
    assert!(!g.object_view(robot(&g, p)).tapped);
}

#[test]
fn embrace_oblivion_can_pay_with_an_artifact_and_destroy_a_noncreature_spacecraft() {
    let p = Player::P0;
    let mut g = scenario(p);
    let spell = put(&mut g, p, "Embrace Oblivion", Zone::Hand);
    let cost = put(&mut g, p, "Candy Trail", Zone::Battlefield);
    let ship = put(&mut g, p.other(), "Warmaker Gunship", Zone::Battlefield);
    assert!(!g.is_creature(ship));
    act(&mut g, Action::AnnounceCast { card: spell });
    select(&mut g, ship);
    let other = g.object_ref(ship);
    reject(&mut g, Action::ChooseAdditionalCost { object: other });
    let object = g.object_ref(cost);
    act(&mut g, Action::ChooseAdditionalCost { object });
    pay(&mut g);
    assert_eq!(g.object_view(cost).zone, Zone::Graveyard);
    assert_eq!(g.object_view(ship).zone, Zone::Battlefield);
    settle(&mut g);
    assert_eq!(g.object_view(ship).zone, Zone::Graveyard);
}

#[test]
fn entropic_battlecruiser_separates_discard_loss_from_the_empty_hand_fallback() {
    let p = Player::P0;
    let mut g = scenario(p);
    let ship = put(&mut g, p, "Entropic Battlecruiser", Zone::Hand);
    cast(&mut g, ship, None);
    settle(&mut g);
    for expected in [20, 17] {
        let beetle = put(&mut g, p, "Virus Beetle", Zone::Hand);
        let discarded = put(&mut g, p.other(), "Island", Zone::Hand);
        cast(&mut g, beetle, None);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectDiscard { .. })
        });
        act(&mut g, Action::Discard { card: discarded });
        quiet(&mut g);
        assert_eq!(life(&g, p.other()), expected);
        assert!(!g.is_creature(ship));
        if expected == 20 {
            act(&mut g, activate(ship, 0));
            let object = g.object_ref(beetle);
            act(&mut g, Action::ChooseAdditionalCost { object });
            pay(&mut g);
            settle(&mut g);
        }
    }

    // Treat the ship as controlled since the start of the turn for the attack branch.
    g.state.objects[ship as usize].summoning_sick = false;
    g.add_counter(ship, CounterKind::Charge, 7);
    assert_eq!(stats(&g, ship), (3, 10));
    assert!(
        g.keywords(ship)
            .contains(Keywords::FLYING.union(Keywords::DEATHTOUCH))
    );
    attack(&mut g, &[ship]);
    let mut empty = g.clone();
    quiet(&mut empty);
    assert_eq!(life(&empty, p.other()), 14);

    let discarded = put(&mut g, p.other(), "Mountain", Zone::Hand);
    until(&mut g, |g| {
        matches!(g.state.pending, Pending::EffectDiscard { .. })
    });
    assert_eq!(g.actor(), Some(p.other()));
    assert!(g.observe(p).objects.iter().all(|o| o.id != discarded));
    assert_eq!(life(&g, p.other()), 17);
    act(&mut g, Action::Discard { card: discarded });
    quiet(&mut g);
    assert_eq!(life(&g, p.other()), 14);
}

#[test]
fn fallers_faithful_snapshots_whether_the_other_creature_was_dealt_damage() {
    let p = Player::P0;
    for (damage, protected, choose_target) in [
        (false, false, false),
        (false, false, true),
        (true, false, true),
        (false, true, true),
    ] {
        let mut g = scenario(p);
        let faithful = put(&mut g, p, "Faller's Faithful", Zone::Hand);
        let creature = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        if damage {
            let shock = put(&mut g, p, "Shock", Zone::Hand);
            cast(&mut g, shock, Some(Target::Card(creature)));
            settle(&mut g);
        }
        if protected {
            let reference = g.object_ref(creature);
            g.grant_keyword_effects(&[creature], reference, Keywords::INDESTRUCTIBLE);
        }
        cast(&mut g, faithful, None);
        until(&mut g, |g| g.legal_actions().contains(&target(creature)));
        reject(&mut g, target(faithful));
        if choose_target {
            select(&mut g, creature);
        } else {
            act(&mut g, Action::FinishTargets);
        }
        quiet(&mut g);
        assert_eq!(
            g.object_view(creature).zone == Zone::Graveyard,
            choose_target && !protected
        );
        assert_eq!(
            g.hand_count(p.other()),
            if choose_target && !damage { 2 } else { 0 }
        );
    }
}

#[test]
fn fell_gravship_mills_before_the_required_creature_or_spacecraft_return() {
    let p = Player::P0;
    let mut g = scenario(p);
    let ship = put(&mut g, p, "Fell Gravship", Zone::Hand);
    let foe = put(&mut g, p.other(), "Hill Giant", Zone::Graveyard);
    let creature = put(&mut g, p, "Grizzly Bears", Zone::Graveyard);
    let artifact = put(&mut g, p, "Candy Trail", Zone::Library);
    let spacecraft = put(&mut g, p, "Warmaker Gunship", Zone::Library);
    let land = put(&mut g, p, "Island", Zone::Library);
    cast(&mut g, ship, None);
    until(&mut g, |g| {
        g.legal_actions().iter().any(
            |a| matches!(a, Action::SelectGraveyardReturn { object } if object.card == spacecraft),
        )
    });
    let return_spacecraft = Action::SelectGraveyardReturn {
        object: g.object_ref(spacecraft),
    };
    for id in [artifact, spacecraft, land] {
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    }
    for id in [artifact, foe] {
        let object = g.object_ref(id);
        reject(&mut g, Action::SelectGraveyardReturn { object });
    }
    let mut alternate = g.clone();
    let object = alternate.object_ref(creature);
    act(&mut alternate, Action::SelectGraveyardReturn { object });
    quiet(&mut alternate);
    assert_eq!(alternate.object_view(creature).zone, Zone::Hand);
    act(&mut g, return_spacecraft);
    quiet(&mut g);
    assert_eq!(g.object_view(spacecraft).zone, Zone::Hand);
    let donor = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
    g.add_counter(ship, CounterKind::Charge, 7);
    assert!(!g.is_creature(ship));
    act(&mut g, activate(ship, 0));
    let object = g.object_ref(donor);
    act(&mut g, Action::ChooseAdditionalCost { object });
    pay(&mut g);
    settle(&mut g);
    assert_eq!(stats(&g, ship), (3, 2));
    assert!(
        g.keywords(ship)
            .contains(Keywords::FLYING.union(Keywords::LIFELINK))
    );
}

#[test]
fn gravblade_heavy_qualifies_once_for_controlling_one_or_more_artifacts() {
    let p = Player::P0;
    let mut g = scenario(p);
    let heavy = put(&mut g, p, "Gravblade Heavy", Zone::Hand);
    put(&mut g, p.other(), "Candy Trail", Zone::Battlefield);
    cast(&mut g, heavy, None);
    settle(&mut g);
    assert_eq!(stats(&g, heavy), (3, 4));
    assert!(!g.keywords(heavy).contains(Keywords::DEATHTOUCH));
    let a = put(&mut g, p, "Candy Trail", Zone::Battlefield);
    let b = put(&mut g, p, "Gilded Lotus", Zone::Battlefield);
    assert_eq!(stats(&g, heavy), (4, 4));
    assert!(g.keywords(heavy).contains(Keywords::DEATHTOUCH));
    g.move_object(a, Zone::Graveyard);
    assert_eq!(stats(&g, heavy), (4, 4));
    g.move_object(b, Zone::Graveyard);
    assert_eq!(stats(&g, heavy), (3, 4));
    assert!(!g.keywords(heavy).contains(Keywords::DEATHTOUCH));
}

#[test]
fn gravkill_exiles_a_creature_or_a_noncreature_spacecraft() {
    let p = Player::P0;
    for name in ["Hill Giant", "Warmaker Gunship"] {
        let mut g = scenario(p);
        let spell = put(&mut g, p, "Gravkill", Zone::Hand);
        let valid = put(&mut g, p.other(), name, Zone::Battlefield);
        let invalid = put(&mut g, p.other(), "Candy Trail", Zone::Battlefield);
        act(&mut g, Action::AnnounceCast { card: spell });
        reject(&mut g, target(invalid));
        select(&mut g, valid);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(valid).zone, Zone::Exile);
    }
}

#[test]
fn gravpack_monoist_is_a_flyer_and_only_death_makes_the_tapped_robot() {
    let p = Player::P0;
    let mut g = scenario(p);
    let monoist = put(&mut g, p, "Gravpack Monoist", Zone::Hand);
    cast(&mut g, monoist, None);
    settle(&mut g);
    assert!(g.keywords(monoist).contains(Keywords::FLYING));
    let mut exiled = g.clone();
    exiled.move_object(monoist, Zone::Exile);
    quiet(&mut exiled);
    assert!(!exiled.observe(p).objects.iter().any(|o| o.is_token()));
    let shock = put(&mut g, p, "Shock", Zone::Hand);
    cast(&mut g, shock, Some(Target::Card(monoist)));
    quiet(&mut g);
    assert_eq!(g.object_view(monoist).zone, Zone::Graveyard);
    assert!(g.object_view(robot(&g, p)).tapped);
}

#[test]
fn hullcarver_has_both_subtypes_and_trades_with_a_larger_blocker() {
    let p = Player::P0;
    let mut g = scenario(p);
    let hullcarver = put(&mut g, p, "Hullcarver", Zone::Battlefield);
    let giant = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let view = g.object_view(hullcarver);
    assert_eq!(view.kinds & 17, 17);
    assert!(view.has_subtype("Robot") && view.has_subtype("Assassin"));
    attack(&mut g, &[hullcarver]);
    until(&mut g, |g| g.state.pending == Pending::Blockers);
    act(
        &mut g,
        Action::Block {
            blocker: giant,
            attacker: hullcarver,
        },
    );
    act(&mut g, Action::FinishBlockers);
    until(&mut g, |g| {
        g.object_view(hullcarver).zone == Zone::Graveyard
    });
    assert_eq!(g.object_view(giant).zone, Zone::Graveyard);
}

#[test]
fn hylderblade_equip_cost_and_void_attach_use_the_same_bonus() {
    let p = Player::P0;
    let mut g = scenario(p);
    let blade = put(&mut g, p, "Hylderblade", Zone::Hand);
    let first = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let second = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let opponent = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    cast(&mut g, blade, None);
    settle(&mut g);
    let mut clean = g.clone();
    clean.begin_step(Step::End);
    quiet(&mut clean);
    assert!(clean.object_view(blade).attached_to.is_none());
    reject(&mut clean, activate(blade, 0));

    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
    act(&mut g, activate(blade, 0));
    reject(&mut g, target(opponent));
    select(&mut g, first);
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(stats(&g, first), (5, 3));
    g.move_object(opponent, Zone::Graveyard);
    g.begin_step(Step::End);
    until(&mut g, |g| g.legal_actions().contains(&target(second)));
    select(&mut g, second);
    settle(&mut g);
    assert_eq!(g.object_view(blade).attached_to, Some(g.object_ref(second)));
    assert_eq!(stats(&g, first), (2, 2));
    assert_eq!(stats(&g, second), (5, 3));
}

#[test]
fn hymn_of_the_faller_surveils_before_drawing_and_void_adds_one_card() {
    let p = Player::P0;
    for void in [false, true] {
        let mut g = scenario(p);
        let spell = put(&mut g, p, "Hymn of the Faller", Zone::Hand);
        let extra = put(&mut g, p, "Forest", Zone::Library);
        let first = put(&mut g, p, "Island", Zone::Library);
        let top = put(&mut g, p, "Mountain", Zone::Library);
        if void {
            let bear = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.move_object(bear, Zone::Graveyard);
        }
        cast(&mut g, spell, None);
        until(&mut g, |g| g.observe(p).library_choice.is_some());
        assert!(g.observe(p.other()).library_choice.is_none());
        assert_eq!(life(&g, p), 20);
        assert_eq!(g.hand_count(p), 0);
        act(
            &mut g,
            Action::ArrangeLibrary {
                card: top,
                destination: LibraryDestination::Graveyard,
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(top).zone, Zone::Graveyard);
        assert_eq!(g.object_view(first).zone, Zone::Hand);
        assert_eq!(
            g.object_view(extra).zone,
            if void { Zone::Hand } else { Zone::Library }
        );
        assert_eq!(life(&g, p), 19);
    }
}

#[test]
fn insatiable_skittermaw_has_menace_and_gains_an_own_end_step_void_counter() {
    let p = Player::P0;
    let mut g = scenario(p);
    let creature = put(&mut g, p, "Insatiable Skittermaw", Zone::Battlefield);
    let a = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let b = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    let mut clean = g.clone();
    clean.begin_step(Step::End);
    quiet(&mut clean);
    assert_eq!(stats(&clean, creature), (2, 2));

    attack(&mut g, &[creature]);
    until(&mut g, |g| g.state.pending == Pending::Blockers);
    act(
        &mut g,
        Action::Block {
            blocker: a,
            attacker: creature,
        },
    );
    reject(&mut g, Action::FinishBlockers);
    act(
        &mut g,
        Action::Block {
            blocker: b,
            attacker: creature,
        },
    );
    act(&mut g, Action::FinishBlockers);

    clean.move_object(a, Zone::Graveyard);
    clean.begin_step(Step::End);
    quiet(&mut clean);
    assert_eq!(stats(&clean, creature), (3, 3));
}
