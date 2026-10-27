//! Full Bore and the exact incarnation of a permanent cast for warp.
use super::*;

#[test]
fn full_bore_boosts_only_warped_creature_with_keywords() {
    for p in [Player::P0, Player::P1] {
        for warped in [false, true] {
            let mut g = scenario(p);
            let creature = put(&mut g, p, "Mightform Harmonizer", Zone::Hand);
            if warped {
                big_helpers::checked(&mut g, Action::AnnounceWarp { card: creature });
                pay(&mut g);
            } else {
                big_helpers::cast(&mut g, creature, None, &[]);
            }
            big_helpers::settle(&mut g);
            assert_eq!(g.object_view(creature).cast_for_warp, warped);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spell = put(&mut g, p, "Full Bore", Zone::Hand);
            big_helpers::checked(&mut g, Action::AnnounceCast { card: spell });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            big_helpers::checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(creature),
                },
            );
            pay(&mut g);
            big_helpers::settle(&mut g);
            assert_eq!(
                g.stats(creature),
                Stats {
                    power: 7,
                    toughness: 6
                }
            );
            let keywords = Keywords::TRAMPLE.union(Keywords::HASTE);
            assert_eq!(g.keywords(creature).contains(keywords), warped);
            g.finish_cleanup();
            assert_eq!(
                g.stats(creature),
                Stats {
                    power: 4,
                    toughness: 4
                }
            );
            assert!(!g.keywords(creature).contains(keywords));
        }
    }
}

#[test]
fn warp_marker_survives_countered_delayed_trigger_and_copy_but_not_reentry() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let creature = put(&mut g, p, "Mightform Harmonizer", Zone::Hand);
        big_helpers::checked(&mut g, Action::AnnounceWarp { card: creature });
        pay(&mut g);
        big_helpers::settle(&mut g);
        let saved = g.snapshot();
        let mut branch = g.clone();
        branch.restore(&saved).unwrap();
        assert!(branch.object_view(creature).cast_for_warp);
        g.begin_step(Step::End);
        g.give_priority();
        g.prepare_triggers();
        let delayed = g
            .state
            .stack
            .iter()
            .find(|item| item.kind == StackKind::DelayedWarpExile)
            .unwrap()
            .id;
        assert!(g.counter_stack_item(delayed, false));
        assert!(g.object_view(creature).cast_for_warp);
        let other = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let recipe = g.copiable_recipe(g.object_ref(creature));
        g.apply_temporary_copy(g.object_ref(other), recipe, CopyDeadline::EndOfTurn);
        assert!(!g.object_view(other).cast_for_warp);
        g.state.objects[creature as usize].controller = p.other();
        assert!(g.object_view(creature).cast_for_warp);
        g.move_object(creature, Zone::Hand);
        g.move_object(creature, Zone::Battlefield);
        assert!(!g.object_view(creature).cast_for_warp);
    }
}

#[test]
fn copy_of_warp_spell_resolves_as_token_without_warp_cast_marker() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let creature = put(&mut g, p, "Mightform Harmonizer", Zone::Hand);
        big_helpers::checked(&mut g, Action::AnnounceWarp { card: creature });
        pay(&mut g);
        let template = g
            .state
            .stack
            .iter()
            .find(|item| item.card == creature && item.kind == StackKind::Spell)
            .unwrap()
            .clone();
        let mut copy_effect = template.clone();
        copy_effect.copy_event = Some(Box::new(CopyEvent {
            spell: template,
            amount: 1,
        }));
        assert!(!g.copy_captured_spell(&copy_effect));
        let copy = g.state.stack.last().unwrap();
        assert!(copy.is_copy);
        let token = copy.card;
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(token).zone, Zone::Battlefield);
        assert!(g.state.objects[token as usize].is_token());
        assert!(!g.object_view(token).cast_for_warp);
        assert!(g.object_view(creature).cast_for_warp);
        let spell = put(&mut g, p, "Full Bore", Zone::Hand);
        big_helpers::cast(&mut g, spell, None, &[Target::Card(token)]);
        big_helpers::settle(&mut g);
        assert_eq!(
            g.stats(token),
            Stats {
                power: 7,
                toughness: 6
            }
        );
        assert!(!g.keywords(token).contains(Keywords::HASTE));
        assert!(!g.keywords(token).contains(Keywords::TRAMPLE));
        let delayed = &g.state.warp.as_ref().unwrap().delayed;
        assert!(delayed.iter().any(|d| d.view.source == g.object_ref(token)));
        assert!(
            delayed
                .iter()
                .any(|d| d.view.source == g.object_ref(creature))
        );
        g.begin_step(Step::End);
        g.give_priority();
        g.prepare_triggers();
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(creature).zone, Zone::Exile);
        assert_eq!(g.object_view(token).zone, Zone::Outside);
    }
}

#[test]
fn full_bore_does_nothing_when_target_changes_controller_before_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let creature = put(&mut g, p, "Mightform Harmonizer", Zone::Hand);
        big_helpers::checked(&mut g, Action::AnnounceWarp { card: creature });
        pay(&mut g);
        big_helpers::settle(&mut g);
        let spell = put(&mut g, p, "Full Bore", Zone::Hand);
        big_helpers::cast(&mut g, spell, None, &[Target::Card(creature)]);
        g.state.objects[creature as usize].controller = p.other();
        big_helpers::settle(&mut g);
        assert!(g.object_view(creature).cast_for_warp);
        assert_eq!(
            g.stats(creature),
            Stats {
                power: 4,
                toughness: 4
            }
        );
        assert!(!g.keywords(creature).contains(Keywords::HASTE));
        assert!(!g.keywords(creature).contains(Keywords::TRAMPLE));
    }
}
