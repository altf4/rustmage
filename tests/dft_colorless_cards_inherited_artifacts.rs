//! Complete printed-rule outcomes for the six inherited DFT artifacts.
mod support;
use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

fn idle(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.resolving.is_none()
}
fn drain(g: &mut Game) {
    for _ in 0..100 {
        if idle(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishTargets
            )
        });
    }
    panic!(
        "inherited artifact resolution ceiling: {:?}",
        g.state.pending
    );
}
fn ready(g: &mut Game, predicate: impl Fn(&Action) -> bool) {
    for _ in 0..80 {
        if g.legal_actions().iter().any(&predicate) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("inherited artifact decision ceiling: {:?}", g.state.pending);
}
fn announcement(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}
fn finish(g: &mut Game) {
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
fn activate(g: &mut Game, card: CardId, ability: u16, target: Option<CardId>) {
    act(g, announcement(card, ability));
    if let Some(card) = target {
        act(
            g,
            Action::ChooseTarget {
                target: Target::Card(card),
            },
        );
    }
    finish(g);
    if matches!(g.state.pending, Pending::PayCosts { .. }) {
        pay(g);
    }
}
fn tokens(g: &Game, p: Player, subtype: &str) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            let id = i as CardId;
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.has_subtype(id, subtype))
            .then_some(id)
        })
        .collect()
}
fn start_attack(g: &mut Game, card: CardId) {
    g.state.objects[card as usize].summoning_sick = false;
    g.state.objects[card as usize].tapped = false;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    g.state.priority = g.state.active;
    g.state.passes = 0;
    act(g, Action::Attack { card });
    act(g, Action::FinishAttackers);
}
fn combat_until(g: &mut Game, done: impl Fn(&Game) -> bool) {
    for _ in 0..100 {
        if done(g) {
            return;
        }
        if let Some(range) = g.damage_assignment_range() {
            act(
                g,
                Action::AssignDamage {
                    target: range.target,
                    amount: range.minimum,
                },
            );
        } else {
            choose(g, |a| {
                matches!(
                    a,
                    Action::Pass | Action::FinishBlockers | Action::PutTrigger { .. }
                )
            });
        }
    }
    panic!("inherited artifact combat ceiling: {:?}", g.state.pending);
}
fn paid_spark(p: Player) -> (Game, CardId) {
    let mut g = scenario(p);
    let source = put(&mut g, p, "The Aetherspark", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
    cast(&mut g, source, None);
    drain(&mut g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(g.counter_count(source, CounterKind::Loyalty), 4);
    assert!(g.object_view(source).card.is_legendary());
    assert!(g.has_subtype(source, "Equipment"));
    assert_eq!(g.current_kinds(source), 80);
    (g, source)
}

#[test]
fn camera_launcher_pays_three_exhausts_once_across_turns_and_blink_resets_the_exact_object() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Camera Launcher", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        cast(&mut g, source, None);
        drain(&mut g);
        assert_eq!(stats(&g, source), (2, 2));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        g.state.players[p.index()].mana[5] = 2;
        reject(&mut g, announcement(source, 0));
        g.state.players[p.index()].mana[5] = 3;
        activate(&mut g, source, 0, None);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
        assert!(!g.object_view(source).tapped);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        drain(&mut g);
        assert_eq!(stats(&g, source), (3, 3));
        let thopters = tokens(&g, p, "Thopter");
        assert_eq!(thopters.len(), 1);
        let token = thopters[0];
        assert_eq!(stats(&g, token), (1, 1));
        assert_eq!(g.current_kinds(token), 17);
        assert_eq!(g.object_view(token).colors, 0);
        assert!(g.keywords(token).contains(Keywords::FLYING));
        g.state.players[p.index()].mana[5] = 3;
        reject(&mut g, announcement(source, 0));
        g.finish_cleanup();
        g.state.priority = p;
        g.state.players[p.index()].mana[5] = 3;
        reject(&mut g, announcement(source, 0));
        let old = g.object_ref(source);
        g.move_object(source, Zone::Exile);
        g.move_object(source, Zone::Battlefield);
        assert_ne!(g.object_ref(source), old);
        assert_eq!(stats(&g, source), (2, 2));
        activate(&mut g, source, 0, None);
        drain(&mut g);
        assert_eq!(stats(&g, source), (3, 3));
        assert_eq!(tokens(&g, p, "Thopter").len(), 2);
    }
}

