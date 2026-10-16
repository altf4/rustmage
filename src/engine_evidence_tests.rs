use super::tests::{act, scenario, spawn};
use super::*;
const NAMES: [&str; 10] = [
    "Analyze the Pollen",
    "Axebane Ferox",
    "Bite Down on Crime",
    "Crimestopper Sprite",
    "Evidence Examiner",
    "Extract a Confession",
    "Forensic Researcher",
    "Sample Collector",
    "Surveillance Monitor",
    "Vitu-Ghazi Inspector",
];
fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
    let c = Card::from_name(n).unwrap();
    assert!(c.is_supported(), "{n}");
    spawn(g, p, c, z)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for s in &mut g.state.players {
        s.mana = [40; 6];
    }
    g
}
fn grave(g: &mut Game, p: Player) {
    named(g, p, "Hamlet Glutton", Zone::Graveyard);
    named(g, p, "Hamlet Glutton", Zone::Graveyard);
}
fn replay(g: &mut Game, a: Action) {
    assert!(
        g.legal_actions().contains(&a),
        "{a:?} {:?}",
        g.state.pending
    );
    let before = g.snapshot();
    let rev = g.revision();
    let p = g.actor().unwrap();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let after = g.snapshot();
    assert_eq!(fork.snapshot(), after);
    g.restore(&before).unwrap();
    assert_eq!(g.apply_index(p, rev, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), after);
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
}
fn pick(g: &mut Game, collect: bool, targets: &[CardId]) {
    let aa = g.legal_actions();
    if matches!(g.state.pending, Pending::Evidence { .. }) && g.state.announcing.is_none() {
        assert!(
            !aa.iter()
                .any(|a| matches!(a, Action::TapMana { .. } | Action::ActivateMana { .. }))
        );
    }
    let a=aa.iter().find(|a|matches!(a,Action::ChooseCollectEvidence{collect:c}if *c==collect))
 .or_else(||aa.iter().find(|a|matches!(a,Action::ChooseTarget{target:Target::Card(id)}if targets.contains(id))))
 .or_else(||aa.iter().find(|a|matches!(a,Action::FinishEvidence)))
 .or_else(||aa.iter().filter(|a|matches!(a,Action::ChooseEvidence{..})).max_by_key(|a|if let Action::ChooseEvidence{object}=a {g.state.objects[object.card as usize].card.fixed_mana_value()}else{0}))
 .or_else(||aa.iter().find(|a|matches!(a,Action::ChooseSearch{..})))
 .or_else(||aa.iter().find(|a|matches!(a,Action::Sacrifice{..})))
 .or_else(||aa.iter().find(|a|matches!(a,Action::PayCosts{..}|Action::PutTrigger{..}|Action::FinishTargets|Action::FinishSearch|Action::Pass)))
 .unwrap_or_else(||panic!("no continuation {:?}: {aa:?}",g.state.pending));
    replay(g, a.clone());
}
fn idle(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.announcing.is_none()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
}
fn settle(g: &mut Game, collect: bool, targets: &[CardId]) {
    for _ in 0..160 {
        if idle(g) {
            return;
        }
        pick(g, collect, targets);
    }
    panic!("unsettled {:?}", g.state.pending)
}
fn cast(g: &mut Game, id: CardId, collect: bool, targets: &[CardId]) {
    replay(g, Action::AnnounceCast { card: id });
    for _ in 0..80 {
        if g.state.announcing.is_none() {
            return;
        }
        pick(g, collect, targets);
    }
    panic!("cast stuck")
}
#[test]
fn all_ten_complete_casts_and_residuals_both_declarations_and_seats() {
    for p in [Player::P0, Player::P1] {
        for collect in [false, true] {
            for (i, n) in NAMES.iter().enumerate() {
                let mut g = setup(p);
                grave(&mut g, p);
                let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                let foe = named(&mut g, p.other(), "Hamlet Glutton", Zone::Battlefield);
                let src = named(&mut g, p, n, Zone::Hand);
                let life = g.state.players[p.index()].life;
                cast(&mut g, src, collect, &[own, foe]);
                let spell = g
                    .state
                    .stack
                    .iter()
                    .find(|s| s.card == src && s.kind == StackKind::Spell)
                    .unwrap();
                assert_eq!(
                    spell.cast.unwrap().evidence,
                    if spell.source_card.collect_evidence_cost().is_some() {
                        Some(collect)
                    } else {
                        None
                    }
                );
                settle(&mut g, collect, &[own, foe]);
                if [0, 2, 5].contains(&i) {
                    assert_eq!(g.object_view(src).zone, Zone::Graveyard)
                } else {
                    assert_eq!(g.object_view(src).zone, Zone::Battlefield)
                }
                match i {
                    0 => {
                        assert_eq!(g.hand_count(p), 1);
                        assert!(g.observe(p).known_hand.len() == 1);
                    }
                    1 => {
                        let v = g.object_view(src);
                        assert_eq!((v.power, v.toughness), (4, 4));
                        assert!(v.keywords.contains(Keywords::HASTE));
                        assert!(v.keywords.contains(Keywords::DEATHTOUCH));
                    }
                    2 => {
                        assert_eq!(g.object_view(own).power, 4);
                        assert_eq!(g.object_view(foe).damage, 4);
                    }
                    3 => {
                        assert!(g.object_view(own).tapped);
                        assert_eq!(g.object_view(own).counters.stun, u64::from(collect));
                    }
                    5 => assert_eq!(g.object_view(foe).zone, Zone::Graveyard),
                    8 => {
                        assert!(g.state.evidence_data.as_ref().unwrap().last.is_some());
                        assert!(g.state.objects.iter().any(|o| o.owner == p
                            && o.zone == Zone::Battlefield
                            && o.card.is_token()
                            && o.card.has_subtype("Thopter")));
                    }
                    9 => {
                        assert_eq!(g.object_view(own).counters.plus_one, u64::from(collect));
                        assert_eq!(
                            g.state.players[p.index()].life,
                            life + if collect { 2 } else { 0 }
                        );
                    }
                    _ => {}
                }
            }
        }
    }
}
#[test]
fn optional_intent_precedes_targets_exact_subset_stays_deferred_and_zero_extras_remain_legal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = named(&mut g, p, "Hamlet Glutton", Zone::Graveyard);
        let b = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        let zero = named(&mut g, p, "Forest", Zone::Graveyard);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let src = named(&mut g, p, "Bite Down on Crime", Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: src });
        assert_eq!(g.observe(p).decision, "evidence_intent");
        assert!(g.observe(p).payment_request.is_none());
        assert!(g.state.announcing.as_ref().unwrap().item.targets.is_empty());
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        let snap = g.snapshot();
        assert_eq!(
            g.apply(p.other(), Action::ChooseCollectEvidence { collect: true }),
            Err(Error::WrongActor)
        );
        assert_eq!(g.snapshot(), snap);
        replay(&mut g, Action::ChooseCollectEvidence { collect: true });
        while !matches!(g.state.pending, Pending::Evidence { .. }) {
            pick(&mut g, true, &[own, foe]);
        }
        let mana = g.state.players[p.index()].mana;
        assert!(g.observe(p.other()).evidence.is_none());
        assert!(g.observe(p.other()).actions.is_empty());
        reject(&mut g, Action::FinishEvidence);
        let old = g.object_ref(a);
        replay(&mut g, Action::ChooseEvidence { object: old });
        assert_eq!(g.object_view(a).zone, Zone::Graveyard);
        assert_eq!(g.object_ref(a), old);
        assert_eq!(g.state.players[p.index()].mana, mana);
        assert!(g.state.evidence_data.is_none());
        assert!(g.legal_actions().contains(&Action::FinishEvidence));
        reject(&mut g, Action::ChooseEvidence { object: old });
        reject(
            &mut g,
            Action::ChooseEvidence {
                object: ObjectRef {
                    generation: old.generation + 1,
                    ..old
                },
            },
        );
        let zr = g.object_ref(zero);
        replay(&mut g, Action::ChooseEvidence { object: zr });
        assert_eq!(g.observe(p).evidence.unwrap().total_mana, 7);
        assert!(g.legal_actions().contains(&Action::ChooseEvidence {
            object: g.object_ref(b)
        }));
        replay(&mut g, Action::FinishEvidence);
        assert_eq!(g.object_view(a).zone, Zone::Graveyard);
        let req = g.observe(p).payment_request.unwrap();
        assert_eq!((req.required, req.generic), ([0, 0, 0, 0, 1, 0], 1));
        pick(&mut g, true, &[own, foe]);
        assert_eq!(g.object_view(a).zone, Zone::Exile);
        assert_eq!(g.object_view(zero).zone, Zone::Exile);
        assert_eq!(g.object_view(b).zone, Zone::Graveyard);
        let e = g.observe(p).last_evidence.unwrap();
        assert_eq!((e.threshold, e.total_mana, e.exiled.len()), (6, 7, 2));
        assert_eq!(e.exiled, vec![g.object_ref(a), g.object_ref(zero)]);
    }
}
#[test]
fn insufficient_grave_cannot_declare_and_funding_queries_are_pure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Bear Cub", Zone::Battlefield);
        named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        named(&mut g, p, "Forest", Zone::Graveyard);
        let id = named(&mut g, p, "Bite Down on Crime", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 2, 0];
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: id })
        );
        grave(&mut g, p);
        let before = g.snapshot();
        for _ in 0..10 {
            assert!(
                g.legal_actions()
                    .contains(&Action::AnnounceCast { card: id })
            );
        }
        assert_eq!(g.snapshot(), before);
        replay(&mut g, Action::AnnounceCast { card: id });
        assert!(
            !g.legal_actions()
                .contains(&Action::ChooseCollectEvidence { collect: false })
        );
    }
}
#[test]
fn greatest_power_choice_uses_live_signed_power_and_every_tie() {
    for p in [Player::P0, Player::P1] {
        for collect in [false, true] {
            let mut g = setup(p);
            grave(&mut g, p);
            let low = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let a = named(&mut g, p.other(), "Hamlet Glutton", Zone::Battlefield);
            let b = named(&mut g, p.other(), "Hamlet Glutton", Zone::Battlefield);
            for (id, n) in [(low, -12), (a, -10), (b, -10)] {
                g.state.objects[id as usize].growth.power = n;
            }
            let id = named(&mut g, p, "Extract a Confession", Zone::Hand);
            cast(&mut g, id, collect, &[]);
            while !matches!(g.state.pending, Pending::EffectChoice { .. }) {
                pick(&mut g, collect, &[]);
            }
            let aa = g.legal_actions();
            assert!(aa.contains(&Action::Sacrifice {
                object: g.object_ref(a)
            }));
            assert!(aa.contains(&Action::Sacrifice {
                object: g.object_ref(b)
            }));
            assert_eq!(
                aa.contains(&Action::Sacrifice {
                    object: g.object_ref(low)
                }),
                !collect
            );
            let low_ref = g.object_ref(low);
            let b_ref = g.object_ref(b);
            if collect {
                reject(&mut g, Action::Sacrifice { object: low_ref });
            }
            replay(&mut g, Action::Sacrifice { object: b_ref });
            assert_eq!(g.object_view(b).zone, Zone::Graveyard);
            assert_eq!(g.object_view(a).zone, Zone::Battlefield);
        }
    }
}
#[test]
fn activated_evidence_is_mandatory_after_targets_with_real_tap_and_no_self_untap() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let src = named(&mut g, p, "Forensic Researcher", Zone::Battlefield);
        let own = named(&mut g, p, "Candy Trail", Zone::Battlefield);
        let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let activate = Action::AnnounceActivate {
            card: src,
            ability: 1,
            counter_payment: Counters::default(),
        };
        assert!(!g.legal_actions().contains(&activate));
        grave(&mut g, p);
        replay(&mut g, activate);
        assert!(matches!(g.state.pending, Pending::Targets { .. }));
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(foe),
            },
        );
        while !matches!(g.state.pending, Pending::Evidence { .. }) {
            pick(&mut g, true, &[foe]);
        }
        reject(&mut g, Action::DeclineEvidence);
        assert!(!g.object_view(src).tapped);
        while g.state.announcing.is_some() {
            pick(&mut g, true, &[foe]);
        }
        assert!(g.object_view(src).tapped);
        assert!(!g.object_view(foe).tapped);
        settle(&mut g, true, &[foe]);
        assert!(g.object_view(foe).tapped);
        g.state.objects[src as usize].tapped = false;
        g.state.objects[own as usize].tapped = true;
        replay(
            &mut g,
            Action::AnnounceActivate {
                card: src,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(src),
            },
        );
        while g.state.announcing.is_some() {
            pick(&mut g, false, &[own]);
        }
        settle(&mut g, false, &[own]);
        assert!(!g.object_view(own).tapped);
    }
}
#[test]
fn ward_payer_is_related_controller_with_real_decline_payment_and_departed_source() {
    for p in [Player::P0, Player::P1] {
        for pay in [false, true] {
            for ability in [false, true] {
                let mut g = setup(p);
                grave(&mut g, p);
                let examiner = named(&mut g, p, "Evidence Examiner", Zone::Battlefield);
                let foe_examiner = named(&mut g, p.other(), "Evidence Examiner", Zone::Battlefield);
                let ward = named(&mut g, p.other(), "Axebane Ferox", Zone::Battlefield);
                let source = if ability {
                    let id = named(&mut g, p, "Forensic Researcher", Zone::Battlefield);
                    replay(
                        &mut g,
                        Action::AnnounceActivate {
                            card: id,
                            ability: 1,
                            counter_payment: Counters::default(),
                        },
                    );
                    while g.state.announcing.is_some() {
                        pick(&mut g, true, &[ward]);
                    }
                    id
                } else {
                    let id = named(&mut g, p, "Shock", Zone::Hand);
                    cast(&mut g, id, false, &[ward]);
                    id
                };
                let victim = g.state.stack.iter().find(|s| s.card == source).unwrap().id;
                g.move_object(ward, Zone::Graveyard);
                for _ in 0..80 {
                    if matches!(g.state.pending, Pending::Evidence { .. }) {
                        break;
                    }
                    pick(&mut g, true, &[ward]);
                }
                assert!(matches!(g.state.pending,Pending::Evidence{player}if player==p));
                assert_eq!(g.observe(p).evidence.unwrap().threshold, 4);
                assert!(g.observe(p.other()).evidence.is_none());
                assert!(g.state.stack.iter().any(|s| s.id == victim));
                if pay {
                    while matches!(g.state.pending, Pending::Evidence { .. }) {
                        pick(&mut g, true, &[]);
                    }
                    assert!(g.state.stack.iter().any(|s| s.id == victim));
                    assert_eq!(g.observe(p).last_evidence.unwrap().player, p);
                } else {
                    replay(&mut g, Action::DeclineEvidence);
                    assert!(!g.state.stack.iter().any(|s| s.id == victim));
                }
                settle(&mut g, true, &[]);
                let clues = |owner| {
                    g.state
                        .objects
                        .iter()
                        .filter(|o| {
                            o.zone == Zone::Battlefield
                                && o.owner == owner
                                && o.card.is_token()
                                && o.card.has_subtype("Clue")
                        })
                        .count()
                };
                assert_eq!(clues(p), usize::from(ability) + usize::from(pay));
                assert_eq!(clues(p.other()), 0);
                assert_eq!(g.object_view(examiner).zone, Zone::Battlefield);
                assert_eq!(g.object_view(foe_examiner).zone, Zone::Battlefield);
            }
        }
    }
}
#[test]
fn ward_with_insufficient_evidence_counters_without_partial_exiles() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let small = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        let ward = named(&mut g, p.other(), "Axebane Ferox", Zone::Battlefield);
        let id = named(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, id, false, &[ward]);
        settle(&mut g, false, &[ward]);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        assert_eq!(g.object_view(small).zone, Zone::Graveyard);
        assert!(g.state.evidence_data.is_none());
        assert_eq!(g.object_view(ward).damage, 0);
    }
}
#[test]
fn optional_decline_after_partial_selection_emits_nothing_and_preserves_all_cards() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        grave(&mut g, p);
        let id = named(&mut g, p, "Surveillance Monitor", Zone::Hand);
        cast(&mut g, id, false, &[]);
        while !matches!(g.state.pending, Pending::Evidence { .. }) {
            pick(&mut g, false, &[]);
        }
        let before = g.snapshot();
        let card = g.observe(p).evidence.unwrap().candidates[0];
        replay(&mut g, Action::ChooseEvidence { object: card });
        assert_eq!(g.object_ref(card.card), card);
        replay(&mut g, Action::DeclineEvidence);
        settle(&mut g, false, &[]);
        assert!(g.observe(p).last_evidence.is_none());
        assert_eq!(g.object_view(card.card).zone, Zone::Graveyard);
        assert!(
            !g.state
                .objects
                .iter()
                .any(|o| o.zone == Zone::Battlefield && o.card.is_token())
        );
        let mut fork = g.clone();
        fork.restore(&before).unwrap();
        settle(&mut fork, true, &[]);
        assert!(fork.observe(p).last_evidence.is_some());
        assert!(g.observe(p).last_evidence.is_none());
    }
}
#[test]
fn examiner_combat_and_monitor_observe_true_collections_after_paying_source_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        grave(&mut g, p);
        let source = named(&mut g, p, "Evidence Examiner", Zone::Battlefield);
        let monitor = named(&mut g, p, "Surveillance Monitor", Zone::Battlefield);
        while g.state.step != Step::BeginCombat {
            replay(&mut g, Action::Pass);
        }
        while !matches!(g.state.pending, Pending::Evidence { .. }) {
            pick(&mut g, true, &[]);
        }
        assert_eq!(g.state.resolving.as_ref().unwrap().item.card, source);
        let old = g.object_ref(source);
        g.move_object(source, Zone::Graveyard);
        settle(&mut g, true, &[]);
        let event = g.observe(p).last_evidence.unwrap();
        assert_eq!(event.source, old);
        assert!(
            !g.state
                .evidence_data
                .as_ref()
                .unwrap()
                .sources
                .contains(&source)
        );
        assert!(
            g.state
                .evidence_data
                .as_ref()
                .unwrap()
                .sources
                .contains(&monitor)
        );
        let thopter = g
            .state
            .objects
            .iter()
            .position(|o| {
                o.owner == p
                    && o.zone == Zone::Battlefield
                    && o.card.is_token()
                    && o.card.has_subtype("Thopter")
            })
            .unwrap() as CardId;
        let v = g.object_view(thopter);
        assert_eq!((v.power, v.toughness), (1, 1));
        assert!(v.keywords.contains(Keywords::FLYING));
        assert!(v.card.is_artifact());
        assert!(
            !g.state
                .objects
                .iter()
                .any(|o| o.zone == Zone::Battlefield && o.card.has_subtype("Clue"))
        );
    }
}
#[test]
fn sample_reflexive_targets_are_chosen_after_collection_and_survive_source_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        grave(&mut g, p);
        let source = named(&mut g, p, "Sample Collector", Zone::Battlefield);
        let friend = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        while g.state.step != Step::Attackers {
            replay(&mut g, Action::Pass);
        }
        replay(&mut g, Action::Attack { card: source });
        replay(&mut g, Action::FinishAttackers);
        while !matches!(g.state.pending, Pending::Evidence { .. }) {
            pick(&mut g, true, &[friend]);
        }
        assert!(g.state.resolving.as_ref().unwrap().item.targets.is_empty());
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseTarget { .. }))
        );
        g.move_object(source, Zone::Graveyard);
        while !g
            .state
            .announcing
            .as_ref()
            .is_some_and(|a| a.item.kind == StackKind::TriggeredAbility { ability: 1 })
        {
            pick(&mut g, true, &[friend]);
        }
        assert!(g.observe(p).last_evidence.is_some());
        assert!(g.state.announcing.as_ref().unwrap().item.targets.is_empty());
        settle(&mut g, true, &[friend]);
        assert_eq!(g.object_view(friend).counters.plus_one, 1);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    }
}
#[test]
fn historical_sprite_and_inspector_entry_qualification_survives_old_source_not_new_incarnation() {
    for p in [Player::P0, Player::P1] {
        for n in ["Crimestopper Sprite", "Vitu-Ghazi Inspector"] {
            let mut g = setup(p);
            grave(&mut g, p);
            let friend = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let id = named(&mut g, p, n, Zone::Hand);
            let life = g.state.players[p.index()].life;
            cast(&mut g, id, true, &[friend]);
            while !g.state.announcing.as_ref().is_some_and(|a| a.trigger) {
                pick(&mut g, true, &[friend]);
            }
            let old = g.object_ref(id);
            assert!(g.state.announcing.as_ref().unwrap().item.evidence_collected);
            g.move_object(id, Zone::Exile);
            g.move_object(id, Zone::Battlefield);
            assert_ne!(g.object_ref(id), old);
            settle(&mut g, true, &[friend]);
            if n == "Crimestopper Sprite" {
                assert_eq!(g.object_view(friend).counters.stun, 1);
                assert!(g.object_view(friend).tapped);
            } else {
                assert_eq!(g.object_view(friend).counters.plus_one, 1);
                assert_eq!(g.state.players[p.index()].life, life + 2);
            }
        }
    }
}
#[test]
fn entry_suppression_blocks_residual_triggers_but_not_the_paid_collection_event() {
    for p in [Player::P0, Player::P1] {
        for n in ["Crimestopper Sprite", "Vitu-Ghazi Inspector"] {
            let mut g = setup(p);
            grave(&mut g, p);
            named(&mut g, p, "Evidence Examiner", Zone::Battlefield);
            named(&mut g, p, "Torpor Orb", Zone::Battlefield);
            let friend = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let id = named(&mut g, p, n, Zone::Hand);
            let life = g.state.players[p.index()].life;
            cast(&mut g, id, true, &[friend]);
            settle(&mut g, true, &[friend]);
            assert_eq!(g.object_view(friend).counters, Counters::default());
            assert!(!g.object_view(friend).tapped);
            assert_eq!(g.state.players[p.index()].life, life);
            let clue = g
                .state
                .objects
                .iter()
                .position(|o| {
                    o.zone == Zone::Battlefield && o.card.is_token() && o.card.has_subtype("Clue")
                })
                .unwrap() as CardId;
            let before = g.hand_count(p);
            replay(
                &mut g,
                Action::AnnounceActivate {
                    card: clue,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            while g.state.announcing.is_some() {
                pick(&mut g, true, &[]);
            }
            assert_eq!(g.object_view(clue).zone, Zone::Outside);
            settle(&mut g, true, &[]);
            assert_eq!(g.hand_count(p), before + 1);
        }
    }
}
#[test]
fn late_cast_and_resolution_collection_failures_restore_every_owned_transaction() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        grave(&mut g, p);
        named(&mut g, p, "Evidence Examiner", Zone::Battlefield);
        let id = named(&mut g, p, "Vitu-Ghazi Inspector", Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: id });
        while !matches!(g.state.pending, Pending::PayCosts { .. }) {
            pick(&mut g, true, &[]);
        }
        g.state.players[p.index()].spells_cast_this_turn = u64::MAX;
        let old = g.snapshot();
        let rev = g.revision();
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        assert!(matches!(g.apply(p, pay), Err(Error::CapacityExceeded(_))));
        assert_eq!(g.snapshot(), old);
        assert_eq!(g.revision(), rev);
        assert!(g.observe(p).last_evidence.is_none());
        let mut g = setup(p);
        grave(&mut g, p);
        let id = named(&mut g, p, "Surveillance Monitor", Zone::Hand);
        cast(&mut g, id, false, &[]);
        while !matches!(g.state.pending, Pending::Evidence { .. }) {
            pick(&mut g, true, &[]);
        }
        let first = g.observe(p).evidence.unwrap().candidates[0];
        replay(&mut g, Action::ChooseEvidence { object: first });
        g.state.evidence_data.as_mut().unwrap().sequence = u64::MAX;
        let old = g.snapshot();
        let rev = g.revision();
        assert!(matches!(
            g.apply(p, Action::FinishEvidence),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), old);
        assert_eq!(g.revision(), rev);
        assert_eq!(g.object_ref(first.card), first);
        assert_eq!(g.object_view(first.card).zone, Zone::Graveyard);
    }
}
#[test]
fn selection_observer_history_allocations_are_owned_counted_and_rollback_exactly() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        grave(&mut g, p);
        let id = named(&mut g, p, "Surveillance Monitor", Zone::Hand);
        let initial = g.state_bytes();
        cast(&mut g, id, false, &[]);
        while !matches!(g.state.pending, Pending::Evidence { .. }) {
            pick(&mut g, true, &[]);
        }
        let before = g.state_bytes();
        let saved = g.snapshot();
        let first = g.observe(p).evidence.unwrap().candidates[0];
        replay(&mut g, Action::ChooseEvidence { object: first });
        assert_eq!(g.state_bytes(), before + std::mem::size_of::<ObjectRef>());
        g.restore(&saved).unwrap();
        assert_eq!(g.state_bytes(), before);
        settle(&mut g, true, &[]);
        let after = g.state_bytes();
        assert!(g.observe(p).last_evidence.is_some());
        g.restore(&saved).unwrap();
        assert_eq!(g.state_bytes(), before);
        assert!(g.observe(p).last_evidence.is_none());
        eprintln!(
            "evidence sizes selection={} eventstate={} object={} initialfixture={initial} active={before} complete={after}",
            std::mem::size_of::<evidence::EvidenceSelection>(),
            std::mem::size_of::<evidence::EvidenceState>(),
            std::mem::size_of::<Object>()
        );
    }
}
#[test]
fn bite_group_zero_only_growth_and_each_partial_illegal_target_resolve_independently() {
    for p in [Player::P0, Player::P1] {
        for missing in 0..4 {
            let mut g = setup(p);
            grave(&mut g, p);
            let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let foe = named(&mut g, p.other(), "Hamlet Glutton", Zone::Battlefield);
            let id = named(&mut g, p, "Bite Down on Crime", Zone::Hand);
            cast(&mut g, id, true, &[own, foe]);
            if missing & 1 != 0 {
                g.move_object(own, Zone::Graveyard);
            }
            if missing & 2 != 0 {
                g.move_object(foe, Zone::Graveyard);
            }
            settle(&mut g, true, &[own, foe]);
            if missing & 1 == 0 {
                assert_eq!(g.object_view(own).power, 4);
            } else {
                assert_eq!(g.state.objects[own as usize].growth, Stats::default());
            }
            if missing & 2 == 0 {
                let v = g.object_view(foe);
                assert_eq!(v.power, 6);
                assert_eq!(v.damage, if missing & 1 == 0 { 4 } else { 0 });
            }
            assert_eq!(g.observe(p).last_evidence.unwrap().threshold, 6);
        }
    }
}
#[test]
fn partial_exile_and_late_observer_allocation_failures_restore_mana_resources_and_tap() {
    for p in [Player::P0, Player::P1] {
        for observer_failure in [false, true] {
            let mut g = setup(p);
            let a = named(&mut g, p, "Hamlet Glutton", Zone::Graveyard);
            let b = named(&mut g, p, "Hamlet Glutton", Zone::Graveyard);
            if !observer_failure {
                g.state.objects[b as usize].generation = u64::MAX;
            }
            let source = if observer_failure {
                named(&mut g, p, "Evidence Examiner", Zone::Battlefield);
                let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                let source = named(&mut g, p, "Forensic Researcher", Zone::Battlefield);
                replay(
                    &mut g,
                    Action::AnnounceActivate {
                        card: source,
                        ability: 1,
                        counter_payment: Counters::default(),
                    },
                );
                while !matches!(g.state.pending, Pending::Evidence { .. }) {
                    pick(&mut g, true, &[foe]);
                }
                source
            } else {
                let source = named(&mut g, p, "Analyze the Pollen", Zone::Hand);
                replay(&mut g, Action::AnnounceCast { card: source });
                while !matches!(g.state.pending, Pending::Evidence { .. }) {
                    pick(&mut g, true, &[]);
                }
                source
            };
            let ar = g.object_ref(a);
            let br = g.object_ref(b);
            replay(&mut g, Action::ChooseEvidence { object: ar });
            replay(&mut g, Action::ChooseEvidence { object: br });
            replay(&mut g, Action::FinishEvidence);
            if observer_failure {
                g.state.next_stack_id = u64::MAX;
            }
            let old = g.snapshot();
            let revision = g.revision();
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            assert!(matches!(
                g.apply(p, action),
                Err(Error::CapacityExceeded(_))
            ));
            assert_eq!(g.snapshot(), old);
            assert_eq!(g.revision(), revision);
            assert_eq!(g.object_view(a).zone, Zone::Graveyard);
            assert_eq!(g.object_view(b).zone, Zone::Graveyard);
            assert!(g.observe(p).last_evidence.is_none());
            assert!(!g.object_view(source).tapped);
        }
    }
}
#[test]
fn actual_authorized_exile_casts_share_optional_evidence_and_no_shortcut_bypasses() {
    for p in [Player::P0, Player::P1] {
        for collect in [false, true] {
            for n in [
                "Analyze the Pollen",
                "Bite Down on Crime",
                "Crimestopper Sprite",
                "Extract a Confession",
                "Vitu-Ghazi Inspector",
            ] {
                let mut g = setup(p);
                grave(&mut g, p);
                let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                let foe = named(&mut g, p.other(), "Hamlet Glutton", Zone::Battlefield);
                let wanted = named(&mut g, p, n, Zone::Library);
                let alania = named(&mut g, p, "Alania's Pathmaker", Zone::Hand);
                cast(&mut g, alania, false, &[]);
                settle(&mut g, false, &[]);
                assert_eq!(g.object_view(wanted).zone, Zone::Exile);
                assert!(
                    g.observe(p)
                        .exile_permissions
                        .iter()
                        .any(|r| r.object == g.object_ref(wanted))
                );
                reject(
                    &mut g,
                    Action::Cast {
                        card: wanted,
                        target: None,
                        payment: [0; 6],
                    },
                );
                reject(
                    &mut g,
                    Action::BeginCast {
                        card: wanted,
                        payment: [0; 6],
                    },
                );
                reject(&mut g, Action::AnnounceCast { card: wanted });
                replay(&mut g, Action::AnnounceExile { card: wanted });
                assert_eq!(g.observe(p).decision, "evidence_intent");
                while g.state.announcing.is_some() {
                    pick(&mut g, collect, &[own, foe]);
                }
                let item = g.state.stack.iter().find(|s| s.card == wanted).unwrap();
                let info = item.cast.unwrap();
                assert_eq!(info.origin, Zone::Exile);
                assert_eq!(info.method, CastMethod::Normal);
                assert_eq!(info.evidence, Some(collect));
                assert!(
                    info.kicker.is_none() && info.offspring.is_none() && info.bargain.is_none()
                );
                assert!(
                    !g.observe(p)
                        .exile_permissions
                        .iter()
                        .any(|r| r.object.card == wanted)
                );
                settle(&mut g, collect, &[own, foe]);
            }
        }
    }
}

