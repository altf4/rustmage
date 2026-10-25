//! Paid whole-card behavior. The staging fixture permits these three fully
//! authored typed bodies to be tested before their catalogue admission flag.
use super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::*;
const ABSORB: &str = "Absorbing Man";
const SHURI: &str = "Shuri, Wakandan Inventor";
const TASK: &str = "Taskmaster, Mercenary Mimic";
fn prepared(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    if card.is_supported() {
        return named(g, p, name, zone);
    }
    assert!(
        [ABSORB, SHURI, TASK].contains(&name),
        "only owned whole typed bodies may use staging"
    );
    assert!(matches!(card.effect(), Effect::Creature(..)));
    let id = named(g, p, "Hill Giant", zone);
    if zone == Zone::Battlefield {
        g.remove_face_providers(id);
    }
    g.state.objects[id as usize].card = card;
    if zone == Zone::Battlefield {
        g.register_battlefield_sources(id);
    }
    id
}
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn pay(g: &mut Game) {
    for _ in 0..20 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, action);
    }
    panic!("copy card announcement remained unpaid");
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = paid_packet(g, p, name, targets);
    settle(g);
    id
}
fn paid_packet(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    priority(g, p);
    let id = prepared(g, p, name, Zone::Hand);
    let (colored, generic) = g.state.objects[id as usize].card.cost();
    g.state.players[p.index()].mana = colored;
    g.state.players[p.index()].mana[5] += generic;
    cast(g, id, None, targets);
    id
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..240 {
        if stop(g) {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                        | Action::Discard { .. }
                )
            })
            .unwrap_or_else(|| panic!("unexpected copy card choice {:?}", g.state.pending));
        checked(g, action);
    }
    panic!("copy card scenario stalled {:?}", g.state.pending);
}
fn next_main(g: &mut Game, p: Player) {
    let turn = g.state.turn;
    until(g, |g| {
        g.state.turn > turn && g.state.active == p && g.state.step == Step::Main1
    });
}
fn trigger(g: &mut Game, source: CardId, target: Option<CardId>) {
    g.prepare_triggers();
    let mut selected = false;
    for _ in 0..100 {
        if g.state.stack.iter().any(|item| {
            item.card == source && matches!(item.kind, StackKind::TriggeredAbility { ability: 0 })
        }) {
            return;
        }
        let actions = g.legal_actions();
        let target_action = target.map(|target| Action::ChooseTarget {
            target: Target::Card(target),
        });
        let action = if !selected && target_action.as_ref().is_some_and(|a| actions.contains(a)) {
            selected = true;
            target_action.unwrap()
        } else {
            actions
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::FinishTargets
                            | Action::PayCosts { .. }
                            | Action::PutTrigger { .. }
                            | Action::Pass
                    )
                })
                .unwrap()
        };
        checked(g, action);
    }
    panic!("first-main copy trigger did not reach stack");
}
fn shuri(g: &mut Game, p: Player, id: CardId, first: CardId, second: CardId) {
    priority(g, p);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(first),
        },
    );
    while g.state.announcing.as_ref().is_some_and(|a| a.group == 0) {
        checked(g, Action::FinishTargets);
    }
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(second),
        },
    );
    pay(g);
    assert!(g.state.objects[id as usize].tapped);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let bytes = g.state_bytes();
    let views = [
        serde_json::to_value(g.observe(Player::P0)).unwrap(),
        serde_json::to_value(g.observe(Player::P1)).unwrap(),
    ];
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.state_bytes(), bytes);
    assert_eq!(
        serde_json::to_value(g.observe(Player::P0)).unwrap(),
        views[0]
    );
    assert_eq!(
        serde_json::to_value(g.observe(Player::P1)).unwrap(),
        views[1]
    );
}
fn stats(g: &Game, id: CardId) -> (i128, i128) {
    let v = g.object_view(id);
    (v.power, v.toughness)
}
#[test]
fn msh_copy_cards_paid_printed_bodies_and_no_cast_turn_first_main_retrotrigger() {
    for p in [Player::P0, Player::P1] {
        for (name, size, keyword) in [
            (ABSORB, (4, 4), Keywords::VIGILANCE),
            (SHURI, (2, 1), Keywords::NONE),
            (TASK, (3, 5), Keywords::NONE),
        ] {
            let mut g = setup(p);
            let id = paid(&mut g, p, name, &[]);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(stats(&g, id), size);
            let v = g.object_view(id);
            assert!(v.is_legendary() && v.has_subtype("Human"));
            assert!(v.keywords.contains(keyword));
            assert!(v.summoning_sick);
            assert!(g.state.pending_triggers.is_empty() && g.state.stack.is_empty());
            assert!(g.temporary_copy_view(g.object_ref(id)).is_none());
        }
    }
}
#[test]
fn msh_copy_cards_absorbing_optional_no_target_keeps_its_printed_body() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, ABSORB, &[]);
        let object = g.object_ref(id);
        next_main(&mut g, p);
        trigger(&mut g, id, None);
        settle(&mut g);
        assert_eq!(g.object_view(id).name(), ABSORB);
        assert_eq!(stats(&g, id), (4, 4));
        assert!(g.temporary_copy_view(object).is_none());
        assert_eq!(g.object_ref(id), object);
    }
}
#[test]
fn msh_copy_cards_absorbing_target_union_any_controller_additive_types_and_fixed_body() {
    for p in [Player::P0, Player::P1] {
        for name in ["Forest", "Arc Reactor", "A Tale for the Ages"] {
            let mut g = setup(p);
            let id = paid(&mut g, p, ABSORB, &[]);
            let target = named(&mut g, p.other(), name, Zone::Battlefield);
            let object = g.object_ref(id);
            next_main(&mut g, p);
            trigger(&mut g, id, Some(target));
            settle(&mut g);
            let v = g.object_view(id);
            assert_eq!(v.name(), ABSORB);
            assert_eq!((v.power, v.toughness), (4, 4));
            assert!(v.is_legendary() && v.has_subtype("Human") && v.has_subtype("Villain"));
            assert!(v.keywords.contains(Keywords::VIGILANCE));
            assert_eq!(v.kinds & 1, 1);
            assert_eq!(v.kinds & g.current_kinds(target), g.current_kinds(target));
            assert_eq!(g.object_ref(id), object);
            assert!(!v.is_token());
            if name == "Forest" {
                assert!(v.has_subtype("Forest"));
                assert!(g.legal_actions().contains(&Action::TapMana { card: id }));
            }
        }
    }
}
#[test]
fn msh_copy_cards_absorbing_excludes_plain_aura_and_creature_targets_atomically() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, ABSORB, &[]);
        let bear = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        named(&mut g, p, "Forest", Zone::Battlefield);
        let aura = paid(&mut g, p, "Witness Protection", &[Target::Card(bear)]);
        next_main(&mut g, p);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(aura),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        trigger(&mut g, id, None);
        settle(&mut g);
        assert!(g.temporary_copy_view(g.object_ref(id)).is_none());
    }
}
#[test]
fn msh_copy_cards_absorbing_source_departure_before_resolution_does_not_copy_new_zone() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, ABSORB, &[]);
        let target = named(&mut g, p, "Forest", Zone::Battlefield);
        next_main(&mut g, p);
        trigger(&mut g, id, Some(target));
        paid(&mut g, p, "Murder", &[Target::Card(id)]);
        assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
        assert_eq!(g.object_view(id).name(), ABSORB);
        assert!(g.state.temporary_copies.is_none());
    }
}
#[test]
fn msh_copy_cards_absorbing_target_departure_fizzles_without_last_known_blueprint() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, ABSORB, &[]);
        let target = named(&mut g, p, "Arc Reactor", Zone::Battlefield);
        next_main(&mut g, p);
        trigger(&mut g, id, Some(target));
        paid(&mut g, p, "Disenchant", &[Target::Card(target)]);
        assert_eq!(g.object_view(id).name(), ABSORB);
        assert_eq!(stats(&g, id), (4, 4));
        assert!(g.state.temporary_copies.is_none());
    }
}
#[test]
fn msh_copy_cards_absorbing_captured_next_turn_expires_under_actual_confiscate_control() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, ABSORB, &[]);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        next_main(&mut g, p);
        trigger(&mut g, id, Some(land));
        settle(&mut g);
        let object = g.object_ref(id);
        next_main(&mut g, p.other());
        paid(&mut g, p.other(), "Confiscate", &[Target::Card(id)]);
        assert_eq!(g.object_view(id).controller, p.other());
        assert_eq!(g.current_kinds(id) & 3, 3);
        next_main(&mut g, p);
        settle(&mut g);
        assert_eq!(g.object_ref(id), object);
        assert_eq!(g.object_view(id).controller, p.other());
        assert_eq!(g.current_kinds(id) & 2, 0);
        assert_eq!(g.object_view(id).name(), ABSORB);
        assert!(g.temporary_copy_view(object).is_none());
    }
}
#[test]
fn msh_copy_cards_shuri_paid_activation_summoning_sickness_timing_and_mana_atomicity() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, SHURI, &[]);
        let action = Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        };
        let a = named(&mut g, p, "Mazemind Tome", Zone::Battlefield);
        let b = named(&mut g, p, "The Ten Rings", Zone::Battlefield);
        reject(&mut g, action.clone());
        next_main(&mut g, p);
        settle(&mut g);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, action.clone());
        shuri(&mut g, p, id, a, b);
        settle(&mut g);
        assert_eq!(g.object_view(a).name(), "The Ten Rings");
        assert!(!g.object_view(a).is_legendary());
        reject(&mut g, action);
    }
}
#[test]
fn msh_copy_cards_shuri_same_artifact_both_groups_and_eot_restore_preserves_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, SHURI, &[]);
        let rings = named(&mut g, p, "The Ten Rings", Zone::Battlefield);
        next_main(&mut g, p);
        settle(&mut g);
        let object = g.object_ref(rings);
        shuri(&mut g, p, id, rings, rings);
        settle(&mut g);
        assert!(!g.object_view(rings).is_legendary());
        assert_eq!(g.object_view(rings).name(), "The Ten Rings");
        assert_eq!(g.object_ref(rings), object);
        assert!(g.temporary_copy_view(object).is_some());
        let turn = g.state.turn;
        until(&mut g, |g| g.state.turn > turn);
        assert!(g.object_view(rings).is_legendary());
        assert_eq!(g.object_ref(rings), object);
        assert!(g.temporary_copy_view(object).is_none());
    }
}
#[test]
fn msh_copy_cards_shuri_either_target_illegal_means_no_copy_even_when_other_survives() {
    for p in [Player::P0, Player::P1] {
        for removed in [0, 1, 2] {
            let mut g = setup(p);
            let id = paid(&mut g, p, SHURI, &[]);
            let a = named(&mut g, p, "Arc Reactor", Zone::Battlefield);
            let b = named(&mut g, p, "The Ten Rings", Zone::Battlefield);
            next_main(&mut g, p);
            settle(&mut g);
            shuri(&mut g, p, id, a, b);
            if removed != 1 {
                paid_packet(&mut g, p, "Disenchant", &[Target::Card(a)]);
            }
            if removed != 0 {
                paid_packet(&mut g, p, "Disenchant", &[Target::Card(b)]);
            }
            settle(&mut g);
            assert!(g.state.temporary_copies.is_none());
            if g.on_battlefield(a) {
                assert_eq!(g.object_view(a).name(), "Arc Reactor");
            }
        }
    }
}
#[test]
fn msh_copy_cards_taskmaster_battlefield_and_either_graveyard_use_replacement_types_and_copied_pt()
{
    for p in [Player::P0, Player::P1] {
        for (owner, zone) in [
            (p, Zone::Battlefield),
            (p, Zone::Graveyard),
            (p.other(), Zone::Graveyard),
        ] {
            let mut g = setup(p);
            let id = paid(&mut g, p, TASK, &[]);
            let target = named(&mut g, owner, "Hill Giant", zone);
            next_main(&mut g, p);
            trigger(&mut g, id, Some(target));
            settle(&mut g);
            let v = g.object_view(id);
            assert_eq!(v.name(), TASK);
            assert_eq!((v.power, v.toughness), (3, 3));
            assert_eq!(v.kinds, 1);
            assert!(
                v.is_legendary()
                    && v.has_subtype("Human")
                    && v.has_subtype("Mercenary")
                    && v.has_subtype("Villain")
            );
            assert!(!v.has_subtype("Giant"));
            assert_eq!(g.state.objects[target as usize].zone, zone);
        }
    }
}
#[test]
fn msh_copy_cards_taskmaster_optional_none_and_noncreature_graveyard_targets_are_rejected() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, TASK, &[]);
        let wrong = named(&mut g, p, "Arc Reactor", Zone::Graveyard);
        next_main(&mut g, p);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(wrong),
            },
        );
        trigger(&mut g, id, None);
        settle(&mut g);
        assert_eq!(stats(&g, id), (3, 5));
        assert!(g.state.temporary_copies.is_none());
    }
}

