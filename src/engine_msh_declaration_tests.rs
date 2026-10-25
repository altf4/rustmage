//! Whole declaration consumers: exact event recipients and live combat targeting.
use super::ability_world::CharacteristicWorld;
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;
const KIT: &str = "S.H.I.E.L.D. Spy Kit";
const CROWD: &str = "Crowd of True Believers";
const ATTUMA: &str = "Attuma, Atlantean Warlord";
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, &[]);
    settle(g);
    id
}
fn finish(g: &mut Game) {
    for _ in 0..15 {
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
                        | Action::PayCosts { .. }
                        | Action::ChooseActivationCost { .. }
                )
            })
            .unwrap();
        checked(g, a);
    }
    panic!("unfinished payment");
}
fn announce(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn activate(g: &mut Game, id: CardId, target: CardId) {
    checked(g, announce(id));
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    finish(g);
}
fn equip(g: &mut Game, kit: CardId, target: CardId) {
    activate(g, kit, target);
    settle(g);
}
fn declare(g: &mut Game, p: Player, ids: &[CardId]) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    for &card in ids {
        checked(g, Action::Attack { card });
    }
    checked(g, Action::FinishAttackers);
}
fn ready(g: &mut Game, ids: &[CardId]) {
    for &id in ids {
        g.state.objects[id as usize].summoning_sick = false;
    }
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..120 {
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
            .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn orphan(g: Game, a: Action) -> Game {
    let before = g.snapshot();
    let actor = g.actor().unwrap();
    let revision = g.revision();
    let index = g.legal_actions().iter().position(|x| *x == a).unwrap();
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, a.clone()).unwrap();
    indexed.apply_index(actor, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    let after = raw.snapshot();
    raw.restore(&before).unwrap();
    raw.apply(actor, a).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}
fn kit_pair(g: &mut Game, p: Player) -> (CardId, CardId) {
    let kit = paid(g, p, KIT);
    let bear = paid(g, p, "Bear Cub");
    equip(g, kit, bear);
    ready(g, &[bear]);
    (kit, bear)
}
fn queued_scry(g: &mut Game) {
    until(g, |g| {
        matches!(g.state.pending, Pending::EffectLibrary { .. })
    });
}
fn visible(g: &Game, p: Player, id: CardId) -> bool {
    g.observe(p).objects.iter().any(|o| o.id == id)
}
// Preparation may already move a new packet into ordering, announcement or stack.
fn packets(g: &Game, card: CardId) -> Vec<&SpellView> {
    g.state
        .pending_triggers
        .iter()
        .chain(&g.state.ordering_triggers)
        .chain(&g.state.stack)
        .chain(g.state.announcing.iter().map(|a| &a.item))
        .filter(|item| item.card == card)
        .collect()
}
#[test]
fn msh_declaration_kit_exact_paid_cost_equip_timing_and_stats() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, KIT, Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        reject(&mut g, Action::AnnounceCast { card: id });
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 0];
        cast(&mut g, id, None, &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, announce(id));
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        checked(&mut g, announce(id));
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(
            g.stats(own),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        g.state.players[p.index()].mana = [30; 6];
        g.state.step = Step::BeginCombat;
        reject(&mut g, announce(id));
        g.state.step = Step::Main1;
        let stale = g.object_ref(own);
        checked(&mut g, announce(id));
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        finish(&mut g);
        g.move_object(own, Zone::Exile);
        g.move_object(own, Zone::Battlefield);
        settle(&mut g);
        assert_ne!(g.object_ref(own), stale);
        assert_eq!(g.object_view(id).attached_to, None);
        assert_eq!(g.stats(own).power, 2);
    }
}
#[test]
fn msh_declaration_kit_paid_attack_private_scry_orphan_replay() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let (kit, bear) = kit_pair(&mut g, p);
        next_turn(&mut g);
        next_turn(&mut g);
        declare(&mut g, p, &[bear]);
        assert!(g.object_view(bear).tapped);
        let packet = packets(&g, kit);
        let event = packet[0].attachment_trigger.as_ref().unwrap();
        assert_eq!(event.recipient, g.object_ref(bear));
        queued_scry(&mut g);
        assert!(!g.object_view(bear).tapped);
        let top = *g.state.players[p.index()].library.last().unwrap();
        assert_eq!(
            g.observe(p).library_choice.as_ref().unwrap().remaining[0]
                .reference
                .card,
            top
        );
        assert!(!visible(&g, p.other(), top));
        assert!(g.observe(p).library_choice.is_some());
        assert!(g.observe(p.other()).library_choice.is_none());
        g = orphan(
            g,
            Action::ArrangeLibrary {
                card: top,
                destination: LibraryDestination::Bottom,
            },
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.first(), Some(&top));
    }
}
#[test]
fn msh_declaration_kit_original_recipient_and_captured_controller() {
    for p in [Player::P0, Player::P1] {
        for change in 0..5 {
            let mut g = setup(p);
            let (kit, bear) = kit_pair(&mut g, p);
            let other = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            g.state.objects[other as usize].tapped = true;
            // Equipment can be controlled by someone other than its equipped attacker.
            g.record_control(g.object_ref(kit), p.other(), None);
            declare(&mut g, p, &[bear]);
            let recipient = g.object_ref(bear);
            match change {
                0 => {
                    g.move_object(kit, Zone::Graveyard);
                }
                1 => {
                    assert!(g.attach_equipment(g.object_ref(kit), g.object_ref(other)));
                }
                2 => {
                    g.move_object(bear, Zone::Exile);
                    g.move_object(bear, Zone::Battlefield);
                    g.state.objects[bear as usize].tapped = true;
                }
                3 => {
                    let aura = named(&mut g, p, "Imprisoned in the Moon", Zone::Battlefield);
                    g.state.objects[aura as usize].attached_to = Some(recipient);
                    assert!(!g.is_creature(bear));
                }
                _ => g.record_control(g.object_ref(kit), p, None),
            }
            queued_scry(&mut g);
            assert_eq!(g.actor(), Some(p.other()));
            assert_eq!(g.object_view(bear).tapped, change == 2);
            assert!(g.object_view(other).tapped);
            settle(&mut g);
        }
    }
}
#[test]
fn msh_declaration_kit_event_alone_is_not_later_live_count() {
    for p in [Player::P0, Player::P1] {
        for two in [false, true] {
            let mut g = setup(p);
            let (kit, bear) = kit_pair(&mut g, p);
            let other = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            let ids = if two { vec![bear, other] } else { vec![bear] };
            declare(&mut g, p, &ids);
            if two {
                g.move_object(other, Zone::Exile);
            } else {
                g.create_token_batch_capture(
                    p,
                    Card::catalogue().nth(5388).unwrap(),
                    1,
                    true,
                    Some(&[CombatDefender::Player(p.other())]),
                    true,
                );
            }
            assert_eq!(packets(&g, kit).len(), usize::from(!two));
            settle(&mut g);
            assert_eq!(g.object_view(bear).tapped, two);
        }
    }
}
#[test]
fn msh_declaration_kit_stun_prohibition_and_suppression() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..4 {
            let mut g = setup(p);
            let (kit, bear) = kit_pair(&mut g, p);
            if mode == 2 {
                let aura = named(&mut g, p, "Stop Cold", Zone::Hand);
                cast(&mut g, aura, None, &[Target::Card(kit)]);
                settle(&mut g);
                assert_eq!(g.stats(bear).power, 2);
            }
            declare(&mut g, p, &[bear]);
            g.state.objects[bear as usize].counters.stun = 1;
            if mode == 1 {
                let ice = named(&mut g, p, "Frozen in Ice", Zone::Battlefield);
                g.state.objects[ice as usize].attached_to = Some(g.object_ref(bear));
            }
            if mode == 3 {
                until(&mut g, |g| g.state.pending == Pending::Priority);
                let aura = named(&mut g, p, "Stop Cold", Zone::Hand);
                cast(&mut g, aura, None, &[Target::Card(kit)]);
            }
            settle(&mut g);
            assert!(g.object_view(bear).tapped);
            assert_eq!(
                g.counter_count(bear, CounterKind::Stun),
                u64::from(mode == 1 || mode == 2)
            );
        }
    }
}
#[test]
fn msh_declaration_crowd_paid_tap_sickness_live_target_and_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 0];
        let crowd = paid(&mut g, p, CROWD);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(
            g.stats(crowd),
            Stats {
                power: 1,
                toughness: 2
            }
        );
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        declare(&mut g, p, &[bear]);
        reject(&mut g, announce(crowd));
        ready(&mut g, &[crowd]);
        let life = g.life(p);
        activate(&mut g, crowd, bear);
        assert!(g.object_view(crowd).tapped);
        reject(&mut g, announce(crowd));
        settle(&mut g);
        assert_eq!(
            g.stats(bear),
            Stats {
                power: 3,
                toughness: 2
            }
        );
        assert_eq!(g.life(p), life + 1);
        next_turn(&mut g);
        assert_eq!(g.stats(bear).power, 2);
    }
}
#[test]
fn msh_declaration_crowd_global_live_alone_and_removed_defender() {
    for p in [Player::P0, Player::P1] {
        for removed in [false, true] {
            let mut g = setup(p);
            let crowd = paid(&mut g, p, CROWD);
            ready(&mut g, &[crowd]);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let second = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            declare(&mut g, p, &[bear, second]);
            reject(&mut g, announce(crowd));
            if removed {
                g.move_object(second, Zone::Exile);
                g.state.attackers[0].defender = None;
            } else {
                g.state.attackers[1].defender = None;
                reject(&mut g, announce(crowd));
                g.move_object(second, Zone::Graveyard);
            }
            activate(&mut g, crowd, bear);
            settle(&mut g);
            assert_eq!(g.stats(bear).power, 3);
        }
    }
}
#[test]
fn msh_declaration_crowd_all_invalid_stops_life_and_growth() {
    for p in [Player::P0, Player::P1] {
        for change in 0..5 {
            let mut g = setup(p);
            let crowd = paid(&mut g, p, CROWD);
            ready(&mut g, &[crowd]);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            declare(&mut g, p, &[bear]);
            let life = g.life(p);
            activate(&mut g, crowd, bear);
            match change {
                0 => {
                    g.move_object(bear, Zone::Exile);
                    g.move_object(bear, Zone::Battlefield);
                }
                1 => g.record_control(g.object_ref(bear), p.other(), None),
                2 => g.state.attackers.clear(),
                3 => {
                    let moon = named(&mut g, p, "Imprisoned in the Moon", Zone::Battlefield);
                    g.state.objects[moon as usize].attached_to = Some(g.object_ref(bear));
                }
                _ => {
                    g.create_token_batch_capture(
                        p,
                        Card::catalogue().nth(5388).unwrap(),
                        1,
                        true,
                        Some(&[CombatDefender::Player(p.other())]),
                        true,
                    );
                }
            }
            settle(&mut g);
            assert_eq!(g.life(p), life);
            if g.is_creature(bear) {
                assert_eq!(g.stats(bear).power, 2);
            }
        }
    }
}
#[test]
fn msh_declaration_crowd_source_departure_and_captured_life_orphan() {
    for p in [Player::P0, Player::P1] {
        for depart in [false, true] {
            let mut g = setup(p);
            let crowd = paid(&mut g, p, CROWD);
            ready(&mut g, &[crowd]);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            declare(&mut g, p, &[bear]);
            let life = g.life(p);
            let other = g.life(p.other());
            checked(&mut g, announce(crowd));
            g = orphan(
                g,
                Action::ChooseTarget {
                    target: Target::Card(bear),
                },
            );
            finish(&mut g);
            if depart {
                g.move_object(crowd, Zone::Graveyard);
            } else {
                g.record_control(g.object_ref(crowd), p.other(), None);
            }
            settle(&mut g);
            assert_eq!(g.life(p), life + 1);
            assert_eq!(g.life(p.other()), other);
        }
    }
}
#[test]
fn msh_declaration_crowd_actual_borrowed_source_payer_and_target() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p.other());
        let donor = paid(&mut g, p.other(), CROWD);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.step == Step::Main1 && g.state.pending == Pending::Priority
        });
        assert_eq!(g.object_view(donor).owner, p.other());
        // The real turn change emptied mana; fund the two actual printed spells.
        g.state.players[p.index()].mana = [30; 6];
        let moon = named(&mut g, p, "Imprisoned in the Moon", Zone::Hand);
        cast(&mut g, moon, None, &[Target::Card(donor)]);
        settle(&mut g);
        let forge = paid(&mut g, p, "Territory Forge");
        assert_eq!(g.object_view(donor).zone, Zone::Exile);
        let b = g
            .borrowed_ability_views()
            .into_iter()
            .find(|b| b.source.card == forge && b.exiled.card == donor)
            .unwrap();
        assert_eq!(b.definition.name(), CROWD);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        declare(&mut g, p, &[bear]);
        let life = g.life(p);
        let a = Action::AnnounceBorrowedAbility {
            card: forge,
            exiled: b.exiled,
            ability: 0,
            counter_payment: Counters::default(),
        };
        checked(&mut g, a);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        finish(&mut g);
        assert!(g.object_view(forge).tapped);
        assert_eq!(
            g.state.stack.last().unwrap().source_facts.card.name(),
            "Territory Forge"
        );
        g.move_object(forge, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.life(p), life + 1);
        assert_eq!(g.stats(bear).power, 3);
    }
}
#[test]
fn msh_declaration_attuma_paid_cost_live_lord_control_types_and_suppression() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, ATTUMA, Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 3];
        reject(&mut g, Action::AnnounceCast { card: id });
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 2];
        cast(&mut g, id, None, &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(
            g.stats(id),
            Stats {
                power: 3,
                toughness: 4
            }
        );
        let own = named(&mut g, p, "Atlantean Cavalry", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Atlantean Cavalry", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert_eq!(
            g.stats(own),
            Stats {
                power: 4,
                toughness: 3
            }
        );
        assert_eq!(g.stats(enemy).power, 3);
        assert_eq!(g.stats(bear).power, 2);
        g.record_control(g.object_ref(own), p.other(), None);
        assert_eq!(g.stats(own).power, 3);
        g.record_control(g.object_ref(enemy), p, None);
        assert_eq!(g.stats(enemy).power, 4);
        g.state.players[p.index()].mana = [30; 6];
        let witness = named(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, witness, None, &[Target::Card(enemy)]);
        settle(&mut g);
        assert_eq!(g.stats(enemy).power, 1);
        let moon = named(&mut g, p, "Imprisoned in the Moon", Zone::Hand);
        cast(&mut g, moon, None, &[Target::Card(id)]);
        settle(&mut g);
        assert_eq!(g.stats(own).power, 3);
    }
}
#[test]
fn msh_declaration_attuma_one_draw_per_player_merfolk_declaration() {
    for p in [Player::P0, Player::P1] {
        for count in 0..=2 {
            let mut g = setup(p);
            let attuma = paid(&mut g, p, ATTUMA);
            let a = named(&mut g, p, "Atlantean Cavalry", Zone::Battlefield);
            let b = named(&mut g, p, "Atlantean Cavalry", Zone::Battlefield);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let hand = g.hand_count(p);
            let drawn = *g.state.players[p.index()].library.last().unwrap();
            let ids = match count {
                0 => vec![bear],
                1 => vec![a],
                _ => vec![a, b],
            };
            declare(&mut g, p, &ids);
            assert_eq!(packets(&g, attuma).len(), usize::from(count > 0));
            settle(&mut g);
            assert_eq!(g.hand_count(p), hand + usize::from(count > 0));
            if count > 0 {
                assert!(visible(&g, p, drawn));
                assert!(!visible(&g, p.other(), drawn));
            }
        }
    }
}
#[test]
fn msh_declaration_attuma_planeswalker_and_player_defenders_stay_distinct() {
    for p in [Player::P0, Player::P1] {
        for merfolk_player in [false, true] {
            let mut g = setup(p);
            let attuma = paid(&mut g, p, ATTUMA);
            let merfolk = named(&mut g, p, "Atlantean Cavalry", Zone::Battlefield);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let walker = named(
                &mut g,
                p.other(),
                "Liliana, Dreadhorde General",
                Zone::Battlefield,
            );
            g.state.active = p;
            g.state.priority = p;
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            // A directly staged planeswalker needs its normal loyalty to survive SBA.
            g.state.objects[walker as usize].counters.loyalty = 6;
            let walker_ref = g.object_ref(walker);
            for (card, to_player) in [(merfolk, merfolk_player), (bear, !merfolk_player)] {
                checked(
                    &mut g,
                    if to_player {
                        Action::Attack { card }
                    } else {
                        Action::AttackPlaneswalker {
                            card,
                            defender: walker_ref,
                        }
                    },
                );
            }
            checked(&mut g, Action::FinishAttackers);
            assert_eq!(packets(&g, attuma).len(), usize::from(merfolk_player));
            settle(&mut g);
        }
    }
}
#[test]
fn msh_declaration_attuma_entered_attacking_is_not_declared() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let attuma = paid(&mut g, p, ATTUMA);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let merfolk = named(&mut g, p, "Atlantean Cavalry", Zone::Battlefield);
        // Isolate the declaration observer boundary: an already-attacking creature was not declared.
        g.state.attackers.push(AttackDeclaration {
            card: merfolk,
            defender: Some(CombatDefender::Player(p.other())),
            blocked: false,
        });
        let hand = g.hand_count(p);
        declare(&mut g, p, &[bear]);
        assert!(packets(&g, attuma).is_empty());
        settle(&mut g);
        assert_eq!(g.hand_count(p), hand);
    }
}
#[test]
fn msh_declaration_attuma_frozen_event_controller_and_departure_orphan_draw() {
    for p in [Player::P0, Player::P1] {
        for change in 0..3 {
            let mut g = setup(p);
            let attuma = paid(&mut g, p, ATTUMA);
            ready(&mut g, &[attuma]);
            let hand = g.hand_count(p);
            let other = g.hand_count(p.other());
            declare(&mut g, p, &[attuma]);
            match change {
                0 => {
                    g.move_object(attuma, Zone::Graveyard);
                }
                1 => g.record_control(g.object_ref(attuma), p.other(), None),
                _ => {
                    let moon = named(&mut g, p, "Imprisoned in the Moon", Zone::Battlefield);
                    g.state.objects[moon as usize].attached_to = Some(g.object_ref(attuma));
                }
            }
            until(&mut g, |g| {
                g.state.pending == Pending::Priority && !g.state.stack.is_empty()
            });
            g = orphan(g, Action::Pass);
            settle(&mut g);
            assert_eq!(g.hand_count(p), hand + 1);
            assert_eq!(g.hand_count(p.other()), other);
        }
    }
}
#[test]
fn msh_declaration_paid_mobilize_keeps_kit_event_but_invalidates_crowd_target() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let kit = paid(&mut g, p, KIT);
        let voice = paid(&mut g, p, "Voice of Victory");
        let crowd = paid(&mut g, p, CROWD);
        equip(&mut g, kit, voice);
        ready(&mut g, &[voice, crowd]);
        declare(&mut g, p, &[voice]);
        settle(&mut g);
        assert_eq!(g.state.attackers.len(), 3);
        assert!(!g.object_view(voice).tapped);
        reject(&mut g, announce(crowd));
        let tokens: Vec<_> = g
            .state
            .attackers
            .iter()
            .filter_map(|a| (a.card != voice).then_some(a.card))
            .collect();
        for token in tokens {
            g.move_object(token, Zone::Exile);
        }
        activate(&mut g, crowd, voice);
        settle(&mut g);
        assert_eq!(g.stats(voice).power, 3);
    }
}
#[test]
fn msh_declaration_attuma_existing_copy_recipe_and_suppression_before_event() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = paid(&mut g, p, ATTUMA);
        let spell = named(&mut g, p, "Applied Geometry", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Card(source)]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Legend { .. })
        });
        let copy = (0..g.state.objects.len())
            .map(|i| i as CardId)
            .find(|&i| {
                g.on_battlefield(i)
                    && g.object_view(i).is_token()
                    && g.object_view(i).card.name() == ATTUMA
            })
            .unwrap();
        let copy_ref = g.object_ref(copy);
        checked(&mut g, Action::KeepLegend { object: copy_ref });
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(g.stats(copy).power, 6);
        assert!(g.object_view(copy).has_subtype("Merfolk"));
        let merfolk = named(&mut g, p, "Atlantean Cavalry", Zone::Battlefield);
        assert_eq!(g.stats(merfolk).power, 4);
        let moon = named(&mut g, p, "Imprisoned in the Moon", Zone::Hand);
        cast(&mut g, moon, None, &[Target::Card(copy)]);
        settle(&mut g);
        let hand = g.hand_count(p);
        declare(&mut g, p, &[merfolk]);
        settle(&mut g);
        assert_eq!(g.hand_count(p), hand);
        assert_eq!(g.stats(merfolk).power, 3);
    }
}