#[test]
fn mana_sacrificed_real_cards_join_evidence_before_and_after_finish() {
    for p in [Player::P0, Player::P1] {
        for initially_sufficient in [false, true] {
            let mut g = setup(p);
            let ham = named(&mut g, p, "Hamlet Glutton", Zone::Graveyard);
            let pizza = named(&mut g, p, "Omni-Cheese Pizza", Zone::Battlefield);
            let spell = named(
                &mut g,
                p,
                if initially_sufficient {
                    "Crimestopper Sprite"
                } else {
                    "Analyze the Pollen"
                },
                Zone::Hand,
            );
            replay(&mut g, Action::AnnounceCast { card: spell });
            replay(&mut g, Action::ChooseCollectEvidence { collect: true });
            while !matches!(g.state.pending, Pending::Evidence { .. }) {
                pick(&mut g, true, &[]);
            }
            let object = g.object_ref(ham);
            replay(&mut g, Action::ChooseEvidence { object });
            if initially_sufficient {
                replay(&mut g, Action::FinishEvidence);
            }
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::ActivateMana { card, .. } if *card == pizza))
                .unwrap();
            replay(&mut g, action);
            assert_eq!(g.object_view(pizza).zone, Zone::Graveyard);
            assert!(matches!(g.state.pending, Pending::Evidence { .. }));
            assert!(g.observe(p.other()).evidence.is_none());
            assert_eq!(g.observe(p).evidence.unwrap().selected, vec![object]);
            let object = g.object_ref(pizza);
            replay(&mut g, Action::ChooseEvidence { object });
            replay(&mut g, Action::FinishEvidence);
            assert_eq!(g.object_view(pizza).zone, Zone::Graveyard);
            let payment = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            replay(&mut g, payment);
            assert_eq!(g.object_view(pizza).zone, Zone::Exile);
            let event = g.observe(p).last_evidence.unwrap();
            assert_eq!(event.total_mana, 9);
            assert_eq!(event.exiled.len(), 2);
            assert_eq!(event.sequence, 1);
        }
    }
}

