//! Paid complete Strongbox/Redcap scenarios using owned indexed/raw actions and restore.
use super::ability_timeline_tests::{cast, checked, named, next_turn, setup};
use super::*;
fn idle(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.announcing.is_none()
        && g.state.resolving.is_none()
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool, pref: &[Action]) {
    g.prepare_triggers();
    for _ in 0..150 {
        if stop(g) {
            return;
        }
        let legal = g.legal_actions();
        let a = pref
            .iter()
            .find(|a| legal.contains(a))
            .cloned()
            .unwrap_or_else(|| {
                if g.state.pending == Pending::Priority {
                    Action::Pass
                } else {
                    legal
                        .iter()
                        .find(|a| {
                            matches!(
                                a,
                                Action::FinishTargets
                                    | Action::PayCosts { .. }
                                    | Action::PutTrigger { .. }
                                    | Action::DeclineEffectPayment
                            )
                        })
                        .or_else(|| legal.iter().find(|a| !matches!(a, Action::Concede)))
                        .unwrap()
                        .clone()
                }
            });
        checked(g, a);
    }
    panic!("stalled {:?}", g.state.pending)
}
fn settle(g: &mut Game, pref: &[Action]) {
    until(g, idle, pref)
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(before, g.snapshot());
    assert_eq!(rev, g.revision());
}
fn attack(g: &mut Game, id: CardId) {
    g.state.objects[id as usize].summoning_sick = false;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    checked(g, Action::Attack { card: id });
    checked(g, Action::FinishAttackers);
    settle(g, &[]);
}
fn raid(g: &mut Game, p: Player) {
    let bear = named(g, p, "Bear Cub", Zone::Battlefield);
    attack(g, bear);
    g.state.step = Step::Main2;
    g.state.priority = p;
    g.state.passes = 0;
}
fn strongbox(g: &mut Game, p: Player) -> CardId {
    raid(g, p);
    let id = named(g, p, "Strongbox Raider", Zone::Hand);
    cast(g, id, None, &[]);
    until(
        g,
        |g| matches!(g.state.pending, Pending::MilledSelection { .. }),
        &[],
    );
    id
}
fn choose(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    checked(g, Action::SelectMilled { object });
    settle(g, &[]);
}
fn rats(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| {
            o.zone == Zone::Battlefield
                && o.controller == p
                && o.card
                    .token_info()
                    .is_some_and(|t| t.subtypes == "Rat" && t.colors == 4)
        })
        .map(|(id, _)| id as CardId)
        .collect()
}