#[test]
fn msh_copy_cards_shuri_reduction_actual_artifact_payment_and_unaffected_activation_costs() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, SHURI, &[]);
        let artifact = named(&mut g, p, "Hedron Archive", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        cast(&mut g, artifact, None, &[]);
        settle(&mut g);
        assert!(g.on_battlefield(artifact));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let tome = named(&mut g, p, "Mazemind Tome", Zone::Battlefield);
        let action = Action::AnnounceActivate {
            card: tome,
            ability: 1,
            counter_payment: Counters::default(),
        };
        // Archive is tapped through its real mana action, so it cannot mask the
        // deliberately insufficient one-mana activation funding check.
        priority(&mut g, p);
        checked(
            &mut g,
            Action::ActivateMana {
                card: artifact,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 0, 2],
            },
        );
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        reject(&mut g, action.clone());
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        let hand = g.hand_count(p);
        checked(&mut g, action);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.hand_count(p), hand + 1);
    }
}
#[test]
fn msh_copy_cards_shuri_reduction_respects_controller_suppression_and_zero_floor() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let shuri = paid(&mut g, p, SHURI, &[]);
        let zero = named(&mut g, p, "Mox Jasper", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        cast(&mut g, zero, None, &[]);
        settle(&mut g);
        assert!(g.on_battlefield(zero));
        let removal = paid(&mut g, p, "Witness Protection", &[Target::Card(shuri)]);
        next_main(&mut g, p);
        settle(&mut g);
        assert!(!g.object_view(shuri).summoning_sick);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: shuri,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let artifact = named(&mut g, p, "Hedron Archive", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        reject(&mut g, Action::AnnounceCast { card: artifact });
        paid(&mut g, p, "Disenchant", &[Target::Card(removal)]);
        self::shuri(&mut g, p, shuri, zero, zero);
        settle(&mut g);
        next_main(&mut g, p.other());
        settle(&mut g);
        let other = named(&mut g, p.other(), "Hedron Archive", Zone::Hand);
        g.state.players[p.other().index()].mana = [0, 0, 0, 0, 0, 3];
        reject(&mut g, Action::AnnounceCast { card: other });
        g.state.players[p.other().index()].mana = [0, 0, 0, 0, 0, 4];
        cast(&mut g, other, None, &[]);
        settle(&mut g);
        assert!(g.on_battlefield(other));
        assert_eq!(g.state.players[p.other().index()].mana, [0; 6]);
    }
}
#[test]
fn msh_copy_cards_taskmaster_retains_copied_power_cda_and_actual_new_controller_counts() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, TASK, &[]);
        let namor = named(&mut g, p, "Namor the Sub-Mariner", Zone::Battlefield);
        named(&mut g, p.other(), "Brineborn Cutthroat", Zone::Battlefield);
        named(&mut g, p.other(), "Brineborn Cutthroat", Zone::Battlefield);
        next_main(&mut g, p);
        trigger(&mut g, id, Some(namor));
        settle(&mut g);
        assert_eq!(stats(&g, id), (1, 4));
        assert!(!g.object_view(id).has_subtype("Merfolk"));
        next_main(&mut g, p.other());
        paid(&mut g, p.other(), "Confiscate", &[Target::Card(id)]);
        assert_eq!(g.object_view(id).controller, p.other());
        assert_eq!(stats(&g, id), (2, 4));
        next_main(&mut g, p);
        settle(&mut g);
        assert_eq!(g.object_view(id).controller, p.other());
        assert_eq!(stats(&g, id), (3, 5));
        assert!(g.temporary_copy_view(g.object_ref(id)).is_none());
    }
}
#[test]
fn msh_copy_cards_taskmaster_copies_current_token_recipe_but_remains_physical_nontoken() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, TASK, &[]);
        paid(&mut g, p, "Mole Man, Moloid Master", &[]);
        let land = named(&mut g, p, "Forest", Zone::Hand);
        priority(&mut g, p);
        checked(&mut g, Action::PlayLand { card: land });
        settle(&mut g);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| {
                o.zone == Zone::Battlefield
                    && o.is_token()
                    && o.card == Card::from_name("@token/msh-moloid").unwrap()
            })
            .map(|(i, _)| i as CardId)
            .unwrap();
        next_main(&mut g, p);
        trigger(&mut g, id, Some(token));
        settle(&mut g);
        let v = g.object_view(id);
        assert_eq!(v.name(), TASK);
        assert!(!v.is_token());
        assert_eq!((v.power, v.toughness), (1, 1));
        assert_eq!(v.kinds, 1);
        assert!(v.has_subtype("Human") && !v.has_subtype("Minion"));
        paid(&mut g, p, "Murder", &[Target::Card(id)]);
        assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
        assert_eq!(
            g.state.objects[id as usize].card,
            Card::from_name(TASK).unwrap()
        );
    }
}

