//! Secret Identity changes only the selected controlled creature until cleanup.
use super::*;

#[test]
fn secret_identity_modes_preserve_counters_and_restore_characteristics() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..=1 {
            let mut g = scenario(p);
            let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.objects[target as usize].counters.plus_one = 1;
            let spell = put(&mut g, p, "Secret Identity", Zone::Hand);
            big_helpers::checked(&mut g, Action::AnnounceCast { card: spell });
            big_helpers::checked(&mut g, Action::ChooseMode { mode });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            big_helpers::checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            pay(&mut g);
            big_helpers::settle(&mut g);
            let (power, toughness, subtype, keywords) = if mode == 0 {
                (2, 2, "Citizen", Keywords::HEXPROOF)
            } else {
                (4, 5, "Hero", Keywords::FLYING.union(Keywords::VIGILANCE))
            };
            assert_eq!(
                (g.stats(target).power, g.stats(target).toughness),
                (power, toughness)
            );
            assert!(g.has_subtype(target, subtype));
            assert!(!g.has_subtype(target, "Bear"));
            assert!(g.keywords(target).contains(keywords));
            g.finish_cleanup();
            assert_eq!((g.stats(target).power, g.stats(target).toughness), (3, 3));
            assert!(g.has_subtype(target, "Bear"));
            assert!(!g.has_subtype(target, subtype));
            assert!(!g.keywords(target).contains(keywords));
        }
    }
}

#[test]
fn secret_identity_stale_target_does_not_modify_new_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Secret Identity", Zone::Hand);
        big_helpers::cast(&mut g, spell, Some(1), &[Target::Card(target)]);
        g.move_object(target, Zone::Hand);
        g.move_object(target, Zone::Battlefield);
        big_helpers::settle(&mut g);
        assert_eq!((g.stats(target).power, g.stats(target).toughness), (2, 2));
        assert!(g.has_subtype(target, "Bear"));
        assert!(!g.has_subtype(target, "Hero"));
        assert!(!g.keywords(target).contains(Keywords::FLYING));
    }
}

#[test]
fn secret_identity_preserves_artifact_type_and_original_abilities() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let target = put(&mut g, p, "Darksteel Colossus", Zone::Battlefield);
        let spell = put(&mut g, p, "Secret Identity", Zone::Hand);
        big_helpers::cast(&mut g, spell, Some(0), &[Target::Card(target)]);
        big_helpers::settle(&mut g);
        assert_eq!(g.current_kinds(target) & (1 | 16), 1 | 16);
        assert!(
            g.keywords(target).contains(
                Keywords::INDESTRUCTIBLE
                    .union(Keywords::TRAMPLE)
                    .union(Keywords::HEXPROOF)
            )
        );
        assert!(g.has_subtype(target, "Citizen"));
        assert!(!g.has_subtype(target, "Golem"));
        g.finish_cleanup();
        assert!(g.has_subtype(target, "Golem"));
        assert!(
            g.keywords(target)
                .contains(Keywords::INDESTRUCTIBLE.union(Keywords::TRAMPLE))
        );
    }
}

#[test]
fn secret_identity_type_replacement_applies_after_changeling() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let target = put(&mut g, p, "Changeling Wayfinder", Zone::Battlefield);
        assert!(g.has_subtype(target, "Bear"));
        let spell = put(&mut g, p, "Secret Identity", Zone::Hand);
        big_helpers::cast(&mut g, spell, Some(1), &[Target::Card(target)]);
        big_helpers::settle(&mut g);
        assert!(g.has_subtype(target, "Hero"));
        assert!(!g.has_subtype(target, "Bear"));
        assert!(g.keywords(target).contains(Keywords::CHANGELING));
        g.finish_cleanup();
        assert!(g.has_subtype(target, "Bear"));
    }
}