#[test]
fn fdn_exile_two_strongbox_paid_raid_gates_mandatory_one_choice_and_public_exile() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let top = named(&mut g, p, "Shock", Zone::Library);
        let source = named(&mut g, p, "Strongbox Raider", Zone::Hand);
        let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
        cast(&mut g, source, None, &[]);
        settle(&mut g, &[]);
        assert_eq!(g.object_view(top).zone, Zone::Library);
        assert!(g.state.exile_permissions.is_empty());
        assert_eq!(
            mana - g.state.players[p.index()].mana.iter().sum::<u32>(),
            4
        );
        assert_eq!(
            g.stats(source),
            Stats {
                power: 5,
                toughness: 2
            }
        );
        let mut g = setup(p);
        let other = named(&mut g, p, "Forest", Zone::Library);
        let chosen = named(&mut g, p, "Shock", Zone::Library);
        let outside = named(&mut g, p, "Think Twice", Zone::Exile);
        let source = strongbox(&mut g, p);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        assert_eq!(g.object_view(chosen).zone, Zone::Exile);
        assert_eq!(g.object_view(other).zone, Zone::Exile);
        assert!(g.state.exile_permissions.is_empty());
        reject(&mut g, Action::FinishMilledSelection);
        let object = g.object_ref(outside);
        reject(&mut g, Action::SelectMilled { object });
        assert_eq!(
            g.observe(p)
                .milled_choice
                .as_ref()
                .unwrap()
                .candidates
                .len(),
            2
        );
        assert!(g.observe(p.other()).milled_choice.is_none());
        choose(&mut g, chosen);
        assert!(g.can_play_exiled(p, chosen));
        assert!(!g.can_play_exiled(p, other));
        assert!(!g.can_play_exiled(p.other(), chosen));
        assert_eq!(
            g.observe(p).exile_permissions,
            g.observe(p.other()).exile_permissions
        );
    }
}
#[test]
fn fdn_exile_two_strongbox_selected_spell_requires_normal_mana_and_target_legality() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let _other = named(&mut g, p, "Forest", Zone::Library);
        let shock = named(&mut g, p, "Shock", Zone::Library);
        strongbox(&mut g, p);
        choose(&mut g, shock);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnounceExile { card: shock });
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        let life = g.state.players[p.other().index()].life;
        checked(&mut g, Action::AnnounceExile { card: shock });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        until(&mut g, |g| g.state.announcing.is_none(), &[]);
        settle(&mut g, &[]);
        assert_eq!(g.state.players[p.other().index()].life, life - 2);
        assert_eq!(g.object_view(shock).zone, Zone::Graveyard);
        assert!(!g.can_play_exiled(p, shock));
    }
}
#[test]
fn fdn_exile_two_strongbox_selected_land_respects_turn_main_phase_and_land_limit() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let _other = named(&mut g, p, "Shock", Zone::Library);
        let land = named(&mut g, p, "Forest", Zone::Library);
        strongbox(&mut g, p);
        choose(&mut g, land);
        g.state.step = Step::BeginCombat;
        reject(&mut g, Action::PlayLand { card: land });
        g.state.step = Step::Main2;
        let played = named(&mut g, p, "Forest", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: played });
        reject(&mut g, Action::PlayLand { card: land });
        next_turn(&mut g);
        assert!(g.can_play_exiled(p, land));
        next_turn(&mut g);
        g.state.step = Step::Main1;
        g.state.priority = p;
        checked(&mut g, Action::PlayLand { card: land });
        assert_eq!(g.object_view(land).zone, Zone::Battlefield);
        assert_eq!(g.object_view(land).controller, p);
    }
}
#[test]
fn fdn_exile_two_strongbox_permission_survives_source_and_expires_after_next_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let _other = named(&mut g, p, "Forest", Zone::Library);
        let shock = named(&mut g, p, "Shock", Zone::Library);
        let source = strongbox(&mut g, p);
        choose(&mut g, shock);
        let deadline = g.state.turn + 2;
        assert_eq!(
            g.state
                .exile_permissions
                .iter()
                .find(|v| v.object.card == shock)
                .unwrap()
                .expires_turn,
            deadline
        );
        g.move_object(source, Zone::Graveyard);
        next_turn(&mut g);
        assert!(g.can_play_exiled(p, shock));
        next_turn(&mut g);
        assert_eq!(g.state.turn, deadline);
        assert!(g.can_play_exiled(p, shock));
        next_turn(&mut g);
        assert!(!g.can_play_exiled(p, shock));
        assert_eq!(g.object_view(shock).zone, Zone::Exile);
    }
}
#[test]
fn fdn_exile_two_strongbox_zero_one_top_cards_and_exact_exile_incarnation() {
    for p in [Player::P0, Player::P1] {
        for amount in [0, 1] {
            let mut g = setup(p);
            let library = g.state.players[p.index()].library.clone();
            for id in library {
                g.move_object(id, Zone::Graveyard);
            }
            let top = (amount == 1).then(|| named(&mut g, p, "Shock", Zone::Library));
            raid(&mut g, p);
            let source = named(&mut g, p, "Strongbox Raider", Zone::Hand);
            cast(&mut g, source, None, &[]);
            if let Some(top) = top {
                until(
                    &mut g,
                    |g| matches!(g.state.pending, Pending::MilledSelection { .. }),
                    &[],
                );
                choose(&mut g, top);
                let old = g.object_ref(top);
                g.move_object(top, Zone::Graveyard);
                g.move_object(top, Zone::Exile);
                assert_ne!(old, g.object_ref(top));
                assert!(!g.can_play_exiled(p, top));
            } else {
                settle(&mut g, &[]);
                assert!(g.state.exile_permissions.is_empty());
            }
        }
    }
}
#[test]
fn fdn_exile_two_redcap_paid_body_two_black_rats_cannot_block_and_menace() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Redcap Gutter-Dweller", Zone::Hand);
        let before = g.state.players[p.index()].mana.iter().sum::<u32>();
        cast(&mut g, source, None, &[]);
        settle(&mut g, &[]);
        assert_eq!(
            before - g.state.players[p.index()].mana.iter().sum::<u32>(),
            4
        );
        let spawned = rats(&g, p);
        assert_eq!(spawned.len(), 2);
        assert!(g.keywords(source).contains(Keywords::MENACE));
        for id in &spawned {
            assert_eq!(
                g.stats(*id),
                Stats {
                    power: 1,
                    toughness: 1
                }
            );
            assert!(g.keywords(*id).contains(Keywords::CANT_BLOCK));
        }
        let b1 = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let b2 = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        attack(&mut g, source);
        g.state.pending = Pending::Blockers;
        g.state.step = Step::Blockers;
        checked(
            &mut g,
            Action::Block {
                attacker: source,
                blocker: b1,
            },
        );
        reject(&mut g, Action::FinishBlockers);
        checked(
            &mut g,
            Action::Block {
                attacker: source,
                blocker: b2,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        let mut g = setup(p);
        let source = named(&mut g, p, "Redcap Gutter-Dweller", Zone::Hand);
        cast(&mut g, source, None, &[]);
        settle(&mut g, &[]);
        let rat = rats(&g, p)[0];
        let attacker = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.active = p.other();
        g.state.priority = p.other();
        attack(&mut g, attacker);
        g.state.pending = Pending::Blockers;
        g.state.step = Step::Blockers;
        reject(
            &mut g,
            Action::Block {
                attacker,
                blocker: rat,
            },
        );
    }
}
#[test]
fn fdn_exile_two_redcap_upkeep_optional_other_sacrifice_counter_and_current_turn_permission() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            let mut g = setup(p);
            let source = named(&mut g, p, "Redcap Gutter-Dweller", Zone::Hand);
            cast(&mut g, source, None, &[]);
            settle(&mut g, &[]);
            let rat = rats(&g, p)[0];
            let top = named(&mut g, p, "Shock", Zone::Library);
            g.begin_step(Step::Upkeep);
            until(
                &mut g,
                |g| g.legal_actions().contains(&Action::DeclineEffectPayment),
                &[],
            );
            let object = g.object_ref(source);
            reject(&mut g, Action::Sacrifice { object });
            let object = g.object_ref(rat);
            checked(
                &mut g,
                if pay {
                    Action::Sacrifice { object }
                } else {
                    Action::DeclineEffectPayment
                },
            );
            settle(&mut g, &[]);
            assert_eq!(
                g.object_view(source).counters.plus_one,
                if pay { 1 } else { 0 }
            );
            assert_eq!(
                g.object_view(top).zone,
                if pay { Zone::Exile } else { Zone::Library }
            );
            assert_eq!(g.can_play_exiled(p, top), pay);
            if pay {
                assert_eq!(
                    g.state
                        .exile_permissions
                        .iter()
                        .find(|v| v.object.card == top)
                        .unwrap()
                        .expires_turn,
                    g.state.turn
                );
                g.move_object(source, Zone::Graveyard);
                next_turn(&mut g);
                assert!(!g.can_play_exiled(p, top));
            }
        }
    }
}
#[test]
fn fdn_exile_two_redcap_upkeep_source_departure_still_exiles_and_foreign_upkeep_does_not_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Redcap Gutter-Dweller", Zone::Hand);
        cast(&mut g, source, None, &[]);
        settle(&mut g, &[]);
        let rat = rats(&g, p)[0];
        let top = named(&mut g, p, "Shock", Zone::Library);
        g.state.active = p.other();
        g.begin_step(Step::Upkeep);
        assert!(g.state.pending_triggers.is_empty());
        g.state.active = p;
        g.state.priority = p;
        g.begin_step(Step::Upkeep);
        until(
            &mut g,
            |g| g.legal_actions().contains(&Action::DeclineEffectPayment),
            &[],
        );
        g.move_object(source, Zone::Graveyard);
        let object = g.object_ref(rat);
        checked(&mut g, Action::Sacrifice { object });
        settle(&mut g, &[]);
        assert_eq!(g.object_view(source).counters.plus_one, 0);
        assert_eq!(g.object_view(top).zone, Zone::Exile);
        assert!(g.can_play_exiled(p, top));
    }
}
