//! Paid casts, exact copy identities, retarget choices and owned continuation proofs.
use super::ability_timeline_tests::{checked, setup};
use super::tests::spawn;
use super::*;

fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}
fn announce(g: &mut Game, card: CardId, targets: &[Target]) {
    checked(g, Action::AnnounceCast { card });
    for _ in 0..20 {
        if g.state.announcing.is_none() {
            return;
        }
        let actions = g.legal_actions();
        let selected = g.state.announcing.as_ref().unwrap().item.targets.len();
        let action = targets
            .get(selected)
            .and_then(|target| {
                actions
                    .iter()
                    .find(|a| **a == Action::ChooseTarget { target: *target })
            })
            .or_else(|| {
                actions.iter().find(|a| {
                    matches!(
                        a,
                        Action::ChooseSpellCost { .. }
                            | Action::PayCosts { .. }
                            | Action::FinishTargets
                    )
                })
            })
            .unwrap_or_else(|| panic!("cast choices {actions:?}"))
            .clone();
        checked(g, action);
    }
    panic!("unpaid cast");
}
fn progress(g: &mut Game, stop_retarget: bool) {
    for _ in 0..150 {
        if stop_retarget && matches!(g.state.pending, Pending::Retarget { .. }) {
            return;
        }
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        let actions = g.legal_actions();
        let action = actions
            .iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::KeepCopyTarget
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                )
            })
            .unwrap_or_else(|| panic!("settle {:?} {actions:?}", g.state.pending))
            .clone();
        checked(g, action);
    }
    panic!("unsettled");
}
fn teach(g: &mut Game, p: Player) {
    let id = named(g, p, "Teach by Example", Zone::Hand);
    announce(g, id, &[]);
    progress(g, false);
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    assert!(g.apply(g.actor().unwrap(), action).is_err());
    assert_eq!(g.snapshot(), before);
}
#[test]
fn teach_copies_next_qualifying_cast_once_and_retargets_without_casting_again() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        teach(&mut g, p);
        let body = named(&mut g, p, "Bear Cub", Zone::Hand);
        announce(&mut g, body, &[]);
        progress(&mut g, false);
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        announce(&mut g, shock, &[Target::Player(p.other())]);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 3);
        progress(&mut g, true);
        assert!(matches!(g.state.pending, Pending::Retarget { .. }));
        reject(
            &mut g,
            Action::ChangeCopyTarget {
                target: Target::Spell(shock),
            },
        );
        checked(
            &mut g,
            Action::ChangeCopyTarget {
                target: Target::Player(p),
            },
        );
        assert_eq!(g.state.stack.iter().filter(|s| s.is_copy).count(), 1);
        let copy = g.state.stack.iter().find(|s| s.is_copy).unwrap().clone();
        assert_ne!(copy.card, shock);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 3);
        assert!(g.legal_actions().iter().any(|a| matches!(a, Action::Pass)));
        progress(&mut g, false);
        assert_eq!(g.state.players[p.index()].life, 18);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(g.state.objects[copy.card as usize].zone, Zone::Outside);
        assert_eq!(g.state.objects[shock as usize].zone, Zone::Graveyard);
        let second = named(&mut g, p, "Shock", Zone::Hand);
        announce(&mut g, second, &[Target::Player(p.other())]);
        progress(&mut g, false);
        assert_eq!(g.state.players[p.other().index()].life, 16);
        assert!(g.state.stack_copy_data.is_none());
    }
}
#[test]
fn storm_copies_fixed_prior_count_and_survives_source_and_original_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let first = named(&mut g, p, "Shock", Zone::Hand);
        announce(&mut g, first, &[Target::Player(p.other())]);
        progress(&mut g, false);
        let storm = named(&mut g, p, "Thousand-Year Storm", Zone::Battlefield);
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        announce(&mut g, shock, &[Target::Player(p.other())]);
        assert_eq!(
            g.state
                .stack
                .iter()
                .filter(|s| s.copy_event.is_some())
                .count(),
            1
        );
        // Countering the original does not invalidate the trigger's frozen spell.
        let id = g
            .state
            .stack
            .iter()
            .find(|s| s.card == shock && s.kind == StackKind::Spell)
            .unwrap()
            .id;
        assert!(g.counter_stack_item(id, false));
        g.move_object(storm, Zone::Graveyard);
        progress(&mut g, true);
        checked(&mut g, Action::KeepCopyTarget);
        assert_eq!(g.state.stack.iter().filter(|s| s.is_copy).count(), 1);
        progress(&mut g, false);
        assert_eq!(g.state.players[p.other().index()].life, 16);
        assert_eq!(g.state.players[p.index()].instant_sorcery_cast_this_turn, 2);
    }
}
#[test]
fn copied_spell_has_its_own_counter_target_and_fizzles_independently() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        teach(&mut g, p);
        let a = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p, "Shock", Zone::Hand);
        announce(&mut g, spell, &[Target::Card(a)]);
        progress(&mut g, true);
        checked(
            &mut g,
            Action::ChangeCopyTarget {
                target: Target::Card(b),
            },
        );
        let copy = g.state.stack.iter().find(|s| s.is_copy).unwrap().clone();
        let counter = named(&mut g, p, "Negate", Zone::Hand);
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: counter })
        );
        announce(&mut g, counter, &[Target::Stack(copy.id)]);
        progress(&mut g, false);
        assert_eq!(g.state.objects[b as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.objects[a as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[copy.card as usize].zone, Zone::Outside);
    }
}
#[test]
fn bolt_bend_discounts_and_changes_an_ability_target_with_original_controller_legality() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Serra Angel", Zone::Battlefield);
        let archer = named(&mut g, p.other(), "Fanatical Firebrand", Zone::Battlefield);
        g.state.priority = p.other();
        let action = g.legal_actions().into_iter().find(|a| matches!(a, Action::Activate { card, target: Some(Target::Player(t)), .. } if *card == archer && *t == p)).unwrap();
        checked(&mut g, action);
        let ability = g.state.stack.last().unwrap().id;
        g.state.priority = p;
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        let bend = named(&mut g, p, "Bolt Bend", Zone::Hand);
        announce(&mut g, bend, &[Target::Stack(ability)]);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        progress(&mut g, true);
        assert!(!g.legal_actions().contains(&Action::KeepCopyTarget));
        reject(
            &mut g,
            Action::ChangeCopyTarget {
                target: Target::Player(p),
            },
        );
        checked(
            &mut g,
            Action::ChangeCopyTarget {
                target: Target::Player(p.other()),
            },
        );
        progress(&mut g, false);
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(g.state.players[p.other().index()].life, 19);
    }
}