#[test]
fn evidence_mana_reachability_rejects_unfunded_and_wrong_color_routes_and_no_window_activations() {
    for p in [Player::P0, Player::P1] {
        // Pizza cannot pay its own {1}; an unfunded future grave card is not evidence.
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0; 6];
        named(&mut g, p, "Hamlet Glutton", Zone::Graveyard);
        named(&mut g, p, "Omni-Cheese Pizza", Zone::Battlefield);
        let spell = named(&mut g, p, "Analyze the Pollen", Zone::Hand);
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: spell })
        );

        // A pure-tap activation has no CR601.2g mana window, even if Pizza
        // would supply enough evidence and the player already has spare mana.
        let mut g = setup(p);
        named(&mut g, p, "Bear Cub", Zone::Graveyard);
        named(&mut g, p, "Omni-Cheese Pizza", Zone::Battlefield);
        let source = named(&mut g, p, "Forensic Researcher", Zone::Battlefield);
        let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        assert!(!g.legal_actions().iter().any(
            |a| matches!(a, Action::AnnounceActivate { card, ability:1, .. } if *card==source)
        ));
        named(&mut g, p, "Hamlet Glutton", Zone::Graveyard);
        replay(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        while !matches!(g.state.pending, Pending::Evidence { .. }) {
            pick(&mut g, true, &[foe]);
        }
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::TapMana { .. } | Action::ActivateMana { .. }))
        );

        // After the only Forest has been tapped, spending that sole green on
        // Pizza to produce blue would strand Analyze's green cost: reject it.
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0; 6];
        named(&mut g, p, "Hamlet Glutton", Zone::Graveyard);
        let pizza = named(&mut g, p, "Omni-Cheese Pizza", Zone::Battlefield);
        let forest = named(&mut g, p, "Forest", Zone::Battlefield);
        let spell = named(&mut g, p, "Analyze the Pollen", Zone::Hand);
        let saved = g.snapshot();
        for _ in 0..20 {
            assert!(
                g.legal_actions()
                    .contains(&Action::AnnounceCast { card: spell })
            );
        }
        assert_eq!(g.snapshot(), saved);
        replay(&mut g, Action::AnnounceCast { card: spell });
        replay(&mut g, Action::ChooseCollectEvidence { collect: true });
        while !matches!(g.state.pending, Pending::Evidence { .. }) {
            pick(&mut g, true, &[]);
        }
        replay(&mut g, Action::TapMana { card: forest });
        let wrong = Action::ActivateMana {
            card: pizza,
            ability: 0,
            payment: [0, 0, 0, 0, 1, 0],
            counter_payment: Counters::default(),
            production: [0, 0, 1, 0, 0, 0],
        };
        reject(&mut g, wrong);
        let right=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana{card,production,..}if *card==pizza && production[4]==1)).unwrap();
        let before = g.snapshot();
        assert_eq!(g.apply(p.other(), right.clone()), Err(Error::WrongActor));
        assert_eq!(g.snapshot(), before);
        replay(&mut g, right);
    }
}

#[test]
fn tapped_mana_sacrifice_cannot_supply_future_evidence_on_many_source_boards() {
    for p in [Player::P0, Player::P1] {
        for tapped in [true, false] {
            let mut g = setup(p);
            g.state.players[p.index()].mana = [0; 6];
            named(&mut g, p, "Hamlet Glutton", Zone::Graveyard);
            let pizza = named(&mut g, p, "Omni-Cheese Pizza", Zone::Battlefield);
            g.state.objects[pizza as usize].tapped = tapped;
            for _ in 0..8 {
                for land in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
                    named(&mut g, p, land, Zone::Battlefield);
                }
            }
            let spell = named(&mut g, p, "Analyze the Pollen", Zone::Hand);
            let (colored, generic) = g.state.objects[spell as usize].card.cost();
            let before = g.snapshot();
            for _ in 0..10 {
                assert_eq!(
                    g.evidence_cast_fundable(p, spell, ManaCost { colored, generic }, true),
                    !tapped
                );
            }
            assert_eq!(g.snapshot(), before);
            replay(&mut g, Action::AnnounceCast { card: spell });
            assert_eq!(
                g.legal_actions()
                    .contains(&Action::ChooseCollectEvidence { collect: true }),
                !tapped
            );
        }
    }
}
