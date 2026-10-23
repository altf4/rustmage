//! Fixed one-shot control core. Printed consumers stay gated until their whole cards work.
use super::ability_timeline_tests::{cast, checked, named, next_turn, settle, setup};
use super::*;

const SOURCE_TO_OPPONENT: &[Instruction] = &[Instruction::GainControl {
    subject: Subject::Source,
    controller: PlayerSet::Opponent,
}];
const TARGET_TO_CONTROLLER: &[Instruction] = &[Instruction::GainControl {
    subject: Subject::Targets,
    controller: PlayerSet::Controller,
}];
const TARGET_TO_PLAYER: &[Instruction] = &[Instruction::GainControl {
    subject: Subject::Targets,
    controller: PlayerSet::Targeted,
}];

// Execute generic typed core fixtures through the same production dispatcher;
// no temporary printed definition or unsupported-card admission is involved.
fn resolve(g: &mut Game, item: SpellView, program: &'static [Instruction]) {
    g.state.resolving = Some(Resolution {
        wheel: None,
        continuation: None,
        chosen_creature_type: None,
        chosen_color: None,
        explore: None,
        evidence: None,
        connive: None,
        item,
        program,
        pc: 0,
        target_group: 0,
        moved_count: None,
        discards: [0; 2],
        mana_payment: None,
        forage: None,
        selected_discards: [Vec::new(), Vec::new()],
        milled: Vec::new(),
        milled_selection: None,
        library: None,
        selection: None,
        target_library: None,
        search: None,
        optional_searches: None,
        choice: None,
    });
    g.run_program();
}
fn target_item(g: &mut Game, p: Player, source: CardId, target: CardId) -> SpellView {
    let mut item = g.activation_item(p, source, 0);
    item.targets.push(g.bind_target(Target::Card(target)));
    item
}
fn replay(g: &mut Game, item: SpellView, program: &'static [Instruction]) {
    let before = g.snapshot();
    let mut branch = {
        let parent = g.clone();
        parent.clone()
    };
    resolve(&mut branch, item.clone(), program);
    resolve(g, item.clone(), program);
    assert_eq!(g.snapshot(), branch.snapshot());
    for p in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(g.observe(p)).unwrap(),
            serde_json::to_value(branch.observe(p)).unwrap()
        );
    }
    let after = g.snapshot();
    g.restore(&before).unwrap();
    resolve(g, item, program);
    assert_eq!(g.snapshot(), after);
    // A real subsequent action traverses owned raw/index branches after the
    // original source parent is disposed, and checks restore and stale revisions.
    checked(g, Action::Pass);
}

#[test]
fn indefinite_control_supersedes_older_eot_even_when_controller_already_matches() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let victim = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p, "Involuntary Employment", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Card(victim)]);
        settle(&mut g);
        assert_eq!(g.object_view(victim).controller, p);
        assert_eq!(g.state.temporary_controls.as_ref().unwrap().len(), 1);
        // A second ordinary temporary effect has the same current controller.
        let item = target_item(&mut g, p, victim, victim);
        resolve(&mut g, item, &[Instruction::GainControlUntilEndTurn]);
        assert_eq!(g.state.temporary_controls.as_ref().unwrap().len(), 2);
        g.state.objects[victim as usize].summoning_sick = false;
        let item = target_item(&mut g, p, victim, victim);
        replay(&mut g, item, TARGET_TO_CONTROLLER);
        assert!(g.state.temporary_controls.is_none());
        assert!(!g.object_view(victim).summoning_sick);
        next_turn(&mut g);
        assert_eq!(g.object_view(victim).controller, p);
        assert_eq!(g.object_view(victim).owner, p.other());
    }
}

