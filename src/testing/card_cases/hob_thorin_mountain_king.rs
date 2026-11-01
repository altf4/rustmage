//! Focused printed outcomes; shared engine invariants live in core tests.
//! Actual attachments produce one independent damage trigger with an exact actor.
use super::*;
use crate::cards::{AnimationCreatureTypes, AnimationDuration, AnimationSpec};
const FIXTURES: &[&str] = &[
    "Thorin, Mountain-king",
    "Bear Cub",
    "Goldvein Pick",
    "Lavaspur Boots",
    "Basilisk Collar",
    "Prophetic Prism",
    "Aggressive Mammoth",
    "Giant Growth",
    "Unsummon",
    "Shock",
    "Darksteel Colossus",
    "Forest",
];
fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    assert!(FIXTURES.contains(&name), "unlisted fixture {name}");
    put(g, p, name, zone)
}
fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.resolving.is_none()
}
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..48 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Thorin decision budget {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    until(g, quiet);
}
fn target(g: &mut Game, id: CardId) {
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(id),
        },
    );
}
fn targets(g: &mut Game) {
    until(g, |g| matches!(g.state.pending, Pending::Targets { .. }));
}
fn paid(g: &mut Game, p: Player) -> CardId {
    let id = fixture(g, p, "Thorin, Mountain-king", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 3];
    cast(g, id, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    targets(g);
    assert_eq!(stats(g, id), (3, 4));
    assert!(g.has_subtype(id, "Dwarf") && g.has_subtype(id, "Noble"));
    assert!(g.object_view(id).is_legendary());

    id
}
fn select(g: &mut Game, equipment: &[CardId], recipient: CardId) {
    for &e in equipment {
        target(g, e);
    }
    act(g, Action::FinishTargets);
    target(g, recipient);
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
fn reflexive(g: &mut Game) {
    targets(g);
    assert!(matches!(
        g.state.announcing.as_ref().unwrap().item.kind,
        StackKind::TriggeredAbility { ability: 1 }
    ));
}
fn damage_target(g: &mut Game, id: Option<CardId>) {
    if let Some(id) = id {
        target(g, id);
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
fn response(g: &mut Game, p: Player, name: &str, target_id: CardId, mana: Mana) {
    // The trigger has just been placed: priority belongs to the active player.
    assert_eq!(g.actor(), Some(p));
    let spell = fixture(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    cast(g, spell, Some(Target::Card(target_id)));
}
#[test]
fn thorin_paid_etb_any_number_owned_equipment_one_damage_trigger_and_actual_trample() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let bear = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        let a = fixture(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        let b = fixture(&mut g, p, "Lavaspur Boots", Zone::Battlefield);
        let enemy = fixture(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
        let prism = fixture(&mut g, p, "Prophetic Prism", Zone::Battlefield);
        let mammoth = fixture(&mut g, p.other(), "Aggressive Mammoth", Zone::Battlefield);
        let thorin = paid(&mut g, p);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(prism),
            },
        );
        target(&mut g, a);
        target(&mut g, b);
        act(&mut g, Action::FinishTargets);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(mammoth),
            },
        );
        target(&mut g, bear);
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EquipmentAttachment { .. })
        });
        assert_eq!(g.object_view(a).attached_to, None);
        assert_eq!(g.object_view(b).attached_to, None);
        let equipment = g.object_ref(b);
        act(
            &mut g,
            Action::ChooseEquipmentAttachment {
                equipment: Some(equipment),
            },
        );
        reflexive(&mut g);
        damage_target(&mut g, Some(mammoth));
        assert_eq!(g.state.stack.len(), 1);
        finish(&mut g);
        assert_eq!(g.object_view(a).attached_to, Some(g.object_ref(bear)));
        assert_eq!(g.object_view(b).attached_to, Some(g.object_ref(bear)));
        assert_eq!(stats(&g, bear), (4, 3));
        assert_eq!(g.object_view(mammoth).damage, 4);
        assert_eq!(g.object_view(bear).damage, 0);
        assert!(!g.state.players[p.index()].equip_activated_this_turn);
        let blocker = fixture(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        g.state.objects[thorin as usize].summoning_sick = false;
        act(&mut g, Action::Attack { card: thorin });
        act(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        act(
            &mut g,
            Action::Block {
                blocker,
                attacker: thorin,
            },
        );
        act(&mut g, Action::FinishBlockers);
        for _ in 0..24 {
            if g.state.step == Step::EndCombat {
                break;
            }
            if let Some(range) = g.damage_assignment_range() {
                act(
                    &mut g,
                    Action::AssignDamage {
                        target: range.target,
                        amount: range.minimum,
                    },
                );
            } else {
                act(&mut g, Action::Pass);
            }
        }
        assert_eq!(g.object_view(blocker).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.other().index()].life, 19);
    }
}
#[test]
fn thorin_zero_equipment_already_attached_and_unattachable_creature_equipment_do_not_trigger() {
    for mode in 0..3 {
        let p = Player::P0;
        let mut g = scenario(p);
        let bear = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        let e = fixture(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        if mode == 1 {
            assert!(g.attach_equipment(g.object_ref(e), g.object_ref(bear)));
        }
        if mode == 2 {
            g.change_object_form(
                g.object_ref(e),
                AnimationSpec {
                    artifact: true,
                    creature_types: AnimationCreatureTypes::Unchanged,
                    base: Some((1, 1)),
                    keywords: Keywords::NONE,
                    duration: AnimationDuration::EndOfTurn,
                },
                characteristic_layers::AnimationTypeMode::AddCreature,
            );
        }
        paid(&mut g, p);
        select(
            &mut g,
            if mode == 0 {
                &[]
            } else {
                std::slice::from_ref(&e)
            },
            bear,
        );
        finish(&mut g);
        assert_eq!(
            g.object_view(e).attached_to,
            if mode == 1 {
                Some(g.object_ref(bear))
            } else {
                None
            }
        );
    }
}

#[test]
fn thorin_damage_reads_current_power_and_recipient_is_damage_source() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let bear = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        let e = fixture(&mut g, p, "Basilisk Collar", Zone::Battlefield);
        let colossus = fixture(&mut g, p.other(), "Darksteel Colossus", Zone::Battlefield);
        paid(&mut g, p);
        select(&mut g, &[e], bear);
        reflexive(&mut g);
        damage_target(&mut g, Some(colossus));
        response(&mut g, p, "Giant Growth", bear, [0, 0, 0, 0, 1, 0]);
        finish(&mut g);
        assert_eq!(g.object_view(colossus).damage, 5);
        assert_eq!(g.state.players[p.index()].life, 25);
        assert_eq!(g.object_view(bear).damage, 0);
    }
}
