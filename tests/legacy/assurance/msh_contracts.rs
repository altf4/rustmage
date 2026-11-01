//! Representative owned decisions; card scenarios do not repeat these contracts.
use super::contracts::branch_contract;
use super::msh_a::{attack, library, resolved, until};
use super::*;

#[test]
fn msh_owned_teamwork_and_partial_crew_decisions_survive_branch_and_restore() {
    let p = Player::P0;
    let mut g = scenario(p);
    let one_power = put(&mut g, p, "Aerial Doombot", Zone::Battlefield);
    put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let card = put(&mut g, p, "Cruel Alliance", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card });
    act(&mut g, Action::ChooseTeamwork { pay: true });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    pay(&mut g);
    let view = g.observe(p).cast_costs.unwrap();
    let clause = view
        .clauses
        .iter()
        .position(|r| matches!(r.clause, CastCostClause::Teamwork(_)))
        .unwrap() as u16;
    act(&mut g, Action::ChooseCastCost { clause });
    let object = g.object_ref(one_power);
    act(&mut g, Action::ChooseTeammate { object });
    assert!(
        !g.legal_actions().contains(&Action::FinishTeamwork),
        "one power cannot pay Teamwork 2"
    );
    branch_contract(g);

    let mut g = scenario(Player::P1);
    let jet = put(&mut g, Player::P1, "Dependable Quinjet", Zone::Battlefield);
    let giant = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    act(
        &mut g,
        Action::AnnounceActivate {
            card: jet,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    let object = g.object_ref(giant);
    act(&mut g, Action::ChooseCrewer { object });
    assert!(
        !g.legal_actions().contains(&Action::FinishCrew),
        "three power cannot crew four"
    );
    branch_contract(g);
}

#[test]
fn msh_private_library_cast_root_has_owned_branches_and_private_observations() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let top = library(
            &mut g,
            p,
            &[
                "Agent Maria Hill",
                "Forest",
                "Forest",
                "Forest",
                "Forest",
                "Forest",
                "Mountain",
            ],
        );
        put(&mut g, p, "Cosmic Cube", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        attack(&mut g, &[bear]);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::FinishResolutionCasts)
        });
        assert_eq!(
            g.observe(p).library_cast_look.as_ref().unwrap().cards.len(),
            6
        );
        assert!(g.observe(p.other()).library_cast_look.is_none());
        assert!(g.observe(p.other()).resolution_free_casts.is_none());
        assert!(
            !g.observe(p.other())
                .objects
                .iter()
                .any(|o| top.contains(&o.id))
        );
        branch_contract(g);
    }
}

#[test]
fn msh_controlled_payer_mana_conversion_uses_resource_owner_not_decision_controller() {
    // The entitlement is the fixture seam. Kingpin's cast trigger, Baxter's paid
    // conversion and extort payment all run through ordinary checked actions.
    for payer in [Player::P0, Player::P1] {
        for external_pool in [[0; 6], [0, 0, 0, 0, 0, 9]] {
            let controller = payer.other();
            let mut g = scenario(payer);
            let secret = put(&mut g, payer, "Shock", Zone::Hand);
            assert!(!g.observe(controller).objects.iter().any(|o| o.id == secret));
            g.schedule_player_control(payer, controller);
            g.begin_turn();
            g.state.step = Step::Main1;
            g.state.priority = payer;
            assert_eq!(g.actor(), Some(controller));
            assert_eq!(g.semantic_actor(), Some(payer));
            assert!(g.observe(controller).objects.iter().any(|o| o.id == secret));
            put(&mut g, payer, "The Kingpin of Crime", Zone::Battlefield);
            let baxter = put(&mut g, payer, "Baxter Building", Zone::Battlefield);
            g.state.players[payer.index()].mana = [0, 0, 0, 0, 0, 4];
            g.state.players[controller.index()].mana = external_pool;
            let stone = put(&mut g, payer, "Mox Jasper", Zone::Hand);
            cast(&mut g, stone, None);
            assert_eq!(g.state.players[payer.index()].mana, [0, 0, 0, 0, 0, 4]);
            until(&mut g, |g| {
                g.legal_actions().contains(&Action::DeclineResolutionMana)
            });
            assert_eq!(
                g.observe(controller).resolution_payment.unwrap().payer,
                payer
            );
            if payer == Player::P0 && external_pool == [0; 6] {
                branch_contract(g.clone());
            }
            let actions = g.legal_actions();
            let index = crate::policy::smoke_index(&g, &actions);
            assert!(
                matches!(&actions[index], Action::ActivateMana { card, payment: [0,0,0,0,0,4], production, .. }
                if *card == baxter && production.iter().copied().sum::<u32>() == 4 && production[0] + production[2] > 0)
            );
            g.apply_index(controller, g.revision(), index).unwrap();
            assert!(g.object_view(baxter).tapped);
            assert_eq!(g.state.players[controller.index()].mana, external_pool);
            let actions = g.legal_actions();
            let index = crate::policy::smoke_index(&g, &actions);
            assert!(matches!(&actions[index], Action::PayResolutionMana { .. }));
            g.apply_index(controller, g.revision(), index).unwrap();
            resolved(&mut g);
            assert_eq!(g.state.players[payer.index()].life, 21);
            assert_eq!(g.state.players[controller.index()].life, 19);
            assert_eq!(
                g.state.players[payer.index()]
                    .mana
                    .iter()
                    .copied()
                    .sum::<u32>(),
                3
            );
            assert_eq!(g.state.players[controller.index()].mana, external_pool);
            assert_eq!(g.object_view(stone).controller, payer);
        }
    }
}