#[test]
fn msh_copy_cards_taskmaster_opposing_ward_requires_real_payment_or_counters_copy_ability() {
    for p in [Player::P0, Player::P1] {
        for paid_ward in [false, true] {
            let mut g = setup(p);
            let id = paid(&mut g, p, TASK, &[]);
            let terror = named(&mut g, p.other(), "Tolarian Terror", Zone::Battlefield);
            next_main(&mut g, p);
            trigger(&mut g, id, Some(terror));
            g.state.players[p.index()].mana = if paid_ward {
                [0, 0, 0, 0, 0, 2]
            } else {
                [0; 6]
            };
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::ResolutionMana { .. })
            });
            checked(
                &mut g,
                if paid_ward {
                    Action::PayResolutionMana {
                        payment: [0, 0, 0, 0, 0, 2],
                    }
                } else {
                    Action::DeclineResolutionMana
                },
            );
            settle(&mut g);
            assert_eq!(stats(&g, id), if paid_ward { (5, 5) } else { (3, 5) });
            assert_eq!(g.temporary_copy_view(g.object_ref(id)).is_some(), paid_ward);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}
#[test]
fn msh_copy_cards_absorbing_copied_forest_mana_is_actual_and_speculatively_fundable() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, ABSORB, &[]);
        let forest = named(&mut g, p.other(), "Forest", Zone::Battlefield);
        next_main(&mut g, p);
        trigger(&mut g, id, Some(forest));
        settle(&mut g);
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0; 6];
        let before = g.snapshot();
        assert!(g.can_fund(FundingGoal {
            actor: p,
            required: [0, 0, 0, 0, 1, 0],
            generic: 0,
            reserve: None,
            source: None
        }));
        assert_eq!(g.snapshot(), before);
        checked(&mut g, Action::TapMana { card: id });
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 1, 0]);
        assert!(g.object_view(id).tapped);
        assert_eq!(g.object_view(id).controller, p);
        assert_eq!(g.object_view(forest).controller, p.other());
    }
}
#[test]
fn msh_copy_cards_absorbing_bounded_artifact_aura_qualifies_then_creature_aura_sba_restores_physical_card()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, ABSORB, &[]);
        let creature = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let aura = paid(&mut g, p, "Witness Protection", &[Target::Card(creature)]);
        // Exact bounded artifact-Aura blueprint, using the already-tested copy
        // exception. This does not claim another playable artifact-Aura producer.
        g.set_copy_recipe(
            g.object_ref(aura),
            CopyRecipe::new(g.state.objects[aura as usize].card)
                .with_exceptions(CopyExceptions::ARTIFACT),
        );
        next_main(&mut g, p);
        trigger(&mut g, id, Some(aura));
        settle(&mut g);
        assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
        assert_eq!(
            g.state.objects[id as usize].card,
            Card::from_name(ABSORB).unwrap()
        );
        assert!(g.temporary_copy_view(g.object_ref(id)).is_none());
        assert!(g.on_battlefield(aura));
    }
}
#[test]
fn msh_copy_cards_absorbing_bounded_artifact_power_cda_is_replaced_by_fixed_four() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, ABSORB, &[]);
        let namor = named(
            &mut g,
            p.other(),
            "Namor the Sub-Mariner",
            Zone::Battlefield,
        );
        // Bounded existing copy-exception blueprint makes this CDA creature a
        // legal artifact target. The paid first-main copy itself is ordinary.
        g.set_copy_recipe(
            g.object_ref(namor),
            CopyRecipe::new(g.state.objects[namor as usize].card)
                .with_exceptions(CopyExceptions::ARTIFACT),
        );
        assert_eq!(stats(&g, namor), (1, 4));
        next_main(&mut g, p);
        trigger(&mut g, id, Some(namor));
        settle(&mut g);
        assert_eq!(stats(&g, id), (4, 4));
        assert!(g.object_view(id).has_subtype("Merfolk"));
        assert!(g.object_view(id).has_subtype("Human"));
        assert_eq!(g.current_kinds(id) & 17, 17);
    }
}
#[test]
fn msh_copy_cards_shuri_source_departure_owned_orphan_raw_index_restore_and_full_nonactor_privacy()
{
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, SHURI, &[]);
        let first = named(&mut g, p, "Mazemind Tome", Zone::Battlefield);
        let second = named(&mut g, p, "Hedron Archive", Zone::Battlefield);
        next_main(&mut g, p);
        settle(&mut g);
        shuri(&mut g, p, source, first, second);
        let secret = named(&mut g, p.other(), "Omniscience", Zone::Hand);
        paid_packet(&mut g, p, "Murder", &[Target::Card(source)]);
        let snapshot = g.snapshot();
        let actor = g.actor().unwrap();
        let revision = g.revision();
        let index = g
            .legal_actions()
            .iter()
            .position(|a| *a == Action::Pass)
            .unwrap();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        drop(g);
        raw.apply(actor, Action::Pass).unwrap();
        indexed.apply_index(actor, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        for viewer in [p, p.other()] {
            assert_eq!(
                serde_json::to_value(raw.observe(viewer)).unwrap(),
                serde_json::to_value(indexed.observe(viewer)).unwrap()
            );
        }
        raw.restore(&snapshot).unwrap();
        assert_eq!(
            raw.apply_index(actor, revision, index),
            Err(Error::StaleRevision)
        );
        assert!(!raw.observe(p).objects.iter().any(|v| v.id == secret));
        settle(&mut raw);
        assert_eq!(raw.state.objects[source as usize].zone, Zone::Graveyard);
        assert_eq!(raw.object_view(first).name(), "Hedron Archive");
        assert!(!raw.object_view(first).is_legendary());
        assert!(!raw.observe(p).objects.iter().any(|v| v.id == secret));
    }
}
#[test]
fn msh_copy_cards_shuri_copy_has_no_enter_trigger_but_keeps_copied_death_program_and_physical_identity()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let shuri_id = paid(&mut g, p, SHURI, &[]);
        let first = named(&mut g, p, "Arc Reactor", Zone::Battlefield);
        let second = named(&mut g, p, "Solemn Simulacrum", Zone::Battlefield);
        next_main(&mut g, p);
        settle(&mut g);
        let object = g.object_ref(first);
        let library = g.state.players[p.index()].library.len();
        shuri(&mut g, p, shuri_id, first, second);
        settle(&mut g);
        assert_eq!(g.object_view(first).name(), "Solemn Simulacrum");
        assert_eq!(g.state.players[p.index()].library.len(), library);
        assert_eq!(g.object_ref(first), object);
        assert!(g.state.pending_triggers.is_empty());
        let hand = g.hand_count(p);
        paid(&mut g, p, "Murder", &[Target::Card(first)]);
        assert_eq!(g.hand_count(p), hand + 1);
        assert_eq!(
            g.state.objects[first as usize].card,
            Card::from_name("Arc Reactor").unwrap()
        );
        assert_eq!(g.state.objects[first as usize].zone, Zone::Graveyard);
        assert!(!g.object_view(first).is_token());
    }
}
#[test]
fn msh_copy_cards_shuri_bounded_target_control_change_is_rechecked_for_both_groups() {
    for p in [Player::P0, Player::P1] {
        for change_first in [false, true] {
            let mut g = setup(p);
            let source = paid(&mut g, p, SHURI, &[]);
            let a = named(&mut g, p, "Mazemind Tome", Zone::Battlefield);
            let b = named(&mut g, p, "Hedron Archive", Zone::Battlefield);
            next_main(&mut g, p);
            settle(&mut g);
            shuri(&mut g, p, source, a, b);
            // Bounded controller event, distinct from departure. Selection, paid
            // costs, resolution legality and the no-copy result use normal rules.
            g.change_controller(g.object_ref(if change_first { a } else { b }), p.other());
            settle(&mut g);
            assert_eq!(g.object_view(a).name(), "Mazemind Tome");
            assert!(g.state.temporary_copies.is_none());
        }
    }
}
#[test]
fn msh_copy_cards_taskmaster_graveyard_uses_physical_face_instead_of_prior_shuri_copy_lki() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let task = paid(&mut g, p, TASK, &[]);
        let source = paid(&mut g, p, SHURI, &[]);
        let original = named(&mut g, p, "Solemn Simulacrum", Zone::Battlefield);
        let blueprint = named(&mut g, p, "Meteor Golem", Zone::Battlefield);
        next_main(&mut g, p);
        trigger(&mut g, task, None);
        settle(&mut g);
        shuri(&mut g, p, source, original, blueprint);
        settle(&mut g);
        assert_eq!(stats(&g, original), (3, 3));
        paid(&mut g, p, "Murder", &[Target::Card(original)]);
        assert_eq!(
            g.state.objects[original as usize].card,
            Card::from_name("Solemn Simulacrum").unwrap()
        );
        next_main(&mut g, p);
        trigger(&mut g, task, Some(original));
        settle(&mut g);
        assert_eq!(stats(&g, task), (2, 2));
        assert_eq!(g.object_view(task).name(), TASK);
        assert!(!g.object_view(task).has_subtype("Golem"));
        assert_eq!(g.current_kinds(task), 1);
    }
}
#[test]
fn msh_copy_cards_shuri_sorcery_timing_and_controlled_artifact_groups_are_actual() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, SHURI, &[]);
        let mine = named(&mut g, p, "Mazemind Tome", Zone::Battlefield);
        let theirs = named(&mut g, p.other(), "Hedron Archive", Zone::Battlefield);
        next_main(&mut g, p);
        settle(&mut g);
        paid_packet(&mut g, p, "Shock", &[Target::Player(p.other())]);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        settle(&mut g);
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(theirs),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(source),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(mine),
            },
        );
        while g.state.announcing.as_ref().is_some_and(|a| a.group == 0) {
            checked(&mut g, Action::FinishTargets);
        }
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(theirs),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(mine),
            },
        );
        pay(&mut g);
        settle(&mut g);
    }
}
#[test]
fn msh_copy_cards_shuri_artifact_reduction_combines_with_real_improvise_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, SHURI, &[]);
        let fuel: Vec<_> = (0..4)
            .map(|_| named(&mut g, p, "Mazemind Tome", Zone::Battlefield))
            .collect();
        let arc = named(&mut g, p, "Arc Reactor", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, Action::AnnounceCast { card: arc });
        checked(&mut g, Action::BeginImprovise);
        for id in &fuel {
            let object = g.object_ref(*id);
            checked(&mut g, Action::ChooseImproviser { object });
        }
        pay(&mut g);
        settle(&mut g);
        assert!(g.on_battlefield(arc));
        assert!(g.object_view(arc).tapped);
        assert!(fuel.iter().all(|id| g.object_view(*id).tapped));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn msh_copy_cards_taskmaster_graveyard_dfc_front_and_copied_transform_keep_physical_single_face() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, TASK, &[]);
        let monica = named(&mut g, p, "Monica Rambeau", Zone::Battlefield);
        priority(&mut g, p);
        // Monica's full printed transform cost is {2}{W}{W}{R}; Taskmaster
        // inherits that same activation cost from the graveyard front face.
        g.state.players[p.index()].mana = [2, 0, 0, 1, 0, 2];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: monica,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert!(g.object_view(monica).face.unwrap().back);
        paid(&mut g, p, "Murder", &[Target::Card(monica)]);
        assert_eq!(g.object_view(monica).card.name(), "Monica Rambeau");
        // Casting Murder while Photon is still present put a real +1/+1
        // counter on Taskmaster. Copying preserves it in addition to 3/3 base.
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
        next_main(&mut g, p);
        trigger(&mut g, source, Some(monica));
        settle(&mut g);
        assert_eq!(stats(&g, source), (4, 4));
        assert!(g.object_view(source).face.is_none());
        let origin = g.copy_ability_origin(g.object_ref(source));
        priority(&mut g, p);
        g.state.players[p.index()].mana = [2, 0, 0, 1, 0, 2];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert!(g.object_view(source).face.is_none());
        assert_eq!(g.copy_ability_origin(g.object_ref(source)), origin);
        assert_eq!(g.object_view(source).name(), TASK);
    }
}

