//! Paid first-tap rules, complete history and exact non-targeted recipients.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;

fn pay(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, a);
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, targets);
    settle(g);
    id
}
fn captain(g: &mut Game, p: Player) -> CardId {
    g.state.players[p.index()].mana = [1, 1, 0, 0, 0, 1];
    let id = paid(g, p, "Captain America, Living Legend", &[]);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    g.state.players[p.index()].mana = [30; 6];
    id
}
fn elf(g: &mut Game, p: Player) -> CardId {
    let id = named(g, p, "Llanowar Elves", Zone::Battlefield);
    g.state.objects[id as usize].summoning_sick = false;
    id
}
fn first_stack(g: &mut Game, source: CardId) {
    g.prepare_triggers();
    for _ in 0..50 {
        if g.state.pending == Pending::Priority && g.state.stack.iter().any(|s| s.card == source) {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PutTrigger { .. } | Action::FinishTargets))
            .unwrap();
        checked(g, a);
    }
    panic!("missing first-tap packet");
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
#[test]
fn msh_first_tap_paid_body_vigilance_no_attacking_tap() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let c = captain(&mut g, p);
        assert_eq!((g.stats(c).power, g.stats(c).toughness), (3, 4));
        assert!(g.keywords(c).contains(Keywords::VIGILANCE));
        assert!(
            g.has_subtype(c, "Hero") && g.has_subtype(c, "Soldier") && g.has_subtype(c, "Human")
        );
        g.state.objects[c as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: c });
        checked(&mut g, Action::FinishAttackers);
        assert!(!g.object_view(c).tapped);
        assert!(g.state.tap_history.is_none());
    }
}
#[test]
fn msh_first_tap_actual_mana_untaps_first_but_not_second() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        captain(&mut g, p);
        let a = elf(&mut g, p);
        checked(&mut g, Action::TapMana { card: a });
        settle(&mut g);
        assert!(!g.object_view(a).tapped);
        assert!(g.first_tap(g.object_ref(a)));
        checked(&mut g, Action::TapMana { card: a });
        settle(&mut g);
        assert!(g.object_view(a).tapped);
        assert!(!g.first_tap(g.object_ref(a)));
        reject(&mut g, Action::TapMana { card: a });
    }
}
#[test]
fn msh_first_tap_history_before_observer_includes_single_and_group_taps() {
    for p in [Player::P0, Player::P1] {
        for group in [false, true] {
            let mut g = setup(p);
            let a = elf(&mut g, p);
            let b = elf(&mut g, p);
            if group {
                g.tap_objects(&[a, b]);
            } else {
                checked(&mut g, Action::TapMana { card: a });
            }
            let old = g.object_ref(a);
            assert!(g.first_tap(old));
            captain(&mut g, p);
            g.untap_objects(&[a], untap_rules::UntapCause::Effect);
            checked(&mut g, Action::TapMana { card: a });
            settle(&mut g);
            assert!(g.object_view(a).tapped);
            assert!(!g.first_tap(old));
        }
    }
}
#[test]
fn msh_first_tap_prior_foreign_control_and_opposing_turn_count() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = elf(&mut g, p.other());
        g.tap_objects(&[a]);
        g.untap_objects(&[a], untap_rules::UntapCause::Effect);
        g.record_control(g.object_ref(a), p, None);
        g.state.objects[a as usize].summoning_sick = false;
        captain(&mut g, p);
        checked(&mut g, Action::TapMana { card: a });
        settle(&mut g);
        assert!(g.object_view(a).tapped);
        let mut h = setup(p);
        let c = captain(&mut h, p);
        let other = named(
            &mut h,
            p.other(),
            "Captain America, Living Legend",
            Zone::Battlefield,
        );
        let a = elf(&mut h, p);
        h.state.active = p.other();
        h.tap_objects(&[a]);
        assert!(
            !h.state
                .pending_triggers
                .iter()
                .any(|s| s.card == c || s.card == other)
        );
        h.state.active = p;
        h.untap_objects(&[a], untap_rules::UntapCause::Effect);
        h.tap_objects(&[a]);
        settle(&mut h);
        assert!(h.object_view(a).tapped);
    }
}
#[test]
fn msh_first_tap_land_history_survives_real_animation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = named(&mut g, p, "Great Hall of the Biblioplex", Zone::Battlefield);
        checked(
            &mut g,
            Action::ActivateMana {
                card: land,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 0, 1],
            },
        );
        captain(&mut g, p);
        g.untap_objects(&[land], untap_rules::UntapCause::Effect);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: land,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert!(g.is_creature(land));
        g.state.objects[land as usize].summoning_sick = false;
        checked(
            &mut g,
            Action::ActivateMana {
                card: land,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 0, 1],
            },
        );
        settle(&mut g);
        assert!(g.object_view(land).tapped);
    }
}
#[test]
fn msh_first_tap_paid_two_target_effect_queues_each_and_stun_replaces_untap() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        captain(&mut g, p);
        let a = elf(&mut g, p);
        let b = elf(&mut g, p);
        paid(&mut g, p, "Out Cold", &[Target::Card(a), Target::Card(b)]);
        for id in [a, b] {
            assert!(g.object_view(id).tapped);
            assert_eq!(g.counter_count(id, CounterKind::Stun), 0);
            assert!(g.first_tap(g.object_ref(id)));
        }
        g.tap_objects(&[a, a, b]);
        assert!(g.first_tap(g.object_ref(a)));
        assert!(g.state.pending_triggers.is_empty());
    }
}
#[test]
fn msh_first_tap_crew_group_interacts_with_single_rewrite_event() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        captain(&mut g, p);
        let plan = paid(&mut g, p, "Rewrite History", &[]);
        let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let v = named(&mut g, p, "Detention Chariot", Zone::Battlefield);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: v,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        for id in [a, b] {
            let object = g.object_ref(id);
            checked(&mut g, Action::ChooseCrewer { object });
        }
        checked(&mut g, Action::FinishCrew);
        settle(&mut g);
        assert!(!g.object_view(a).tapped && !g.object_view(b).tapped);
        assert!(g.is_creature(v));
        assert_eq!(g.counter_count(plan, CounterKind::Plan), 1);
    }
}
#[test]
fn msh_first_tap_convoke_reserves_without_history_and_commits_each_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        captain(&mut g, p);
        let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let victim = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let spell = named(&mut g, p, "Vote Out", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 1];
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        checked(&mut g, Action::BeginConvoke);
        for id in [a, b] {
            let object = g.object_ref(id);
            checked(
                &mut g,
                Action::ChooseConvoker {
                    object,
                    contribution: ConvokeContribution::Generic,
                },
            );
            assert!(!g.first_tap(object));
            assert!(!g.object_view(id).tapped);
        }
        pay(&mut g);
        settle(&mut g);
        assert!(!g.object_view(a).tapped && !g.object_view(b).tapped);
        assert!(g.first_tap(g.object_ref(a)));
        assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_first_tap_intervening_condition_fails_after_untap_and_retap() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let c = captain(&mut g, p);
        let a = elf(&mut g, p);
        checked(&mut g, Action::TapMana { card: a });
        first_stack(&mut g, c);
        paid(&mut g, p, "Shore Up", &[Target::Card(a)]);
        // Shore Up resolved before the old Captain packet, so use a separate case
        // that commits the intervening second tap while the old packet is pending.
        assert!(!g.object_view(a).tapped);
        let b = elf(&mut g, p);
        checked(&mut g, Action::TapMana { card: b });
        first_stack(&mut g, c);
        g.untap_objects(&[b], untap_rules::UntapCause::Effect);
        g.tap_objects(&[b]);
        settle(&mut g);
        assert!(g.object_view(b).tapped);
        assert!(!g.first_tap(g.object_ref(b)));
    }
}
#[test]
fn msh_first_tap_source_departure_theft_and_later_suppression_preserve_packet() {
    for p in [Player::P0, Player::P1] {
        for change in 0..3 {
            let mut g = setup(p);
            let c = captain(&mut g, p);
            let a = elf(&mut g, p);
            checked(&mut g, Action::TapMana { card: a });
            first_stack(&mut g, c);
            if change == 0 {
                paid(&mut g, p, "Murder", &[Target::Card(c)]);
                assert_eq!(g.object_view(c).zone, Zone::Graveyard);
            } else if change == 1 {
                g.record_control(g.object_ref(c), p.other(), None);
                settle(&mut g);
            } else {
                paid(&mut g, p, "Eaten by Piranhas", &[Target::Card(c)]);
            }
            assert!(!g.object_view(a).tapped);
        }
    }
}
#[test]
fn msh_first_tap_removed_printed_ability_and_foreign_creatures_do_not_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let c = captain(&mut g, p);
        let a = elf(&mut g, p);
        let b = elf(&mut g, p.other());
        paid(&mut g, p, "Witness Protection", &[Target::Card(c)]);
        g.tap_objects(&[a, b]);
        settle(&mut g);
        assert!(g.object_view(a).tapped && g.object_view(b).tapped);
    }
}
#[test]
fn msh_first_tap_recipient_new_incarnation_does_not_inherit_old_packet_or_history() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let c = captain(&mut g, p);
        let a = elf(&mut g, p);
        let old = g.object_ref(a);
        checked(&mut g, Action::TapMana { card: a });
        first_stack(&mut g, c);
        g.move_object(a, Zone::Exile);
        let movement = g.entry_move(a, Zone::Battlefield, true);
        g.stage_entries(vec![movement], None);
        g.advance_entries();
        settle(&mut g);
        assert_ne!(old, g.object_ref(a));
        assert!(g.object_view(a).tapped);
        assert!(!g.first_tap(g.object_ref(a)));
        g.untap_objects(&[a], untap_rules::UntapCause::Effect);
        g.state.objects[a as usize].summoning_sick = false;
        checked(&mut g, Action::TapMana { card: a });
        settle(&mut g);
        assert!(!g.object_view(a).tapped);
        assert!(g.first_tap(g.object_ref(a)));
    }
}
#[test]
fn msh_first_tap_turn_reset_and_owned_cow_history_survive_parent_disposal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        captain(&mut g, p);
        let a = elf(&mut g, p);
        checked(&mut g, Action::TapMana { card: a });
        settle(&mut g);
        let old = g.object_ref(a);
        let mut branch = {
            let parent = g.clone();
            parent.clone()
        };
        assert!(
            branch
                .state
                .tap_history
                .as_ref()
                .unwrap()
                .ptr_eq(g.state.tap_history.as_ref().unwrap())
        );
        checked(&mut branch, Action::TapMana { card: a });
        settle(&mut branch);
        assert!(!branch.first_tap(old));
        assert!(g.first_tap(old));
        checked(&mut g, Action::TapMana { card: a });
        settle(&mut g);
        next_turn(&mut g);
        next_turn(&mut g);
        assert_eq!(g.state.active, p);
        assert!(g.state.tap_history.is_none());
        assert!(!g.object_view(a).tapped);
        checked(&mut g, Action::TapMana { card: a });
        settle(&mut g);
        assert!(!g.object_view(a).tapped);
    }
}