#[test]
fn camera_launcher_owned_paid_branch_keeps_token_effect_but_never_counters_a_new_incarnation() {
    let p = Player::P0;
    let mut parent = scenario(p);
    let source = put(&mut parent, p, "Camera Launcher", Zone::Battlefield);
    parent.state.players[0].mana = [0, 0, 0, 0, 0, 3];
    act(&mut parent, announcement(source, 0));
    let saved = parent.snapshot();
    let payment = parent
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == payment)
        .unwrap();
    let actor = indexed.actor().unwrap();
    let revision = indexed.revision();
    act(&mut raw, payment);
    indexed.apply_index(actor, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    drop(parent);
    drain(&mut raw);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(stats(&raw, source), (3, 3));
    indexed.restore(&saved).unwrap();
    pay(&mut indexed);
    indexed.move_object(source, Zone::Exile);
    indexed.move_object(source, Zone::Battlefield);
    drain(&mut indexed);
    assert_eq!(stats(&indexed, source), (2, 2));
    assert_eq!(tokens(&indexed, p, "Thopter").len(), 1);
    indexed.state.players[0].mana[5] = 3;
    activate(&mut indexed, source, 0, None);
    drain(&mut indexed);
    assert_eq!(stats(&indexed, source), (3, 3));
    assert_eq!(tokens(&indexed, p, "Thopter").len(), 2);
}

#[test]
fn monument_real_cycling_uses_each_mode_once_and_resets_on_the_opponents_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let monument = put(&mut g, p, "Monument to Endurance", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        cast(&mut g, monument, None);
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        for mode in 0..4 {
            let ferry = put(&mut g, p, "Skybox Ferry", Zone::Hand);
            g.state.players[p.index()].mana[5] = 2;
            activate(&mut g, ferry, 1, None);
            assert_eq!(g.object_view(ferry).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            if mode < 3 {
                ready(&mut g, |a| matches!(a, Action::ChooseMode { .. }));
                for used in 0..mode {
                    reject(&mut g, Action::ChooseMode { mode: used });
                }
                act(&mut g, Action::ChooseMode { mode });
            }
            drain(&mut g);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                mode as u64 + 2
            );
        }
        assert_eq!(g.state.players[p.other().index()].life, 17);
        let treasure = tokens(&g, p, "Treasure");
        assert_eq!(treasure.len(), 1);
        assert_eq!(g.current_kinds(treasure[0]), 16);
        assert_eq!(g.object_view(treasure[0]).colors, 0);
        act(
            &mut g,
            Action::ActivateMana {
                card: treasure[0],
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 1, 0, 0, 0, 0],
            },
        );
        assert_ne!(g.object_view(treasure[0]).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, [0, 1, 0, 0, 0, 0]);
        g.finish_cleanup();
        assert_eq!(g.state.active, p.other());
        let ferry = put(&mut g, p, "Skybox Ferry", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        if g.actor() != Some(p) {
            act(&mut g, Action::Pass);
        }
        activate(&mut g, ferry, 1, None);
        ready(&mut g, |a| *a == Action::ChooseMode { mode: 0 });
        act(&mut g, Action::ChooseMode { mode: 0 });
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        assert_eq!(g.state.players[p.other().index()].life, 17);
    }
}

#[test]
fn monument_two_discarded_cards_generate_two_triggers_and_two_objects_have_independent_modes() {
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Monument to Endurance", Zone::Battlefield);
    let first = put(&mut g, p, "Forest", Zone::Hand);
    let second = put(&mut g, p, "Forest", Zone::Hand);
    let kiora = put(&mut g, p, "Kiora, the Rising Tide", Zone::Hand);
    cast(&mut g, kiora, None);
    ready(&mut g, |a| *a == Action::Discard { card: first });
    act(&mut g, Action::Discard { card: first });
    act(&mut g, Action::Discard { card: second });
    ready(&mut g, |a| *a == Action::ChooseMode { mode: 0 });
    act(&mut g, Action::ChooseMode { mode: 0 });
    ready(&mut g, |a| *a == Action::ChooseMode { mode: 1 });
    reject(&mut g, Action::ChooseMode { mode: 0 });
    act(&mut g, Action::ChooseMode { mode: 1 });
    drain(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 3);
    assert_eq!(tokens(&g, p, "Treasure").len(), 1);
    let mut two = scenario(p);
    for _ in 0..2 {
        put(&mut two, p, "Monument to Endurance", Zone::Battlefield);
    }
    let ferry = put(&mut two, p, "Skybox Ferry", Zone::Hand);
    activate(&mut two, ferry, 1, None);
    for _ in 0..2 {
        ready(&mut two, |a| *a == Action::ChooseMode { mode: 0 });
        act(&mut two, Action::ChooseMode { mode: 0 });
    }
    drain(&mut two);
    assert_eq!(two.state.players[0].cards_drawn_this_turn, 3);
}

