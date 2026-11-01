//! Landfall targets and warp casting for Mightform Harmonizer.
use super::*;

#[test]
fn mightform_landfall_doubles_current_power_and_rejects_opponents() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Mightform Harmonizer", Zone::Battlefield);
        let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let land = put(&mut g, p, "Forest", Zone::Hand);
        g.move_object(land, Zone::Battlefield);
        g.prepare_triggers();
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
        g.state.objects[target as usize].growth.power += 1;
        g.move_object(source, Zone::Graveyard);
        big_helpers::settle(&mut g);
        assert_eq!(g.stats(target).power, 6);
        g.finish_cleanup();
        assert_eq!(g.stats(target).power, 2);
    }
}

#[test]
fn mightform_casts_for_warp_and_is_exiled_at_end_step() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Mightform Harmonizer", Zone::Hand);
        big_helpers::checked(&mut g, Action::AnnounceWarp { card: source });
        pay(&mut g);
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        g.begin_step(Step::End);
        g.give_priority();
        g.prepare_triggers();
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
    }
}

#[test]
fn mightform_doubles_negative_power_at_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Mightform Harmonizer", Zone::Battlefield);
        let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let land = put(&mut g, p, "Forest", Zone::Hand);
        g.move_object(land, Zone::Battlefield);
        g.prepare_triggers();
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        g.state.objects[target as usize].growth.power = -4;
        big_helpers::settle(&mut g);
        assert_eq!(g.stats(target).power, -4);
    }
}

#[test]
fn mightform_warp_permission_is_owner_only_delayed_and_incarnation_bound() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Mightform Harmonizer", Zone::Hand);
        big_helpers::checked(&mut g, Action::AnnounceWarp { card: source });
        pay(&mut g);
        big_helpers::settle(&mut g);
        g.begin_step(Step::End);
        g.give_priority();
        g.prepare_triggers();
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
        assert!(!g.can_cast_warped(p, source));
        g.state.turn += 1;
        assert!(g.can_cast_warped(p, source));
        assert!(!g.can_cast_warped(p.other(), source));
        g.move_object(source, Zone::Graveyard);
        g.move_object(source, Zone::Exile);
        assert!(!g.can_cast_warped(p, source));
    }
}

#[test]
fn mightform_warp_exile_does_not_bind_a_new_battlefield_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Mightform Harmonizer", Zone::Hand);
        big_helpers::checked(&mut g, Action::AnnounceWarp { card: source });
        pay(&mut g);
        big_helpers::settle(&mut g);
        g.move_object(source, Zone::Hand);
        g.move_object(source, Zone::Battlefield);
        g.begin_step(Step::End);
        g.give_priority();
        g.prepare_triggers();
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        assert!(!g.can_cast_warped(p, source));
    }
}
