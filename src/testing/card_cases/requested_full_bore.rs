//! Focused printed outcomes; shared engine invariants live in core tests.
//! Full Bore and the exact incarnation of a permanent cast for warp.
use super::*;

#[test]
fn full_bore_boosts_only_warped_creature_with_keywords() {
    {
        let p = Player::P0;
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
fn full_bore_does_nothing_when_target_changes_controller_before_resolution() {
    {
        let p = Player::P0;
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
