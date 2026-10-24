//! Paid whole-card tests for scaled creature damage and activation-only graveyard guards.
use super::ability_world::CharacteristicWorld;
use super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::*;
fn announce(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn finish(g: &mut Game) {
    for _ in 0..12 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::FinishTargets
                        | Action::ChooseActivationCost { .. }
                        | Action::PayCosts { .. }
                )
            })
            .unwrap();
        checked(g, a);
    }
    panic!("unfinished payment");
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, targets);
    settle(g);
    id
}
fn punch(g: &mut Game, p: Player, source: CardId, target: CardId) -> CardId {
    let id = named(g, p, "Punishing Punch", Zone::Hand);
    cast(g, id, None, &[Target::Card(source), Target::Card(target)]);
    id
}
fn corpses(g: &mut Game, p: Player) -> [CardId; 2] {
    [
        named(g, p, "Bear Cub", Zone::Graveyard),
        named(g, p, "Hill Giant", Zone::Graveyard),
    ]
}
fn villains(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && o.card == Card::from_name("@token/cf0227c0984ee9114e6c311f").unwrap())
            .then_some(id as CardId)
        })
        .collect()
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..100 {
        if stop(g) {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                )
            })
            .unwrap_or_else(|| panic!("stalled {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("stalled {:?}", g.state.pending);
}
#[test]
fn msh_punch_zola_punch_exact_paid_full_discount_threshold_and_private_mana_request() {
    for p in [Player::P0, Player::P1] {
        for count in 0..=2 {
            let mut g = setup(p);
            let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
            for _ in 0..count {
                named(&mut g, p, "Hill Giant", Zone::Graveyard);
            }
            named(&mut g, p, "Shock", Zone::Graveyard);
            corpses(&mut g, p.other());
            let secret = named(&mut g, p, "Shock", Zone::Hand);
            let s = named(&mut g, p, "Punishing Punch", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 1, if count == 2 { 0 } else { 2 }];
            checked(&mut g, Action::AnnounceCast { card: s });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            while !matches!(g.state.pending, Pending::PayCosts { .. }) {
                checked(&mut g, Action::FinishTargets);
            }
            let request = g.announcement_payment_request(p).unwrap();
            assert_eq!(request.required, [0, 0, 0, 0, 1, 0]);
            assert_eq!(request.generic, if count == 2 { 0 } else { 2 });
            assert!(g.announcement_payment_request(p.other()).is_none());
            assert!(g.observe(p.other()).objects.iter().all(|o| o.id != secret));
            reject(&mut g, Action::PayCosts { payment: [0; 6] });
            finish(&mut g);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.object_view(enemy).damage, 4);
            assert_eq!(g.object_view(own).damage, 0);
            assert_eq!(g.object_view(s).zone, Zone::Graveyard);
        }
    }
}
#[test]
fn msh_punch_zola_punch_target_routes_reject_wrong_relations_players_and_land_atomically() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        let s = named(&mut g, p, "Punishing Punch", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: s });
        for target in [
            Target::Card(enemy),
            Target::Card(land),
            Target::Player(p.other()),
        ] {
            reject(&mut g, Action::ChooseTarget { target });
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        for target in [
            Target::Card(own),
            Target::Card(land),
            Target::Player(p.other()),
        ] {
            reject(&mut g, Action::ChooseTarget { target });
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(enemy).damage, 4);
    }
}
#[test]
fn msh_punch_zola_punch_partial_illegality_blink_departure_control_and_hexproof_no_damage() {
    for p in [Player::P0, Player::P1] {
        for which in 0..2 {
            for mode in 0..4 {
                let mut g = setup(p);
                let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                let enemy = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
                punch(&mut g, p, own, enemy);
                let id = [own, enemy][which];
                match mode {
                    0 => {
                        g.move_object(id, Zone::Hand);
                    }
                    1 => {
                        g.move_object(id, Zone::Hand);
                        g.move_object(id, Zone::Battlefield);
                    }
                    2 => g.change_controller(
                        g.object_ref(id),
                        if which == 0 { p.other() } else { p },
                    ),
                    _ => {
                        if which == 0 {
                            // Opponent-controlled source is no longer a legal first target.
                            g.change_controller(g.object_ref(id), p.other());
                        } else {
                            g.state.objects[id as usize].gained_keywords = Keywords::HEXPROOF;
                        }
                    }
                }
                settle(&mut g);
                assert_eq!(g.object_view(enemy).damage, 0, "{which} {mode}");
                assert_eq!(g.object_view(own).damage, 0);
            }
        }
    }
}
#[test]
fn msh_punch_zola_punch_uses_current_power_after_paid_response_and_not_spell_power() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Darksteel Colossus", Zone::Battlefield);
        punch(&mut g, p, own, enemy);
        // Paid instant response resolves first, raising the source's power from two to five.
        paid(&mut g, p, "Giant Growth", &[Target::Card(own)]);
        settle(&mut g);
        assert_eq!(g.stats(own).power, 5);
        assert_eq!(g.object_view(enemy).damage, 10);
        assert_eq!(g.object_view(own).damage, 0);
    }
}
#[test]
fn msh_punch_zola_punch_damage_source_lifelink_deathtouch_and_negative_power() {
    for p in [Player::P0, Player::P1] {
        for power in [-2, 0, 1] {
            let mut g = setup(p);
            let own = named(&mut g, p, "Aegis Turtle", Zone::Battlefield);
            g.state.objects[own as usize].growth.power = power;
            g.state.objects[own as usize].gained_keywords =
                Keywords::LIFELINK.union(Keywords::DEATHTOUCH);
            let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            punch(&mut g, p, own, enemy);
            settle(&mut g);
            assert_eq!(g.life(p), 20 + 2 * power.max(0));
            assert_eq!(
                g.object_view(enemy).zone,
                if power > 0 {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(g.object_view(own).damage, 0);
        }
    }
}
#[test]
fn msh_punch_zola_punch_respects_prevention() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let king = named(&mut g, p.other(), "King T'Challa", Zone::Hand);
        g.state.priority = p.other();
        checked(&mut g, Action::AnnounceBack { card: king });
        finish(&mut g);
        settle(&mut g);
        g.state.priority = p;
        punch(&mut g, p, own, king);
        settle(&mut g);
        assert_eq!(g.object_view(king).damage, 0);
        assert_eq!(g.object_view(king).zone, Zone::Battlefield);
    }
}
#[test]
fn msh_punch_zola_zola_paid_body_and_graveyard_start_restriction_excludes_opponents_tokens_noncreatures()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 2];
        let source = paid(&mut g, p, "Arnim Zola, Bio-Fanatic", &[]);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let v = g.object_view(source);
        assert_eq!((v.power, v.toughness, v.kinds, v.colors), (2, 3, 17, 4));
        assert!(g.has_subtype(source, "Scientist") && g.has_subtype(source, "Villain"));
        assert!(v.card.is_legendary());
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        corpses(&mut g, p.other());
        named(&mut g, p, "Shock", Zone::Graveyard);
        let token = named(
            &mut g,
            p,
            "@token/cf0227c0984ee9114e6c311f",
            Zone::Graveyard,
        );
        assert!(g.state.objects[token as usize].is_token());
        let one = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        g.state.objects[source as usize].summoning_sick = false;
        reject(&mut g, announce(source));
        let two = named(&mut g, p, "Hill Giant", Zone::Graveyard);
        // Ownership, not a stale controller, identifies a card in this player's graveyard.
        g.state.objects[one as usize].controller = p.other();
        g.state.objects[two as usize].controller = p.other();
        g.state.objects[source as usize].summoning_sick = true;
        reject(&mut g, announce(source));
        g.state.objects[source as usize].summoning_sick = false;
        checked(&mut g, announce(source));
        reject(&mut g, Action::PayCosts { payment: [0; 6] });
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(g.object_view(source).tapped);
        let made = villains(&g, p);
        assert_eq!(made.len(), 1);
        let v = g.object_view(made[0]);
        assert_eq!((v.power, v.toughness, v.colors, v.kinds), (2, 1, 4, 1));
        assert!(v.tapped && v.keywords.contains(Keywords::MENACE));
        assert!(g.has_subtype(made[0], "Villain"));
        reject(&mut g, announce(source));
    }
}
#[test]
fn msh_punch_zola_zola_activation_restriction_not_rechecked_after_graveyard_or_source_changes() {
    for p in [Player::P0, Player::P1] {
        for change in 0..3 {
            let mut g = setup(p);
            let source = paid(&mut g, p, "Arnim Zola, Bio-Fanatic", &[]);
            g.state.objects[source as usize].summoning_sick = false;
            let dead = corpses(&mut g, p);
            checked(&mut g, announce(source));
            finish(&mut g);
            for id in dead {
                g.move_object(id, Zone::Exile);
            }
            match change {
                0 => g.change_controller(g.object_ref(source), p.other()),
                1 => {
                    g.move_object(source, Zone::Graveyard);
                }
                _ => {
                    g.move_object(source, Zone::Hand);
                    g.move_object(source, Zone::Battlefield);
                }
            }
            settle(&mut g);
            assert_eq!(villains(&g, p).len(), 1);
            assert!(villains(&g, p.other()).is_empty());
            assert!(g.object_view(villains(&g, p)[0]).tapped);
        }
    }
}
#[test]
fn msh_punch_zola_zola_current_controller_restriction_and_token_replacement_owner() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.active = p.other();
        g.state.priority = p.other();
        let source = paid(&mut g, p.other(), "Arnim Zola, Bio-Fanatic", &[]);
        g.state.active = p;
        g.change_controller(g.object_ref(source), p);
        g.state.priority = p;
        g.state.objects[source as usize].summoning_sick = false;
        corpses(&mut g, p.other());
        reject(&mut g, announce(source));
        corpses(&mut g, p);
        paid(&mut g, p, "Doubling Season", &[]);
        checked(&mut g, announce(source));
        finish(&mut g);
        settle(&mut g);
        assert_eq!(villains(&g, p).len(), 2);
        assert!(villains(&g, p.other()).is_empty());
        assert!(villains(&g, p).iter().all(|&id| g.object_view(id).tapped));
    }
}
#[test]
fn msh_punch_zola_zola_token_menace_requires_two_blockers() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, "Arnim Zola, Bio-Fanatic", &[]);
        g.state.objects[source as usize].summoning_sick = false;
        corpses(&mut g, p);
        checked(&mut g, announce(source));
        finish(&mut g);
        settle(&mut g);
        let token = villains(&g, p)[0];
        g.state.objects[token as usize].tapped = false;
        g.state.objects[token as usize].summoning_sick = false;
        let a = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
        let b = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
        g.state.active = p;
        g.state.priority = p;
        g.state.pending = Pending::Priority;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: token });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(
            &mut g,
            Action::Block {
                blocker: a,
                attacker: token,
            },
        );
        reject(&mut g, Action::FinishBlockers);
        checked(
            &mut g,
            Action::Block {
                blocker: b,
                attacker: token,
            },
        );
        checked(&mut g, Action::FinishBlockers);
    }
}

