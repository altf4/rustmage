//! Behold witnesses must follow speculative payment state rather than the parent.
use super::*;
use crate::engine::assurance::big_helpers::{named, setup};

#[test]
fn projected_departure_and_control_change_remove_behold_witness() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let source = g.object_ref(elf);
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        };
        let before = g.snapshot();
        let world = FundingState::from_game(&g, &goal);
        assert!(hand_choices::behold_candidates(&world, p, "Elf", &[]).contains(&source));
        let mut departed = world.clone();
        let resource = departed.resources.iter_mut().find(|r| r.id == elf).unwrap();
        resource.object.zone = Zone::Graveyard;
        assert!(!hand_choices::behold_candidates(&departed, p, "Elf", &[]).contains(&source));
        let mut stolen = world.clone();
        let resource = stolen.resources.iter_mut().find(|r| r.id == elf).unwrap();
        resource.object.controller = p.other();
        assert!(!hand_choices::behold_candidates(&stolen, p, "Elf", &[]).contains(&source));
        assert_eq!(g.snapshot(), before);
    }
}

#[test]
fn projected_sanctum_entry_explores_behold_and_tapped_routes() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = named(&mut g, p, "Island", Zone::Hand);
        g.state.objects[land as usize].card = Card::from_name("Theorist's Sanctum").unwrap();
        let witness = named(&mut g, p, "Island", Zone::Hand);
        g.state.objects[witness as usize].card = Card::from_name("Jace, Reality Sculptor").unwrap();
        let before = g.snapshot();
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        };
        let mut world = FundingState::from_game(&g, &goal);
        world.prepare_entry_projection();
        let movement = entries::EntryMove::from_view(g.object_view(land), Zone::Battlefield, false);
        let mut tapped = false;
        let mut untapped = false;
        world.visit_entry_moves(vec![movement], true, None, &mut |after| {
            let object = after.object(land);
            if object.zone == Zone::Battlefield {
                tapped |= object.tapped;
                untapped |= !object.tapped;
                assert_eq!(after.intrinsic_mana_color(land), Some(1));
            }
            false
        });
        assert!(tapped && untapped);
        assert_eq!(g.snapshot(), before);
    }
}

#[test]
fn projected_sanctum_respects_order_of_behold_and_untapped_replacements() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = named(&mut g, p, "Island", Zone::Hand);
        g.state.objects[land as usize].card = Card::from_name("Theorist's Sanctum").unwrap();
        named(&mut g, p, "Spelunking", Zone::Battlefield);
        let before = g.snapshot();
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        };
        let mut world = FundingState::from_game(&g, &goal);
        world.prepare_entry_projection();
        let movement = entries::EntryMove::from_view(g.object_view(land), Zone::Battlefield, false);
        let mut tapped = false;
        let mut untapped = false;
        world.visit_entry_moves(vec![movement], true, None, &mut |after| {
            let object = after.object(land);
            if object.zone == Zone::Battlefield {
                tapped |= object.tapped;
                untapped |= !object.tapped;
            }
            false
        });
        assert!(tapped && untapped);
        assert_eq!(g.snapshot(), before);
    }
}
