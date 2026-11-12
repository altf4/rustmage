//! A cast declares the size of its sacrifice cost independently of spell X.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const VIPER_COST: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::SacrificeForGenericReduction,
            filter: SacrificeFilter {
                nonland: true,
                kinds: 0,
                other: false,
                source_only: false,
                tokens: None,
                subtype: None,
                keywords: Keywords::NONE,
                artifact_enchantment_or_token: false,
            },
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const RED: Mana = [0, 0, 0, 1, 0, 0];

fn fixture() -> fixture_rules::Guard {
    fixture_rules::activate(&[(Card::from_name("Hill Giant").unwrap(), &VIPER_COST)])
}
fn select_clause(g: &mut Game, wanted: CastCostClause) {
    let row = g
        .observe(g.actor().unwrap())
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == wanted && !row.paid)
        .unwrap() as u16;
    act(g, Action::ChooseCastCost { clause: row });
}
fn choose_sacrifice(g: &Game, id: CardId) -> Action {
    Action::ChooseAdditionalCost {
        object: g.object_ref(id),
    }
}

#[test]
fn declare_more_sacrifices_than_generic_and_pay_in_either_order() {
    let _rules = fixture();
    for p in [Player::P0, Player::P1] {
        let mut root = scenario(p);
        root.state.players[p.index()].mana = RED;
        let viper = put(&mut root, p, "Hill Giant", Zone::Hand);
        let mut cost = vec![
            put(&mut root, p, "Bear Cub", Zone::Battlefield),
            put(&mut root, p, "Candy Trail", Zone::Battlefield),
        ];
        let own_land = put(&mut root, p, "Forest", Zone::Battlefield);
        root.state.objects[own_land as usize].tapped = true;
        let foe_food = put(&mut root, p.other(), "Candy Trail", Zone::Battlefield);
        let announce = Action::AnnounceCast { card: viper };
        reject(&mut root, announce.clone());
        cost.push(put(&mut root, p, "Candy Trail", Zone::Battlefield));
        cost.push(put(&mut root, p, "Short Bow", Zone::Battlefield));
        act(&mut root, announce);
        let domain = root.x_choice().unwrap();
        assert_eq!(
            domain.intervals,
            vec![XInterval {
                minimum: 3,
                maximum: 4
            }]
        );
        reject(&mut root, Action::ChooseX { value: 2 });
        reject(&mut root, Action::ChooseX { value: 5 });

        let declaration = Action::ChooseX { value: 4 };
        let before = root.snapshot();
        let revision = root.revision();
        let index = root
            .legal_actions()
            .iter()
            .position(|a| *a == declaration)
            .unwrap();
        let mut indexed = root.clone();
        act(&mut root, declaration.clone());
        indexed.apply_index(p, revision, index).unwrap();
        assert_eq!(root.snapshot(), indexed.snapshot());
        indexed.restore(&before).unwrap();
        assert_eq!(
            indexed.apply_index(p, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, declaration);
        assert_eq!(root.snapshot(), indexed.snapshot());
        assert_eq!(root.state.announcing.as_ref().unwrap().item.x, None);
        assert_eq!(
            root.state
                .announcing
                .as_ref()
                .unwrap()
                .total_cost
                .unwrap()
                .generic,
            0
        );

        for (g, mana_first) in [(&mut root, true), (&mut indexed, false)] {
            act(g, Action::BeginOrderedCosts);
            if mana_first {
                select_clause(g, CastCostClause::Mana);
                pay(g);
            }
            select_clause(g, CastCostClause::Additional);
            let wrong_land = choose_sacrifice(g, own_land);
            let wrong_controller = choose_sacrifice(g, foe_food);
            reject(g, wrong_land);
            reject(g, wrong_controller);
            for (i, &object) in cost.iter().enumerate() {
                let action = choose_sacrifice(g, object);
                act(g, action.clone());
                if i + 1 < cost.len() {
                    reject(g, action);
                    assert_eq!(g.object_view(object).zone, Zone::Battlefield);
                }
            }
            if !mana_first {
                select_clause(g, CastCostClause::Mana);
                pay(g);
            }
            let item = g.state.stack.last().unwrap();
            assert_eq!(item.x, None);
            let paid = item.paid_additional_cost.as_ref().unwrap();
            assert_eq!((paid.selected, paid.count), (4, 4));
            settle(g);
            assert_eq!(g.object_view(viper).zone, Zone::Battlefield);
            assert!(
                cost.iter()
                    .all(|&object| g.object_view(object).zone == Zone::Graveyard)
            );
            assert_eq!(g.object_view(own_land).zone, Zone::Battlefield);
            assert_eq!(g.object_view(foe_food).zone, Zone::Battlefield);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}

#[test]
fn zero_sacrifices_and_tapping_a_later_sacrifice_are_legal_routes() {
    let _rules = fixture();
    for p in [Player::P0, Player::P1] {
        let mut full = scenario(p);
        let viper = put(&mut full, p, "Hill Giant", Zone::Hand);
        let kept = put(&mut full, p, "Candy Trail", Zone::Battlefield);
        act(&mut full, Action::AnnounceCast { card: viper });
        act(&mut full, Action::ChooseX { value: 0 });
        pay(&mut full);
        assert!(
            full.state
                .stack
                .last()
                .unwrap()
                .paid_additional_cost
                .is_none()
        );
        settle(&mut full);
        assert_eq!(full.object_view(kept).zone, Zone::Battlefield);

        let mut discounted = scenario(p);
        discounted.state.players[p.index()].mana = RED;
        let viper = put(&mut discounted, p, "Hill Giant", Zone::Hand);
        let elf = put(&mut discounted, p, "Llanowar Elves", Zone::Battlefield);
        let food = put(&mut discounted, p, "Candy Trail", Zone::Battlefield);
        act(&mut discounted, Action::AnnounceCast { card: viper });
        assert_eq!(
            discounted.x_choice().unwrap().intervals,
            vec![XInterval {
                minimum: 2,
                maximum: 2
            }]
        );
        act(&mut discounted, Action::ChooseX { value: 2 });
        choose(
            &mut discounted,
            |action| matches!(action, Action::ActivateMana { card, .. } | Action::TapMana { card } if *card == elf),
        );
        assert!(discounted.object_view(elf).tapped);
        pay(&mut discounted);
        select_clause(&mut discounted, CastCostClause::Additional);
        for object in [elf, food] {
            let action = choose_sacrifice(&discounted, object);
            act(&mut discounted, action);
        }
        settle(&mut discounted);
        assert_eq!(discounted.object_view(viper).zone, Zone::Battlefield);
        assert_eq!(discounted.object_view(elf).zone, Zone::Graveyard);
        assert_eq!(discounted.object_view(food).zone, Zone::Graveyard);
        assert_eq!(discounted.state.players[p.index()].mana, [0; 6]);
    }
}