#[test]
fn msh_copy_cards_first_main_triggers_neither_opponents_main_nor_second_main() {
    for p in [Player::P0, Player::P1] {
        for name in [ABSORB, TASK] {
            let mut g = setup(p);
            let id = paid(&mut g, p, name, &[]);
            next_main(&mut g, p.other());
            assert!(
                !g.state
                    .stack
                    .iter()
                    .chain(&g.state.pending_triggers)
                    .any(|item| item.card == id)
            );
            assert!(
                !g.state
                    .announcing
                    .as_ref()
                    .is_some_and(|a| a.item.card == id)
            );
            next_main(&mut g, p);
            trigger(&mut g, id, None);
            settle(&mut g);
            let turn = g.state.turn;
            until(&mut g, |g| {
                g.state.turn == turn && g.state.step == Step::Main2
            });
            assert!(
                !g.state
                    .stack
                    .iter()
                    .chain(&g.state.pending_triggers)
                    .any(|item| item.card == id)
            );
            assert!(
                !g.state
                    .announcing
                    .as_ref()
                    .is_some_and(|a| a.item.card == id)
            );
        }
    }
}
#[test]
fn msh_copy_cards_shuri_reduces_only_generic_part_of_colored_artifact_spell() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, SHURI, &[]);
        let artifact = named(&mut g, p, "Simulacrum Synthesizer", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        reject(&mut g, Action::AnnounceCast { card: artifact });
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        reject(&mut g, Action::AnnounceCast { card: artifact });
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
        cast(&mut g, artifact, None, &[]);
        settle(&mut g);
        assert!(g.on_battlefield(artifact));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn msh_copy_cards_shuri_copies_runtime_token_blueprint_without_its_noncopyable_haste_or_delayed_exile()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, SHURI, &[]);
        let recipient = named(&mut g, p, "Mazemind Tome", Zone::Battlefield);
        let creature = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        next_main(&mut g, p);
        settle(&mut g);
        paid(&mut g, p, "Molten Duplication", &[Target::Card(creature)]);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| o.zone == Zone::Battlefield && o.is_copy_token())
            .map(|(i, _)| i as CardId)
            .unwrap();
        assert!(g.keywords(token).contains(Keywords::HASTE));
        shuri(&mut g, p, source, recipient, token);
        settle(&mut g);
        assert_eq!(g.object_view(recipient).name(), "Hill Giant");
        assert_eq!(stats(&g, recipient), (3, 3));
        assert_eq!(g.current_kinds(recipient) & 17, 17);
        assert!(!g.object_view(recipient).is_token());
        assert!(!g.keywords(recipient).contains(Keywords::HASTE));
        let object = g.object_ref(recipient);
        let turn = g.state.turn;
        until(&mut g, |g| g.state.turn > turn);
        assert!(g.on_battlefield(recipient));
        assert_eq!(g.object_ref(recipient), object);
        assert_eq!(g.object_view(recipient).name(), "Mazemind Tome");
        assert!(!g.on_battlefield(token));
    }
}