#[test]
fn later_temporary_controls_restore_the_lasting_baseline_and_zone_change_resets_it() {
    for p in [Player::P0, Player::P1] {
        for leaves in [false, true] {
            let mut g = setup(p);
            let victim = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let source = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let item = target_item(&mut g, p, source, victim);
            replay(&mut g, item, TARGET_TO_CONTROLLER);
            // The giver leaving does not cancel an indefinite transfer.
            g.move_object(source, Zone::Graveyard);
            for controller in [p.other(), p] {
                let item = target_item(&mut g, controller, victim, victim);
                resolve(&mut g, item, &[Instruction::GainControlUntilEndTurn]);
            }
            assert_eq!(g.state.temporary_controls.as_ref().unwrap().len(), 2);
            if leaves {
                let old = g.object_ref(victim);
                g.move_object(victim, Zone::Exile);
                g.move_object(victim, Zone::Battlefield);
                assert_ne!(old, g.object_ref(victim));
                assert!(g.state.temporary_controls.is_none());
            }
            next_turn(&mut g);
            assert_eq!(
                g.object_view(victim).controller,
                if leaves { p.other() } else { p }
            );
            assert_eq!(g.object_view(victim).owner, p.other());
            assert!(g.state.temporary_controls.is_none());
        }
    }
}

#[test]
fn nontargeted_source_transfer_ignores_hexproof_captures_controller_and_removes_combat() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, source, None, &[]);
        settle(&mut g);
        g.state.objects[source as usize].summoning_sick = false;
        g.state.objects[source as usize].tapped = true;
        g.state.players[p.other().index()].hexproof_until_end_turn = true;
        g.state.attackers.push(Attacker {
            card: source,
            defender: Some(CombatDefender::Player(p.other())),
            blocked: false,
        });
        let item = g.activation_item(p, source, 0);
        replay(&mut g, item.clone(), SOURCE_TO_OPPONENT);
        assert_eq!(g.object_view(source).controller, p.other());
        assert_eq!(g.object_view(source).owner, p);
        assert!(g.object_view(source).summoning_sick);
        assert!(g.object_view(source).tapped); // Control alone does not untap.
        assert!(g.state.attackers.is_empty());
        // The packet's captured controller still determines its opponent even
        // though the live source has changed controller since its creation.
        replay(&mut g, item, SOURCE_TO_OPPONENT);
        assert_eq!(g.object_view(source).controller, p.other());
        next_turn(&mut g);
        assert_eq!(g.object_view(source).controller, p.other());
    }
}

#[test]
fn two_group_targets_need_both_valid_recipients_and_never_follow_reincarnations() {
    for p in [Player::P0, Player::P1] {
        for invalid in 0..4 {
            let mut g = setup(p);
            let source = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let recipient = named(&mut g, p, "Mountain", Zone::Battlefield);
            let mut item = g.activation_item(p, source, 0);
            if invalid != 1 {
                item.targets.push(g.bind_target(Target::Player(p.other())));
            }
            if invalid != 2 {
                let mut bound = g.bind_target(Target::Card(recipient));
                bound.group = 1;
                item.targets.push(bound);
            }
            if invalid == 3 {
                g.move_object(recipient, Zone::Exile);
                g.move_object(recipient, Zone::Battlefield);
            }
            // Missing entries represent independently invalid targets already
            // removed by resolve_top. Valid player and permanent are in distinct
            // groups, which must both survive this instruction's dispatch.
            replay(&mut g, item, TARGET_TO_PLAYER);
            assert_eq!(
                g.object_view(recipient).controller,
                if invalid == 0 { p.other() } else { p }
            );
            assert_eq!(g.object_view(recipient).owner, p);
        }
    }
}

#[test]
fn activated_source_departure_or_new_incarnation_is_not_an_indefinite_recipient() {
    for p in [Player::P0, Player::P1] {
        for returns in [false, true] {
            let mut g = setup(p);
            let source = named(&mut g, p, "Fanatical Firebrand", Zone::Battlefield);
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            while g.state.announcing.is_some() {
                let action = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                    .unwrap();
                checked(&mut g, action);
            }
            assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            let item = g.state.stack.pop().unwrap();
            if returns {
                g.move_object(source, Zone::Battlefield);
                assert_ne!(g.object_ref(source), item.object);
            }
            replay(&mut g, item, SOURCE_TO_OPPONENT);
            assert_eq!(g.object_view(source).controller, p);
            assert_eq!(
                g.object_view(source).zone,
                if returns {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
        }
    }
}
