//! Defend the Rider and the Pilot's crew/saddle contribution.
use super::*;

fn pilot(g: &mut Game, p: Player) -> CardId {
    let spell = put(g, p, "Defend the Rider", Zone::Hand);
    big_helpers::cast(g, spell, Some(1), &[]);
    big_helpers::settle(g);
    g.state
        .objects
        .iter()
        .enumerate()
        .find(|(_, o)| {
            o.zone == Zone::Battlefield
                && o.controller == p
                && o.card == Card::from_name("@token/pilot_1_1_colorless").unwrap()
        })
        .map(|(id, _)| id as CardId)
        .unwrap()
}

#[test]
fn defend_pilot_pays_crew_and_saddle_without_changing_actual_power() {
    for p in [Player::P0, Player::P1] {
        for crew in [false, true] {
            let mut g = scenario(p);
            let token = pilot(&mut g, p);
            let source = put(
                &mut g,
                p,
                if crew {
                    "Great Gilded Boat"
                } else {
                    "Quilled Charger"
                },
                Zone::Battlefield,
            );
            assert_eq!(g.stats(token).power, 1);
            big_helpers::checked(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            let object = g.object_ref(token);
            if crew {
                big_helpers::checked(&mut g, Action::ChooseCrewer { object });
                big_helpers::checked(&mut g, Action::FinishCrew);
            } else {
                big_helpers::checked(&mut g, Action::ChooseSaddler { object });
                big_helpers::checked(&mut g, Action::FinishSaddle);
            }
            big_helpers::settle(&mut g);
            assert!(g.object_view(token).tapped);
            assert_eq!(g.stats(token).power, 1);
        }
    }
}

#[test]
fn defend_protects_controlled_permanent_until_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let land = put(&mut g, p, "Forest", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Forest", Zone::Battlefield);
        let spell = put(&mut g, p, "Defend the Rider", Zone::Hand);
        big_helpers::checked(&mut g, Action::AnnounceCast { card: spell });
        big_helpers::checked(&mut g, Action::ChooseMode { mode: 0 });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        pay(&mut g);
        big_helpers::settle(&mut g);
        assert!(
            g.keywords(land)
                .contains(Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE))
        );
        g.finish_cleanup();
        assert!(!g.keywords(land).contains(Keywords::HEXPROOF));
    }
}

#[test]
fn pilot_copy_and_ability_removal_affect_fundability_without_parent_mutation() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let token = pilot(&mut g, p);
        let vehicle = put(&mut g, p, "Great Gilded Boat", Zone::Battlefield);
        let source = g.object_ref(vehicle);
        assert!(g.crew_fundable(source, p, 2));
        let before = g.snapshot();
        let mut branch = g.clone();
        let copy = put(&mut branch, p, "Bear Cub", Zone::Battlefield);
        let recipe = branch.copiable_recipe(branch.object_ref(token));
        branch.apply_temporary_copy(branch.object_ref(copy), recipe, CopyDeadline::EndOfTurn);
        assert_eq!(branch.crew_saddle_power(copy, vehicle, "Vehicle"), 3);
        let removal = put(&mut branch, p, "Witness Protection", Zone::Hand);
        big_helpers::cast(&mut branch, removal, None, &[Target::Card(token)]);
        big_helpers::settle(&mut branch);
        assert_eq!(branch.crew_saddle_power(token, vehicle, "Vehicle"), 1);
        assert_eq!(g.snapshot(), before);
        branch.restore(&before).unwrap();
        assert!(branch.crew_fundable(source, p, 2));
    }
}

#[test]
fn pilot_threshold_three_requires_mount_or_vehicle_type() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let token = pilot(&mut g, p);
        let vehicle = put(&mut g, p, "Great Gilded Boat", Zone::Battlefield);
        let v = g.object_ref(vehicle);
        assert!(g.crew_fundable(v, p, 3));
        let mount = put(&mut g, p, "Quilled Charger", Zone::Battlefield);
        let m = g.object_ref(mount);
        // The Mount cannot contribute to itself, and the Pilot contributes 3.
        assert!(g.saddle_fundable(m, p, 3));
        assert_eq!(g.crew_saddle_power(token, vehicle, "Vehicle"), 3);
        assert_eq!(g.crew_saddle_power(token, mount, "Mount"), 3);
        let recipe = CopyRecipe::new(Card::from_name("Bear Cub").unwrap());
        g.apply_temporary_copy(v, recipe, CopyDeadline::EndOfTurn);
        g.apply_temporary_copy(m, recipe, CopyDeadline::EndOfTurn);
        assert_eq!(g.crew_saddle_power(token, vehicle, "Vehicle"), 1);
        assert_eq!(g.crew_saddle_power(token, mount, "Mount"), 1);
    }
}
