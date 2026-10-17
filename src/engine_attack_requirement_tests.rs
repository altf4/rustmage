//! Mandatory-attack regression tests use the fully implemented Juggernaut.
use super::tests::{act, cast, scenario, spawn};
use super::*;
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for player in &mut g.state.players {
        player.mana = [20; 6];
    }
    g
}
fn required(g: &mut Game, p: Player) -> CardId {
    let card = Card::from_name("Juggernaut").unwrap();
    assert!(card.is_supported());
    spawn(g, p, card, Zone::Battlefield)
}
fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
    spawn(g, p, Card::from_name(n).unwrap(), z)
}
fn begin(g: &mut Game) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    g.state.selected_attackers.clear();
    g.state.attackers.clear();
}
fn finishing(g: &Game) -> bool {
    g.legal_actions().contains(&Action::FinishAttackers)
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
fn replay(g: &mut Game, action: Action) {
    let p = g.actor().unwrap();
    let revision = g.revision();
    let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
    let before = g.snapshot();
    assert_eq!(g.apply(p.other(), action.clone()), Err(Error::WrongActor));
    assert_eq!(g.snapshot(), before);
    let mut raw = {
        let parent = g.clone();
        parent.clone()
    };
    let mut indexed = {
        let parent = g.clone();
        parent.clone()
    };
    raw.apply(p, action.clone()).unwrap();
    indexed.apply_index(p, revision, index).unwrap();
    act(g, action.clone());
    assert_eq!(g.snapshot(), raw.snapshot());
    assert_eq!(g.snapshot(), indexed.snapshot());
    for viewer in [p, p.other()] {
        assert_eq!(
            serde_json::to_value(g.observe(viewer)).unwrap(),
            serde_json::to_value(indexed.observe(viewer)).unwrap()
        );
    }
    let after = g.snapshot();
    g.restore(&before).unwrap();
    assert_eq!(g.apply_index(p, revision, index), Err(Error::StaleRevision));
    act(g, action);
    assert_eq!(g.snapshot(), after);
}
fn settle(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..80 {
        if g.state.pending == Pending::Priority && g.state.stack.is_empty() {
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
                )
            })
            .unwrap();
        replay(g, action);
    }
    panic!("unsettled");
}
#[test]
fn attack_requirement_all_able_sources_must_be_selected_optional_attackers_remain_optional() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = required(&mut g, p);
        let b = required(&mut g, p);
        let optional = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let enemy = required(&mut g, p.other());
        begin(&mut g);
        assert!(!finishing(&g));
        reject(&mut g, Action::FinishAttackers);
        reject(&mut g, Action::Attack { card: enemy });
        replay(&mut g, Action::Attack { card: optional });
        assert!(!finishing(&g));
        replay(&mut g, Action::Attack { card: a });
        assert!(!finishing(&g));
        assert!(!g.object_view(a).tapped);
        reject(&mut g, Action::Attack { card: a });
        reject(&mut g, Action::FinishAttackers);
        replay(&mut g, Action::Attack { card: b });
        assert!(finishing(&g));
        replay(&mut g, Action::FinishAttackers);
        assert_eq!(g.state.attackers.len(), 3);
        assert!(
            g.state
                .attackers
                .iter()
                .all(|a| g.object_view(a.card).tapped)
        );
        assert!(g.state.selected_attackers.is_empty());
        assert!(g.state.players[p.index()].attacked_this_turn);
    }
}
#[test]
fn attack_requirement_inability_respects_tap_sickness_haste_defender_zone_control_and_type() {
    for p in [Player::P0, Player::P1] {
        for reason in 0..6 {
            let mut g = setup(p);
            let id = required(&mut g, p);
            match reason {
                0 => g.state.objects[id as usize].tapped = true,
                1 => g.state.objects[id as usize].summoning_sick = true,
                2 => g.state.objects[id as usize].gained_keywords = Keywords::DEFENDER,
                3 => g.state.objects[id as usize].controller = p.other(),
                4 => {
                    g.move_object(id, Zone::Hand);
                }
                5 => g.state.objects[id as usize].card = Card::Forest,
                _ => unreachable!(),
            }
            begin(&mut g);
            assert!(finishing(&g));
            reject(&mut g, Action::Attack { card: id });
            replay(&mut g, Action::FinishAttackers);
            assert!(g.state.attackers.is_empty());
        }
        let mut g = setup(p);
        let id = required(&mut g, p);
        g.state.objects[id as usize].summoning_sick = true;
        g.state.objects[id as usize].gained_keywords = Keywords::HASTE;
        begin(&mut g);
        assert!(!finishing(&g));
        replay(&mut g, Action::Attack { card: id });
        replay(&mut g, Action::FinishAttackers);
    }
}
#[test]
fn attack_requirement_paid_pacifism_prohibition_overrides_the_requirement() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = required(&mut g, p);
        let aura = named(&mut g, p, "Pacifism", Zone::Hand);
        cast(&mut g, aura, Some(Target::Card(id)));
        settle(&mut g);
        assert!(g.attacks_each_combat(id));
        assert!(g.combat_restricted(id, true));
        begin(&mut g);
        assert!(finishing(&g));
        reject(&mut g, Action::Attack { card: id });
        g.move_object(aura, Zone::Graveyard);
        assert!(!finishing(&g));
        replay(&mut g, Action::Attack { card: id });
        replay(&mut g, Action::FinishAttackers);
    }
}
#[test]
fn attack_requirement_paid_ability_removal_disables_requirement_and_restores_after_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = required(&mut g, p);
        let aura = named(&mut g, p, "Stop Cold", Zone::Hand);
        cast(&mut g, aura, Some(Target::Card(id)));
        settle(&mut g);
        g.untap_objects(&[id], untap_rules::UntapCause::Effect);
        assert!(!g.object_view(id).tapped);
        assert!(!g.attacks_each_combat(id));
        begin(&mut g);
        assert!(g.can_declare_attacker(p, id));
        assert!(finishing(&g));
        assert!(g.legal_actions().contains(&Action::Attack { card: id }));
        g.move_object(aura, Zone::Graveyard);
        assert!(g.attacks_each_combat(id));
        assert!(!finishing(&g));
        replay(&mut g, Action::Attack { card: id });
        replay(&mut g, Action::FinishAttackers);
    }
}
#[test]
fn attack_requirement_opposing_planeswalker_is_a_valid_choice_and_stale_or_friendly_defenders_are_rejected()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = required(&mut g, p);
        let walker = named(&mut g, p.other(), "Ajani, Caller of the Pride", Zone::Hand);
        g.move_object(walker, Zone::Battlefield);
        let friendly = named(&mut g, p, "Ajani, Caller of the Pride", Zone::Hand);
        g.move_object(friendly, Zone::Battlefield);
        begin(&mut g);
        let friendly_ref = g.object_ref(friendly);
        reject(
            &mut g,
            Action::AttackPlaneswalker {
                card: id,
                defender: friendly_ref,
            },
        );
        let reference = g.object_ref(walker);
        let before = g.snapshot();
        replay(
            &mut g,
            Action::AttackPlaneswalker {
                card: id,
                defender: reference,
            },
        );
        assert!(finishing(&g));
        replay(&mut g, Action::FinishAttackers);
        assert_eq!(
            g.state.attackers[0].defender,
            Some(CombatDefender::Planeswalker(reference))
        );
        g.restore(&before).unwrap();
        g.move_object(walker, Zone::Hand);
        g.move_object(walker, Zone::Battlefield);
        reject(
            &mut g,
            Action::AttackPlaneswalker {
                card: id,
                defender: reference,
            },
        );
        replay(&mut g, Action::Attack { card: id });
        replay(&mut g, Action::FinishAttackers);
    }
}
#[test]
fn attack_requirement_finish_rechecks_selected_declaration_and_each_combat_without_history_shortcut()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = required(&mut g, p);
        begin(&mut g);
        replay(&mut g, Action::Attack { card: id });
        let selected = g.snapshot();
        // Native boundary fixtures verify the finish gate independently of selection.
        g.state.objects[id as usize].tapped = true;
        assert!(!finishing(&g));
        reject(&mut g, Action::FinishAttackers);
        g.restore(&selected).unwrap();
        g.state.selected_attackers[0].defender = Some(CombatDefender::Player(p));
        assert!(!finishing(&g));
        reject(&mut g, Action::FinishAttackers);
        g.restore(&selected).unwrap();
        g.state
            .selected_attackers
            .push(g.state.selected_attackers[0].clone());
        assert!(!finishing(&g));
        reject(&mut g, Action::FinishAttackers);
        g.restore(&selected).unwrap();
        replay(&mut g, Action::FinishAttackers);
        // A second combat still requires this ready creature despite its attack history.
        g.untap_objects(&[id], untap_rules::UntapCause::Effect);
        begin(&mut g);
        assert!(!finishing(&g));
        replay(&mut g, Action::Attack { card: id });
        replay(&mut g, Action::FinishAttackers);
    }
}
#[test]
fn attack_requirement_new_incarnation_cannot_inherit_a_selected_declaration() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = required(&mut g, p);
        begin(&mut g);
        replay(&mut g, Action::Attack { card: id });
        let old = g.object_ref(id);
        g.move_object(id, Zone::Hand);
        g.move_object(id, Zone::Battlefield);
        assert_ne!(old, g.object_ref(id));
        assert!(g.state.selected_attackers.is_empty());
        assert!(g.object_view(id).summoning_sick);
        assert!(finishing(&g));
        g.state.objects[id as usize].summoning_sick = false;
        assert!(!finishing(&g));
        replay(&mut g, Action::Attack { card: id });
        replay(&mut g, Action::FinishAttackers);
    }
}