/// Real paid entry and first-main copy; optional unrelated Aura forces layer-six evaluation.
pub(super) fn paid_subtype_cda_fixture(
    p: Player,
    blueprint: &str,
    layer_six: bool,
) -> (Game, CardId) {
    let mut g = setup(p);
    let original = paid(&mut g, p, blueprint, &[]);
    let task = paid(&mut g, p, TASK, &[]);
    next_main(&mut g, p);
    trigger(&mut g, task, Some(original));
    settle(&mut g);
    if layer_six {
        let bear = paid(&mut g, p, "Bear Cub", &[]);
        paid(&mut g, p, "Witness Protection", &[Target::Card(bear)]);
    }
    (g, task)
}

#[test]
fn msh_copy_subtype_cda_paid_taskmaster_excludes_intrinsic_changeling_in_both_query_paths() {
    for p in [Player::P0, Player::P1] {
        for blueprint in ["Three Tree Mascot", "Barkform Harvester"] {
            for layer_six in [false, true] {
                let (g, task) = paid_subtype_cda_fixture(p, blueprint, layer_six);
                assert_eq!(g.evaluated_abilities().is_some(), layer_six);
                assert!(!g.keywords(task).contains(Keywords::CHANGELING));
                assert!(!g.object_view(task).keywords.contains(Keywords::CHANGELING));
                assert!(!g.has_subtype(task, "Merfolk"));
                assert!(!g.has_subtype(task, "Hero"));
                assert!(g.has_subtype(task, "Human") && g.has_subtype(task, "Mercenary"));
                assert!(g.has_subtype(task, "Villain"));
                if blueprint == "Barkform Harvester" {
                    assert!(g.keywords(task).contains(Keywords::REACH));
                }
            }
        }
    }
}

