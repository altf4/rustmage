//! Focused printed outcomes; shared engine invariants live in core tests.
//! Oracle outcomes for optional graveyard targeting and paid graveyard recovery.
use super::*;

#[test]
fn abandoned_gollum_graveyard_recovery_pays_two_and_artifact_or_creature_as_sorcery() {
    {
        let p = Player::P0;
        for fuel_name in ["Grizzly Bears", "Goldvein Pick"] {
            let mut g = scenario(p);
            let gollum = put(&mut g, p, "Gollum the Abandoned", Zone::Graveyard);
            let fuel = put(&mut g, p, fuel_name, Zone::Battlefield);
            let wrong = put(&mut g, p, "Pacifism", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), fuel_name, Zone::Battlefield);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
            let start = Action::AnnounceActivate {
                card: gollum,
                ability: 0,
                counter_payment: Counters::default(),
            };
            g.state.active = p.other();
            reject(&mut g, start.clone());
            g.state.active = p;
            act(&mut g, start);
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            for id in [wrong, enemy] {
                let object = g.object_ref(id);
                reject(&mut g, Action::ChooseAdditionalCost { object });
            }

            let object = g.object_ref(fuel);
            act(&mut g, Action::ChooseAdditionalCost { object });
            pay(&mut g);
            assert_eq!(g.object_view(fuel).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            settle(&mut g);
            assert_eq!(g.object_view(gollum).zone, Zone::Hand);
        }
    }
}
