//! Both faces and their emblem through ordinary actions. Enable after admission.
use super::big_helpers::{cast as paid_cast, checked, settle as finish};
use super::*;

fn activate_oko(g: &mut Game, oko: CardId, ability: u16, target: Option<CardId>) {
    checked(
        g,
        Action::AnnounceActivate {
            card: oko,
            ability,
            counter_payment: Counters::default(),
        },
    );
    if let Some(card) = target {
        checked(
            g,
            Action::ChooseTarget {
                target: Target::Card(card),
            },
        );
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        checked(g, Action::FinishTargets);
    }
    pay(g);
}

fn pay_for_transform(g: &mut Game) {
    g.begin_step(Step::Main1);
    g.give_priority();
    g.prepare_triggers();
    g.resolve_top();
    choose(g, |action| {
        matches!(action, Action::PayResolutionMana { .. })
    });
    finish(g);
}

#[test]
fn oko_lorwyn_liege_whole_card() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let oko = put(&mut g, player, "Oko, Lorwyn Liege", Zone::Hand);
        paid_cast(&mut g, oko, None, &[]);
        finish(&mut g);
        assert_eq!(g.counter_count(oko, CounterKind::Loyalty), 3);
        let bear = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let mut plus_two = g.clone();
        activate_oko(&mut plus_two, oko, 0, Some(bear));
        finish(&mut plus_two);
        assert!(plus_two.object_view(bear).has_subtype("Dragon"));
        assert!(plus_two.object_view(bear).has_subtype("Elf"));
        plus_two.finish_cleanup();
        assert!(plus_two.object_view(bear).has_subtype("Dragon"));
        activate_oko(&mut g, oko, 1, Some(bear));
        finish(&mut g);
        assert_eq!(stats(&g, bear), (0, 2));

        let mut transformed = scenario(player);
        let oko = put(
            &mut transformed,
            player,
            "Oko, Lorwyn Liege",
            Zone::Battlefield,
        );
        transformed.state.objects[oko as usize].counters.loyalty = 3;
        pay_for_transform(&mut transformed);
        assert_eq!(
            transformed.object_view(oko).card.name(),
            "Oko, Shadowmoor Scion"
        );
        let mut mill = transformed.clone();
        let land = put(&mut mill, player, "Forest", Zone::Library);
        activate_oko(&mut mill, oko, 0, None);
        mill.resolve_top();
        let object = mill.object_ref(land);
        checked(
            &mut mill,
            Action::RecoverMilled {
                object: Some(object),
            },
        );
        finish(&mut mill);
        assert_eq!(mill.object_view(land).zone, Zone::Hand);

        let mut elk = transformed.clone();
        activate_oko(&mut elk, oko, 1, None);
        finish(&mut elk);
        let elk_card = Card::from_name("@token/ecl-oko-elk").unwrap();
        assert_eq!(
            elk.state
                .objects
                .iter()
                .filter(|o| o.card == elk_card && o.zone == Zone::Battlefield)
                .count(),
            2
        );

        let mut ultimate = transformed.clone();
        ultimate.state.objects[oko as usize].counters.loyalty = 6;
        let bear = put(&mut ultimate, player, "Bear Cub", Zone::Battlefield);
        activate_oko(&mut ultimate, oko, 2, None);
        ultimate.resolve_top();
        checked(
            &mut ultimate,
            Action::ChooseCreatureType {
                creature_type: crate::cards::CreatureType::from_name("Bear").unwrap(),
            },
        );
        finish(&mut ultimate);
        assert_eq!(stats(&ultimate, bear), (5, 5));
        assert!(ultimate.keywords(bear).contains(Keywords::VIGILANCE));
        assert!(ultimate.keywords(bear).contains(Keywords::HEXPROOF));
        pay_for_transform(&mut transformed);
        assert_eq!(
            transformed.object_view(oko).card.name(),
            "Oko, Lorwyn Liege"
        );
    }
}