#[test]
fn msh_copy_subtype_cda_paid_absorbing_additive_types_keep_changeling() {
    for p in [Player::P0, Player::P1] {
        for blueprint in ["Three Tree Mascot", "Barkform Harvester"] {
            let mut g = setup(p);
            let original = paid(&mut g, p, blueprint, &[]);
            let source = paid(&mut g, p, ABSORB, &[]);
            next_main(&mut g, p);
            trigger(&mut g, source, Some(original));
            settle(&mut g);
            assert!(g.keywords(source).contains(Keywords::CHANGELING));
            assert!(g.keywords(source).contains(Keywords::VIGILANCE));
            assert!(g.has_subtype(source, "Merfolk") && g.has_subtype(source, "Hero"));
            assert_eq!(stats(&g, source), (4, 4));
            let bear = paid(&mut g, p, "Bear Cub", &[]);
            paid(&mut g, p, "Witness Protection", &[Target::Card(bear)]);
            assert!(g.evaluated_abilities().is_some());
            assert!(
                g.object_view(source)
                    .keywords
                    .contains(Keywords::CHANGELING)
            );
        }
    }
}

#[test]
fn msh_copy_subtype_cda_external_keyword_grant_survives_intrinsic_filter_and_restore() {
    for p in [Player::P0, Player::P1] {
        let (mut g, task) = paid_subtype_cda_fixture(p, "Barkform Harvester", false);
        let snapshot = g.snapshot();
        // The legacy gained-keyword component is also later than the intrinsic baseline.
        g.state.objects[task as usize].gained_keywords = Keywords::CHANGELING;
        assert!(g.object_view(task).keywords.contains(Keywords::CHANGELING));
        g.restore(&snapshot).unwrap();
        // Bounded existing resolution-grant API, not an invented playable grant card.
        g.grant_keyword_effects(&[task], g.object_ref(task), Keywords::CHANGELING);
        assert!(g.object_view(task).keywords.contains(Keywords::CHANGELING));
        assert!(g.keywords(task).contains(Keywords::REACH));
        let mut child = {
            let parent = g.clone();
            parent.clone()
        };
        drop(g);
        assert!(
            child
                .object_view(task)
                .keywords
                .contains(Keywords::CHANGELING)
        );
        child.restore(&snapshot).unwrap();
        assert!(
            !child
                .object_view(task)
                .keywords
                .contains(Keywords::CHANGELING)
        );
    }
}

