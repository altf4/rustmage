//! Printed outcomes and bounded ownership checks for amass attachment and live land allowances.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..48 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("plate/company decision budget {:?}", g.state.pending);
}
fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
}
fn drive(g: &mut Game, preferred: Option<CardId>) {
    for _ in 0..96 {
        if quiet(g) {
            return;
        }
        let actions = g.legal_actions();
        let action = preferred
            .and_then(|id| {
                actions.iter().find(|a| match a {
                    Action::ChooseArmy { object } => object.card == id,
                    Action::ChooseTarget { target } => *target == Target::Card(id),
                    _ => false,
                })
            })
            .or_else(|| {
                actions
                    .iter()
                    .find(|a| matches!(a, Action::ChooseReplacement { .. }))
            })
            .or_else(|| {
                actions.iter().find(|a| {
                    matches!(
                        a,
                        Action::ChooseArmy { .. }
                            | Action::FinishTargets
                            | Action::PutTrigger { .. }
                            | Action::Pass
                    )
                })
            })
            .unwrap_or_else(|| panic!("explicit choice {:?}", g.state.pending))
            .clone();
        act(g, action);
    }
    panic!("plate/company resolution budget");
}
fn types(g: &mut Game, id: CardId, names: &'static [&'static str]) {
    let recipient = g.object_ref(id);
    let timestamp = g.next_role_timestamp();
    g.state
        .roles
        .get_or_insert_with(Default::default)
        .creature_types
        .push(subtypes::CreatureTypesEffectView {
            recipient,
            source: recipient,
            types: names,
            additional: false,
            timestamp,
        });
}
fn armies(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && g.is_creature(id as CardId)
                && g.has_subtype(id as CardId, "Army"))
            .then_some(id as CardId)
        })
        .collect()
}
fn plate(g: &mut Game, p: Player, black: bool) -> CardId {
    let id = put(g, p, "Goblin Plate Mail", Zone::Hand);
    g.state.players[p.index()].mana = if black {
        [0, 0, 1, 0, 0, 1]
    } else {
        [0, 0, 0, 1, 0, 1]
    };
    cast(g, id, None);
    id
}
fn company(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Thranduil's Company", Zone::Hand);
    g.state.players[p.index()].mana = [0, 1, 0, 0, 1, 2];
    cast(g, id, None);
    drive(g, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(stats(g, id), (3, 4));
    assert!(g.has_subtype(id, "Elf") && g.has_subtype(id, "Soldier"));
    id
}
fn paid(g: &mut Game, p: Player, name: &str, target: Option<Target>) -> CardId {
    let id = put(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = [12; 6];
    cast(g, id, target);
    drive(g, None);
    id
}
fn bind(g: &mut Game, id: CardId) {
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(id),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
fn play(g: &mut Game, land: CardId, target: CardId) {
    act(g, Action::PlayLand { card: land });
    drive(g, Some(target));
}
#[test]
fn hob_plate_company_plate_hybrid_amass_recipe_paid_equip_and_menace() {
    for p in [Player::P0, Player::P1] {
        for black in [false, true] {
            let mut g = scenario(p);
            let plate = plate(&mut g, p, black);
            drive(&mut g, None);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            let a = armies(&g, p);
            assert_eq!(a.len(), 1);
            let army = a[0];
            let v = g.object_view(army);
            assert_eq!((v.colors, v.kinds, v.power, v.toughness), (4, 1, 2, 1));
            assert!(g.has_subtype(army, "Goblin"));
            assert_eq!(g.counter_count(army, CounterKind::PlusOne), 1);
            assert_eq!(g.object_view(plate).attached_to, Some(g.object_ref(army)));
            assert!(g.keywords(army).contains(Keywords::MENACE));
            let bearer = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let first = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let second = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: plate,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(first),
                },
            );
            bind(&mut g, bearer);
            pay(&mut g);
            drive(&mut g, None);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(stats(&g, bearer), (3, 2));
            assert_eq!(stats(&g, army), (1, 1));
            assert!(!g.keywords(army).contains(Keywords::MENACE));
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            act(&mut g, Action::Attack { card: bearer });
            act(&mut g, Action::FinishAttackers);
            assert_eq!(g.actor(), Some(p));
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: plate,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            until(&mut g, |g| g.state.pending == Pending::Blockers);
            act(
                &mut g,
                Action::Block {
                    blocker: first,
                    attacker: bearer,
                },
            );
            reject(&mut g, Action::FinishBlockers);
            act(
                &mut g,
                Action::Block {
                    blocker: second,
                    attacker: bearer,
                },
            );
            act(&mut g, Action::FinishBlockers);
        }
    }
}
#[test]
fn hob_plate_company_plate_exact_nontarget_army_choice_and_owned_independence() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let a = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        types(&mut g, a, &["Army"]);
        g.state.objects[a as usize].gained_keywords = Keywords::HEXPROOF;
        let b = put(&mut g, p, "@token/goblin_army", Zone::Battlefield);
        g.set_counters(b, Counters::of(CounterKind::PlusOne, 1));
        let enemy = put(&mut g, p.other(), "@token/goblin_army", Zone::Battlefield);
        g.set_counters(enemy, Counters::of(CounterKind::PlusOne, 1));
        let source = plate(&mut g, p, false);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseArmy {
                object: g.object_ref(a),
            })
        });
        let enemy = g.object_ref(enemy);
        reject(&mut g, Action::ChooseArmy { object: enemy });
        super::contracts::branch_contract(g.clone());
        let saved = g.snapshot();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        drop(g);
        let action = Action::ChooseArmy {
            object: raw.object_ref(a),
        };
        let index = indexed
            .legal_actions()
            .iter()
            .position(|x| *x == action)
            .unwrap();
        act(&mut raw, action.clone());
        indexed.apply_index(p, indexed.revision(), index).unwrap();
        drive(&mut raw, Some(a));
        drive(&mut indexed, Some(a));
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.object_view(source).attached_to, Some(raw.object_ref(a)));
        assert_eq!(stats(&raw, a), (3, 2));
        assert!(raw.has_subtype(a, "Goblin"));
        assert_eq!(stats(&raw, b), (1, 1));
        indexed.restore(&saved).unwrap();
        let object = indexed.object_ref(b);
        act(&mut indexed, Action::ChooseArmy { object });
        drive(&mut indexed, Some(b));
        assert_eq!(
            indexed.object_view(source).attached_to,
            Some(indexed.object_ref(b))
        );
        assert_eq!(stats(&indexed, a), (1, 1));
        assert_eq!(stats(&indexed, b), (3, 2));
    }
}
#[test]
fn hob_plate_company_plate_counter_replacement_commits_attachment_after_owned_work() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let army = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        types(&mut g, army, &["Army"]);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        let source = plate(&mut g, p, true);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Replacement { .. })
        });
        assert_eq!(g.object_view(source).attached_to, None);
        assert_eq!(g.counter_count(army, CounterKind::PlusOne), 0);
        super::contracts::branch_contract(g.clone());
        let saved = g.snapshot();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        drop(g);
        let action = raw
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseReplacement { .. }))
            .unwrap();
        let index = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        act(&mut raw, action.clone());
        indexed.apply_index(p, indexed.revision(), index).unwrap();
        drive(&mut raw, None);
        drive(&mut indexed, None);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.counter_count(army, CounterKind::PlusOne), 4);
        assert_eq!(stats(&raw, army), (6, 5));
        assert_eq!(
            raw.object_view(source).attached_to,
            Some(raw.object_ref(army))
        );
        indexed.restore(&saved).unwrap();
        act(&mut indexed, action);
        drive(&mut indexed, None);
        assert_eq!(raw.snapshot(), indexed.snapshot());
    }
}
#[test]
fn hob_plate_company_plate_token_replacements_and_departed_source_still_amass() {
    for p in [Player::P0, Player::P1] {
        for replace in [false, true] {
            let mut g = scenario(p);
            if replace {
                put(&mut g, p, "Doubling Season", Zone::Battlefield);
            }
            let source = plate(&mut g, p, false);
            until(&mut g, |g| {
                g.state.stack.iter().any(|s| {
                    s.source.is_some_and(|identity| identity.id == source)
                        && matches!(s.kind, StackKind::TriggeredAbility { .. })
                })
            });
            g.move_object(source, Zone::Hand);
            drive(&mut g, None);
            let a = armies(&g, p);
            assert_eq!(a.len(), 1);
            let army = a[0];
            assert_eq!(
                g.counter_count(army, CounterKind::PlusOne),
                if replace { 2 } else { 1 }
            );
            assert_eq!(g.object_view(source).attached_to, None);
            assert!(g.has_subtype(army, "Goblin"));
            assert!(!g.keywords(army).contains(Keywords::MENACE));
        }
    }
}
#[test]
fn hob_plate_company_plate_old_source_incarnation_does_not_attach_returned_equipment() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let a = put(&mut g, p, "@token/goblin_army", Zone::Battlefield);
        g.set_counters(a, Counters::of(CounterKind::PlusOne, 1));
        let b = put(&mut g, p, "@token/goblin_army", Zone::Battlefield);
        g.set_counters(b, Counters::of(CounterKind::PlusOne, 1));
        let source = plate(&mut g, p, false);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectChoice { .. })
        });
        let old = g.object_ref(source);
        g.move_object(source, Zone::Hand);
        g.move_object(source, Zone::Battlefield);
        assert_ne!(old, g.object_ref(source));
        let object = g.object_ref(a);
        act(&mut g, Action::ChooseArmy { object });
        assert_eq!(g.counter_count(a, CounterKind::PlusOne), 2);
        assert_eq!(g.object_view(source).attached_to, None);
        drive(&mut g, Some(b));
        assert_eq!(g.object_view(source).attached_to, Some(g.object_ref(b)));
        assert_eq!(g.counter_count(a, CounterKind::PlusOne), 2);
        assert_eq!(g.counter_count(b, CounterKind::PlusOne), 2);
    }
}
#[test]
fn hob_plate_company_company_live_qualifier_suppression_control_and_used_count() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = company(&mut g, p);
        let first = put(&mut g, p, "Forest", Zone::Hand);
        let second = put(&mut g, p, "Island", Zone::Hand);
        let third = put(&mut g, p, "Forest", Zone::Hand);
        put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        play(&mut g, first, source);
        reject(&mut g, Action::PlayLand { card: second });
        let elf = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        assert!(
            g.legal_actions()
                .contains(&Action::PlayLand { card: second })
        );
        let aura = paid(&mut g, p, "Witness Protection", Some(Target::Card(elf)));
        reject(&mut g, Action::PlayLand { card: second });
        paid(&mut g, p, "Disenchant", Some(Target::Card(aura)));
        assert!(
            g.legal_actions()
                .contains(&Action::PlayLand { card: second })
        );
        let aura = paid(&mut g, p, "Witness Protection", Some(Target::Card(source)));
        reject(&mut g, Action::PlayLand { card: second });
        paid(&mut g, p, "Disenchant", Some(Target::Card(aura)));
        play(&mut g, second, source);
        assert_eq!(g.state.continuous_land_plays_used, 1);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 4);
        g.move_object(elf, Zone::Hand);
        reject(&mut g, Action::PlayLand { card: third });
        g.move_object(elf, Zone::Battlefield);
        reject(&mut g, Action::PlayLand { card: third });
        g.move_object(source, Zone::Hand);
        g.move_object(source, Zone::Battlefield);
        reject(&mut g, Action::PlayLand { card: third });
        let extra = put(&mut g, p, "Thranduil's Company", Zone::Battlefield);
        assert!(
            g.legal_actions()
                .contains(&Action::PlayLand { card: third })
        );
        play(&mut g, third, extra);
        assert_eq!(g.state.continuous_land_plays_used, 2);
        let fourth = put(&mut g, p, "Forest", Zone::Hand);
        g.record_control(g.object_ref(extra), p.other(), None);
        reject(&mut g, Action::PlayLand { card: fourth });
        g.record_control(g.object_ref(extra), p, None);
        reject(&mut g, Action::PlayLand { card: fourth });
    }
}
#[test]
fn hob_plate_company_company_token_noncreature_elf_and_owned_land_allowance() {
    for p in [Player::P0, Player::P1] {
        for token in [false, true] {
            let mut g = scenario(p);
            let source = company(&mut g, p);
            let qualifier = put(
                &mut g,
                p,
                if token {
                    "@token/ca8c0a7df61e962ddae7596f"
                } else {
                    "Heraldic Banner"
                },
                Zone::Battlefield,
            );
            if !token {
                types(&mut g, qualifier, &["Elf"]);
                assert!(!g.is_creature(qualifier));
            }
            let first = put(&mut g, p, "Forest", Zone::Hand);
            let second = put(&mut g, p, "Island", Zone::Hand);
            play(&mut g, first, source);
            assert!(
                g.legal_actions()
                    .contains(&Action::PlayLand { card: second })
            );
            super::contracts::branch_contract(g.clone());
            let saved = g.snapshot();
            let mut raw = g.clone();
            let mut indexed = g.clone();
            drop(g);
            let action = Action::PlayLand { card: second };
            let index = indexed
                .legal_actions()
                .iter()
                .position(|a| *a == action)
                .unwrap();
            act(&mut raw, action.clone());
            indexed.apply_index(p, indexed.revision(), index).unwrap();
            drive(&mut raw, Some(source));
            drive(&mut indexed, Some(source));
            assert_eq!(raw.snapshot(), indexed.snapshot());
            assert_eq!(raw.counter_count(source, CounterKind::PlusOne), 4);
            indexed.restore(&saved).unwrap();
            act(&mut indexed, action);
            drive(&mut indexed, Some(source));
            assert_eq!(raw.snapshot(), indexed.snapshot());
        }
    }
}
#[test]
fn hob_plate_company_company_landfall_targets_replacements_source_loss_and_cleanup() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..5 {
            let mut g = scenario(p);
            let source = company(&mut g, p);
            let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            if mode == 3 {
                put(&mut g, p, "Doubling Season", Zone::Battlefield);
            }
            let enemy_land = put(&mut g, p.other(), "Forest", Zone::Hand);
            g.move_object(enemy_land, Zone::Battlefield);
            g.prepare_triggers();
            drive(&mut g, None);
            assert_eq!(g.counter_count(target, CounterKind::PlusOne), 0);
            let land = put(&mut g, p, "Forest", Zone::Hand);
            act(&mut g, Action::PlayLand { card: land });
            until(&mut g, |g| {
                g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Card(target),
                })
            });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(land),
                },
            );
            bind(&mut g, target);
            if mode == 0 {
                g.move_object(source, Zone::Exile);
            }
            if mode == 1 {
                g.move_object(target, Zone::Hand);
                g.move_object(target, Zone::Battlefield);
            }
            if mode == 2 {
                g.record_control(g.object_ref(target), p.other(), None);
            }
            drive(&mut g, None);
            let expected = match mode {
                1 | 2 => 0,
                3 => 4,
                _ => 2,
            };
            assert_eq!(g.counter_count(target, CounterKind::PlusOne), expected);
            assert_eq!(
                g.keywords(target).contains(Keywords::VIGILANCE),
                expected != 0
            );
            if mode == 4 {
                let incoming = put(&mut g, p, "Island", Zone::Hand);
                g.move_object(incoming, Zone::Battlefield);
                g.prepare_triggers();
                drive(&mut g, Some(target));
                assert_eq!(g.counter_count(target, CounterKind::PlusOne), 4);
                assert!(g.state.land_played);
                assert_eq!(g.state.continuous_land_plays_used, 0);
            }
            if mode == 0 {
                let turn = g.state.turn;
                for _ in 0..64 {
                    if g.state.turn != turn {
                        break;
                    }
                    choose(&mut g, |a| {
                        matches!(
                            a,
                            Action::Pass | Action::FinishAttackers | Action::FinishBlockers
                        )
                    });
                }
                assert_ne!(g.state.turn, turn);
                assert!(!g.keywords(target).contains(Keywords::VIGILANCE));
                assert_eq!(g.counter_count(target, CounterKind::PlusOne), 2);
            }
        }
    }
}
#[test]
fn hob_plate_company_plate_copied_trigger_chooses_its_own_army_then_original_rechooses() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let copier = put(&mut g, p, "Scientist Supreme of A.I.M.", Zone::Battlefield);
        let a = put(&mut g, p, "@token/goblin_army", Zone::Battlefield);
        g.set_counters(a, Counters::of(CounterKind::PlusOne, 1));
        let b = put(&mut g, p, "@token/goblin_army", Zone::Battlefield);
        g.set_counters(b, Counters::of(CounterKind::PlusOne, 1));
        let source = plate(&mut g, p, true);
        until(&mut g, |g| {
            g.state.stack.iter().any(|s| {
                s.source.is_some_and(|identity| identity.id == source)
                    && matches!(s.kind, StackKind::TriggeredAbility { .. })
            })
        });
        let original = g.state.stack.last().unwrap().id;
        g.state.players[p.index()].mana = [0; 6];
        act(
            &mut g,
            Action::AnnounceActivate {
                card: copier,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Stack(original),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].life, 18);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseArmy {
                object: g.object_ref(a),
            })
        });
        let object = g.object_ref(a);
        act(&mut g, Action::ChooseArmy { object });
        assert_eq!(g.counter_count(a, CounterKind::PlusOne), 2);
        assert_eq!(g.object_view(source).attached_to, Some(g.object_ref(a)));
        drive(&mut g, Some(b));
        assert_eq!(g.counter_count(b, CounterKind::PlusOne), 2);
        assert_eq!(g.counter_count(a, CounterKind::PlusOne), 2);
        assert_eq!(g.object_view(source).attached_to, Some(g.object_ref(b)));
    }
}
#[test]
fn hob_plate_company_company_land_allowance_resets_only_on_new_own_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = company(&mut g, p);
        put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        for _ in 0..2 {
            let land = put(&mut g, p, "Forest", Zone::Hand);
            play(&mut g, land, source);
        }
        let third = put(&mut g, p, "Island", Zone::Hand);
        reject(&mut g, Action::PlayLand { card: third });
        let turn = g.state.turn;
        for _ in 0..128 {
            if g.state.turn > turn
                && g.state.active == p
                && g.state.step == Step::Main1
                && quiet(&g)
            {
                break;
            }
            choose(&mut g, |a| {
                matches!(
                    a,
                    Action::Pass | Action::FinishAttackers | Action::FinishBlockers
                )
            });
        }
        assert!(g.state.turn > turn);
        assert_eq!(g.state.active, p);
        assert_eq!(g.state.step, Step::Main1);
        assert_eq!(g.state.continuous_land_plays_used, 0);
        assert!(!g.state.land_played);
        play(&mut g, third, source);
        let fourth = put(&mut g, p, "Forest", Zone::Hand);
        play(&mut g, fourth, source);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 8);
    }
}