#[test]
fn scrap_compactor_first_mode_requires_creature_and_pays_three_tap_and_sacrifice_before_damage() {
    for stale in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Scrap Compactor", Zone::Hand);
        g.state.players[0].mana = [0, 0, 0, 0, 0, 1];
        cast(&mut g, source, None);
        drain(&mut g);
        assert_eq!(g.state.players[0].mana, [0; 6]);
        let target = put(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
        let vehicle = put(&mut g, p.other(), "Skybox Ferry", Zone::Battlefield);
        g.state.players[0].mana[5] = 2;
        reject(&mut g, announcement(source, 0));
        g.state.players[0].mana[5] = 3;
        g.state.objects[source as usize].tapped = true;
        reject(&mut g, announcement(source, 0));
        g.state.objects[source as usize].tapped = false;
        act(&mut g, announcement(source, 0));
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(vehicle),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        finish(&mut g);
        pay(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(g.state.players[0].mana, [0; 6]);
        assert_eq!(g.object_view(target).damage, 0);
        if stale {
            g.move_object(target, Zone::Exile);
            g.move_object(target, Zone::Battlefield);
        }
        drain(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        assert_eq!(g.object_view(target).damage, if stale { 0 } else { 3 });
        assert_eq!(g.object_view(vehicle).zone, Zone::Battlefield);
    }
}

#[test]
fn scrap_compactor_second_mode_pays_six_to_destroy_an_uncrewed_vehicle_or_creature() {
    for (name, indestructible) in [
        ("Skybox Ferry", false),
        ("Bear Cub", false),
        ("Bear Cub", true),
    ] {
        let mut g = scenario(Player::P1);
        let source = put(&mut g, Player::P1, "Scrap Compactor", Zone::Battlefield);
        let target = put(&mut g, Player::P0, name, Zone::Battlefield);
        if indestructible {
            g.state.objects[target as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
        }
        let artifact = put(
            &mut g,
            Player::P0,
            "Monument to Endurance",
            Zone::Battlefield,
        );
        g.state.players[1].mana = [0, 0, 0, 0, 0, 5];
        reject(&mut g, announcement(source, 1));
        g.state.players[1].mana[5] = 6;
        act(&mut g, announcement(source, 1));
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(artifact),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        finish(&mut g);
        pay(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(g.state.players[1].mana, [0; 6]);
        drain(&mut g);
        assert_eq!(
            g.object_view(target).zone,
            if indestructible {
                Zone::Battlefield
            } else {
                Zone::Graveyard
            }
        );
    }
}

#[test]
fn skybox_ferry_is_a_flying_four_four_only_after_crew_two_and_animation_expires() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Skybox Ferry", Zone::Hand);
    g.state.players[0].mana = [0, 0, 0, 0, 0, 5];
    cast(&mut g, source, None);
    drain(&mut g);
    assert!(!g.is_creature(source));
    assert!(g.has_subtype(source, "Vehicle"));
    assert!(g.keywords(source).contains(Keywords::FLYING));
    assert_eq!(g.state.players[0].mana, [0; 6]);
    put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
    reject(&mut g, announcement(source, 0));
    let pilot = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
    g.state.objects[pilot as usize].summoning_sick = true;
    act(&mut g, announcement(source, 0));
    if matches!(g.state.pending, Pending::PayCosts { .. }) {
        pay(&mut g);
    }
    let object = g.object_ref(pilot);
    act(&mut g, Action::ChooseCrewer { object });
    act(&mut g, Action::FinishCrew);
    assert!(g.object_view(pilot).tapped);
    assert!(!g.is_creature(source));
    drain(&mut g);
    assert_eq!(stats(&g, source), (4, 4));
    assert!(g.is_creature(source));
    assert!(g.keywords(source).contains(Keywords::FLYING));
    assert!(!g.can_declare_attacker(p, source));
    g.finish_cleanup();
    assert!(!g.is_creature(source));
    assert!(g.keywords(source).contains(Keywords::FLYING));
}

#[test]
fn both_inherited_cyclers_pay_two_discard_at_commitment_and_draw_without_battlefield_triggers() {
    for p in [Player::P0, Player::P1] {
        for (name, ability) in [("Skybox Ferry", 1), ("Wreck Remover", 0)] {
            let mut g = scenario(p);
            let source = put(&mut g, p, name, Zone::Hand);
            let grave = put(&mut g, p.other(), "Forest", Zone::Graveyard);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
            reject(&mut g, announcement(source, ability));
            g.state.players[p.index()].mana[5] = 2;
            activate(&mut g, source, ability, None);
            assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            drain(&mut g);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
            assert_eq!(g.state.players[p.index()].life, 20);
            assert_eq!(g.object_view(grave).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn aetherspark_plus_one_has_optional_own_target_and_one_sorcery_speed_loyalty_use_per_turn() {
    for p in [Player::P0, Player::P1] {
        for targeted in [false, true] {
            let (mut g, source) = paid_spark(p);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.step = Step::BeginCombat;
            reject(&mut g, announcement(source, 0));
            g.state.step = Step::Main1;
            act(&mut g, announcement(source, 0));
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            if targeted {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(own),
                    },
                );
            }
            finish(&mut g);
            pay(&mut g);
            assert_eq!(g.counter_count(source, CounterKind::Loyalty), 5);
            assert_eq!(g.counter_count(own, CounterKind::PlusOne), 0);
            drain(&mut g);
            assert_eq!(
                g.counter_count(own, CounterKind::PlusOne),
                u64::from(targeted)
            );
            assert_eq!(
                g.object_view(source).attached_to,
                targeted.then(|| g.object_ref(own))
            );
            reject(&mut g, announcement(source, 0));
            reject(&mut g, announcement(source, 1));
        }
    }
}

#[test]
fn aetherspark_attachment_blocks_attacks_until_the_host_leaves_and_own_combat_adds_exact_loyalty() {
    for p in [Player::P0, Player::P1] {
        let (mut g, source) = paid_spark(p);
        let host = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        activate(&mut g, source, 0, Some(host));
        drain(&mut g);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.active = p.other();
        g.state.priority = p.other();
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        let attack = Action::AttackPlaneswalker {
            card: enemy,
            defender: g.object_ref(source),
        };
        reject(&mut g, attack.clone());
        let saved = g.snapshot();
        g.move_object(host, Zone::Graveyard);
        assert!(g.legal_actions().contains(&attack));
        g.restore(&saved).unwrap();
        g.state.active = p;
        start_attack(&mut g, host);
        combat_until(&mut g, |g| {
            g.state
                .pending_triggers
                .iter()
                .chain(g.state.stack.iter())
                .any(|i| i.creature_combat_event.is_some())
        });
        let packet = g
            .state
            .pending_triggers
            .iter()
            .chain(g.state.stack.iter())
            .find(|i| i.creature_combat_event.is_some())
            .unwrap();
        assert_eq!(packet.creature_combat_event.as_ref().unwrap().amount, 3);
        let saved = g.snapshot();
        let mut owned = g.clone();
        drop(g);
        drain(&mut owned);
        assert_eq!(owned.counter_count(source, CounterKind::Loyalty), 8);
        assert_eq!(owned.state.players[p.other().index()].life, 17);
        owned.restore(&saved).unwrap();
        drain(&mut owned);
        assert_eq!(owned.counter_count(source, CounterKind::Loyalty), 8);
    }
}

#[test]
fn aetherspark_equipped_creature_combat_on_opponents_turn_does_not_add_loyalty() {
    let p = Player::P0;
    let (mut g, source) = paid_spark(p);
    let host = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    activate(&mut g, source, 0, Some(host));
    drain(&mut g);
    let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    g.state.active = p.other();
    start_attack(&mut g, enemy);
    ready(&mut g, |a| {
        *a == Action::Block {
            blocker: host,
            attacker: enemy,
        }
    });
    act(
        &mut g,
        Action::Block {
            blocker: host,
            attacker: enemy,
        },
    );
    act(&mut g, Action::FinishBlockers);
    combat_until(&mut g, |g| {
        g.object_view(enemy).zone == Zone::Graveyard && idle(g)
    });
    assert_eq!(g.counter_count(source, CounterKind::Loyalty), 5);
    assert_eq!(g.object_view(host).zone, Zone::Battlefield);
    assert_eq!(g.object_view(host).damage, 2);
}

#[test]
fn aetherspark_minus_five_draws_two_after_its_last_loyalty_is_paid() {
    for p in [Player::P0, Player::P1] {
        let (mut g, source) = paid_spark(p);
        reject(&mut g, announcement(source, 1));
        g.add_counter(source, CounterKind::Loyalty, 1);
        activate(&mut g, source, 1, None);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
    }
}

#[test]
fn aetherspark_minus_ten_uses_the_stack_then_makes_ten_mana_of_exactly_one_color() {
    for p in [Player::P0, Player::P1] {
        let (mut g, source) = paid_spark(p);
        g.add_counter(source, CounterKind::Loyalty, 5);
        reject(&mut g, announcement(source, 2));
        g.add_counter(source, CounterKind::Loyalty, 1);
        activate(&mut g, source, 2, None);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(g.state.pending, Pending::Priority);
        assert_eq!(g.state.stack.len(), 1);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        ready(&mut g, |a| matches!(a, Action::ChooseMana { .. }));
        reject(
            &mut g,
            Action::ChooseMana {
                production: [5, 5, 0, 0, 0, 0],
            },
        );
        reject(
            &mut g,
            Action::ChooseMana {
                production: [0, 0, 0, 0, 0, 10],
            },
        );
        let saved = g.snapshot();
        let action = Action::ChooseMana {
            production: [0, 0, 0, 10, 0, 0],
        };
        let mut indexed = g.clone();
        let index = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        let actor = indexed.actor().unwrap();
        let revision = indexed.revision();
        act(&mut g, action);
        indexed.apply_index(actor, revision, index).unwrap();
        drain(&mut g);
        drain(&mut indexed);
        assert_eq!(g.snapshot(), indexed.snapshot());
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 10, 0, 0]);
        indexed.restore(&saved).unwrap();
        act(
            &mut indexed,
            Action::ChooseMana {
                production: [0, 10, 0, 0, 0, 0],
            },
        );
        drain(&mut indexed);
        assert_eq!(indexed.state.players[p.index()].mana, [0, 10, 0, 0, 0, 0]);
    }
}

#[test]
fn wreck_remover_paid_entry_exiles_from_either_graveyard_and_its_zero_target_attack_still_gains_life()
 {
    for p in [Player::P0, Player::P1] {
        for owner in [p, p.other()] {
            let mut g = scenario(p);
            let target = put(&mut g, owner, "Forest", Zone::Graveyard);
            let illegal = put(&mut g, owner, "Bear Cub", Zone::Battlefield);
            let source = put(&mut g, p, "Wreck Remover", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
            cast(&mut g, source, None);
            ready(&mut g, |a| {
                *a == Action::ChooseTarget {
                    target: Target::Card(target),
                }
            });
            assert_eq!(stats(&g, source), (3, 4));
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(illegal),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            finish(&mut g);
            drain(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Exile);
            assert_eq!(g.state.players[p.index()].life, 21);
            let remaining = put(&mut g, p.other(), "Mountain", Zone::Graveyard);
            start_attack(&mut g, source);
            ready(&mut g, |a| *a == Action::FinishTargets);
            act(&mut g, Action::FinishTargets);
            drain(&mut g);
            assert_eq!(g.state.players[p.index()].life, 22);
            assert_eq!(g.object_view(remaining).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn wreck_remover_losing_its_only_target_fizzles_life_gain_and_never_rebinds_a_graveyard_reentry() {
    let p = Player::P0;
    let mut g = scenario(p);
    let target = put(&mut g, p.other(), "Forest", Zone::Graveyard);
    let source = put(&mut g, p, "Wreck Remover", Zone::Hand);
    cast(&mut g, source, None);
    ready(&mut g, |a| {
        *a == Action::ChooseTarget {
            target: Target::Card(target),
        }
    });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    finish(&mut g);
    let original = g.object_ref(target);
    g.move_object(target, Zone::Exile);
    g.move_object(target, Zone::Graveyard);
    assert_ne!(g.object_ref(target), original);
    drain(&mut g);
    assert_eq!(g.object_view(target).zone, Zone::Graveyard);
    assert_eq!(g.state.players[0].life, 20);
}

#[test]
fn monument_new_incarnation_can_choose_the_same_mode_again_in_the_same_turn() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Monument to Endurance", Zone::Hand);
    g.state.players[0].mana = [0, 0, 0, 0, 0, 3];
    cast(&mut g, source, None);
    drain(&mut g);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    let original = g.object_ref(source);
    for iteration in 0..2 {
        let ferry = put(&mut g, p, "Skybox Ferry", Zone::Hand);
        g.state.players[0].mana[5] = 2;
        activate(&mut g, ferry, 1, None);
        ready(&mut g, |a| *a == Action::ChooseMode { mode: 2 });
        act(&mut g, Action::ChooseMode { mode: 2 });
        drain(&mut g);
        assert_eq!(g.state.players[1].life, 17 - 3 * iteration);
        if iteration == 0 {
            g.move_object(source, Zone::Exile);
            g.move_object(source, Zone::Battlefield);
            assert_ne!(g.object_ref(source), original);
        }
    }
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 2);
}