#[test]
fn msh_copy_subtype_cda_undercover_conditional_all_types_remains_live() {
    for p in [Player::P0, Player::P1] {
        let (mut g, task) = paid_subtype_cda_fixture(p, "Undercover Skrull", false);
        assert!(!g.has_subtype(task, "Merfolk"));
        for _ in 0..2 {
            let bear = paid(&mut g, p, "Bear Cub", &[]);
            paid(&mut g, p, "Murder", &[Target::Card(bear)]);
        }
        assert!(g.has_subtype(task, "Merfolk") && g.has_subtype(task, "Hero"));
        assert!(!g.keywords(task).contains(Keywords::CHANGELING));
        assert_eq!(stats(&g, task), (3, 3));
        paid(&mut g, p, "Rest in Peace", &[]);
        assert!(!g.has_subtype(task, "Merfolk"));
        assert!(!g.keywords(task).contains(Keywords::CHANGELING));
        assert_eq!(stats(&g, task), (1, 1));
    }
}

#[test]
fn msh_copy_subtype_cda_paid_molten_then_shuri_flat_copy_never_restores_changeling() {
    for p in [Player::P0, Player::P1] {
        let (mut g, task) = paid_subtype_cda_fixture(p, "Barkform Harvester", false);
        paid_packet(&mut g, p, "Molten Duplication", &[Target::Card(task)]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Legend { .. })
        });
        let copied = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, object)| object.zone == Zone::Battlefield && object.is_copy_token())
            .map(|(i, _)| i as CardId)
            .unwrap();
        let copied_ref = g.object_ref(copied);
        checked(&mut g, Action::KeepLegend { object: copied_ref });
        settle(&mut g);
        assert!(
            !g.object_view(copied)
                .keywords
                .contains(Keywords::CHANGELING)
        );
        assert!(g.object_view(copied).keywords.contains(Keywords::REACH));
        assert!(!g.has_subtype(copied, "Merfolk"));
        let shuri_source = paid(&mut g, p, SHURI, &[]);
        paid(
            &mut g,
            p,
            "Impolite Entrance",
            &[Target::Card(shuri_source)],
        );
        let recipient = paid(&mut g, p, "Arc Reactor", &[]);
        shuri(&mut g, p, shuri_source, recipient, copied);
        settle(&mut g);
        assert!(
            !g.object_view(recipient)
                .keywords
                .contains(Keywords::CHANGELING)
        );
        assert!(g.object_view(recipient).keywords.contains(Keywords::REACH));
        assert!(!g.has_subtype(recipient, "Merfolk"));
        assert!(!ability_world::CharacteristicWorld::legendary(
            &g, recipient
        ));
    }
}

