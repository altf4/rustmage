//! Focused printed outcomes; shared engine invariants live in core tests.
//! Landfall targets and warp casting for Mightform Harmonizer.
use super::*;

#[test]
fn mightform_landfall_doubles_current_power_and_rejects_opponents() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
