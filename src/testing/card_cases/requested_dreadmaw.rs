//! Focused printed outcomes; shared engine invariants live in core tests.
//! Dreadmaw's Ire grants a combat trigger to the exact attacking creature.
use super::*;

#[test]
fn dreadmaw_targets_attacker_and_grants_damage_trigger_for_both_seats() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let attacker = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let idle = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let artifact = put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
        let own_artifact = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        g.state.objects[attacker as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        let spell = put(&mut g, p, "Dreadmaw's Ire", Zone::Hand);
        big_helpers::checked(&mut g, Action::AnnounceCast { card: spell });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(idle),
            },
        );
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(attacker),
            },
        );
        pay(&mut g);
        big_helpers::settle(&mut g);
        assert_eq!(
            g.stats(attacker),
            Stats {
                power: 4,
                toughness: 4
            }
        );
        assert!(g.keywords(attacker).contains(Keywords::TRAMPLE));

        let source = g.damage_source(attacker);
        g.apply_damage_batch(&[DamageEvent {
            source,
            target: Target::Player(p.other()),
            amount: 1,
            combat: true,
            preventable: true,
        }]);
        g.prepare_triggers();
        for _ in 0..30 {
            if matches!(g.state.pending, Pending::Targets { .. }) {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own_artifact),
            },
        );
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(artifact),
            },
        );
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(artifact).zone, Zone::Graveyard);
        g.finish_cleanup();
        assert!(!g.keywords(attacker).contains(Keywords::TRAMPLE));
    }
}

fn dreadmaw_setup(p: Player) -> (Game, CardId, CardId) {
    let mut g = scenario(p);
    let attacker = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let artifact = put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
    g.state.objects[attacker as usize].summoning_sick = false;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    big_helpers::checked(&mut g, Action::Attack { card: attacker });
    big_helpers::checked(&mut g, Action::FinishAttackers);
    let spell = put(&mut g, p, "Dreadmaw's Ire", Zone::Hand);
    big_helpers::cast(&mut g, spell, None, &[Target::Card(attacker)]);
    big_helpers::settle(&mut g);
    (g, attacker, artifact)
}

#[test]
fn dreadmaw_only_triggers_for_combat_and_does_not_follow_a_new_incarnation() {
    {
        let p = Player::P0;
        let (mut g, attacker, _) = dreadmaw_setup(p);
        let first = g.object_ref(attacker);
        let source = g.damage_source(attacker);
        let before = g.state.pending_triggers.len();
        g.apply_damage_batch(&[DamageEvent {
            source,
            target: Target::Player(p.other()),
            amount: 1,
            combat: false,
            preventable: true,
        }]);
        assert_eq!(g.state.pending_triggers.len(), before);
        g.move_object(attacker, Zone::Exile);
        g.move_object(attacker, Zone::Battlefield);
        assert_ne!(g.object_ref(attacker), first);
        let source = g.damage_source(attacker);
        g.apply_damage_batch(&[DamageEvent {
            source,
            target: Target::Player(p.other()),
            amount: 1,
            combat: true,
            preventable: true,
        }]);
        assert_eq!(g.state.pending_triggers.len(), before);
    }
}