#[test]
fn msh_copy_subtype_cda_taskmaster_cannot_spend_hero_mana_on_copied_mascot_activation() {
    for p in [Player::P0, Player::P1] {
        let (mut g, task) = paid_subtype_cda_fixture(p, "Three Tree Mascot", false);
        // The original changeling could otherwise legally convert Hero-only mana
        // through its own Hero activation before the enclosing Taskmaster cost.
        let original = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(i, object)| {
                *i != task as usize
                    && object.zone == Zone::Battlefield
                    && object.card.name() == "Three Tree Mascot"
            })
            .map(|(i, _)| i as CardId)
            .unwrap();
        paid(&mut g, p, "Murder", &[Target::Card(original)]);
        let tower = named(&mut g, p, "Avengers Tower", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: tower });
        g.state.players[p.index()].mana = [0; 6];
        checked(
            &mut g,
            Action::ActivateMana {
                card: tower,
                ability: 1,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 1, 0, 0, 0, 0],
            },
        );
        reject(
            &mut g,
            Action::AnnounceMana {
                card: task,
                ability: 0,
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 1, 0],
            },
        );
        g.state.players[p.index()].mana[5] = 1;
        checked(
            &mut g,
            Action::ActivateMana {
                card: task,
                ability: 0,
                payment: [0, 0, 0, 0, 0, 1],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 1, 0],
            },
        );
        assert_eq!(g.state.players[p.index()].mana, [0, 1, 0, 0, 1, 0]);
        assert!(!g.object_view(task).keywords.contains(Keywords::CHANGELING));
    }
}