#[test]
fn msh_punch_zola_punch_factor_precedes_damage_replacement_with_creature_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Darksteel Colossus", Zone::Battlefield);
        paid(&mut g, p, "Twinflame Tyrant", &[]);
        g.state.objects[own as usize].gained_keywords = Keywords::LIFELINK;
        punch(&mut g, p, own, enemy);
        settle(&mut g);
        assert_eq!(g.object_view(enemy).damage, 8);
        assert_eq!(g.life(p), 28);
        assert_eq!(g.object_view(own).damage, 0);
    }
}

#[test]
fn msh_punch_zola_zola_threshold_can_disappear_during_actual_nested_mana_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, "Arnim Zola, Bio-Fanatic", &[]);
        g.state.objects[source as usize].summoning_sick = false;
        let rubble = named(&mut g, p, "Rubble Rouser", Zone::Battlefield);
        let fuel = corpses(&mut g, p)[0];
        let foreign = named(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
        let secret = named(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        let before = g.snapshot();
        assert!(g.legal_actions().contains(&announce(source)));
        assert_eq!(
            g.snapshot(),
            before,
            "funding does not preemptively exile fuel"
        );
        checked(&mut g, announce(source));
        while g.state.pending != (Pending::PayCosts { player: p }) {
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::ChooseActivationCost { .. } | Action::FinishTargets
                    )
                })
                .unwrap();
            checked(&mut g, a);
        }
        let parent = g.state.announcing.as_ref().unwrap().item.clone();
        checked(
            &mut g,
            Action::AnnounceMana {
                card: rubble,
                ability: 0,
                counter_payment: Counters::default(),
                production: [0, 0, 0, 1, 0, 0],
            },
        );
        assert!(
            g.observe(p.other())
                .suspended_mana_announcements
                .iter()
                .all(|v| v.cost.is_none())
        );
        assert!(g.observe(p.other()).objects.iter().all(|o| o.id != secret));
        let object = g.object_ref(foreign);
        reject(&mut g, Action::ChooseAdditionalCost { object });
        let object = g.object_ref(fuel);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        assert_eq!(g.object_view(fuel).zone, Zone::Exile);
        assert!(!g.activation_guard_available(
            ActivationGuard::GraveyardCardsAtLeast {
                kind: 1,
                minimum: 2
            },
            source,
            p
        ));
        assert_eq!(g.state.announcing.as_ref().unwrap().item, parent);
        assert_eq!(g.state.pending, Pending::PayCosts { player: p });
        finish(&mut g);
        settle(&mut g);
        assert_eq!(villains(&g, p).len(), 1);
        assert!(g.object_view(source).tapped);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.life(p.other()), 19);
    }
}
