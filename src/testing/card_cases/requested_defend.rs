//! Focused printed outcomes; shared engine invariants live in core tests.
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
