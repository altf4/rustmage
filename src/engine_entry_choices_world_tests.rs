use super::super::ability_timeline_tests::{cast, checked, settle, setup};
use super::super::ability_world::CharacteristicWorld;
use super::super::tests::spawn;
use super::*;
use crate::cards::{ChoiceColor, ChosenModifier, CreatureType, EntryChoiceKind, EntryChoiceValue};
#[test]
fn entry_choices_funding_exact_generation_and_sparse_counter_modifier_overlay() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let banner = spawn(
            &mut g,
            p,
            Card::from_name("Heraldic Banner").unwrap(),
            Zone::Hand,
        );
        cast(&mut g, banner, None, &[]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        checked(
            &mut g,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::Color(ChoiceColor::Red),
            },
        );
        settle(&mut g);
        let creature = spawn(
            &mut g,
            p,
            Card::from_name("Bear Cub").unwrap(),
            Zone::Battlefield,
        );
        let kinship = spawn(
            &mut g,
            p,
            Card::from_name("Banner of Kinship").unwrap(),
            Zone::Hand,
        );
        cast(&mut g, kinship, None, &[]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        checked(
            &mut g,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::CreatureType(CreatureType::from_name("Bear").unwrap()),
            },
        );
        settle(&mut g);
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 1,
            reserve: None,
            source: None,
        };
        let original = g.snapshot();
        let mut world = FundingState::from_game(&g, &goal);
        let production = ManaProduction::ChosenColor;
        assert_eq!(
            production.in_world(&world, banner),
            ManaProduction::Fixed([0, 0, 0, 1, 0, 0])
        );
        let modifier = ChosenModifier {
            kind: EntryChoiceKind::CreatureType,
            other: false,
            stats: Stats {
                power: 1,
                toughness: 1,
            },
            source_counter: Some(CounterKind::Fellowship),
        };
        assert_eq!(
            entry_choices::modifier_bonus(&world, kinship, creature, modifier).power,
            1
        );
        let kinship_ref = world.reference(kinship);
        live_counters::set(
            &mut world.named_counters,
            kinship_ref,
            Counters::of(CounterKind::Fellowship, 3).named,
        );
        assert_eq!(
            entry_choices::modifier_bonus(&world, kinship, creature, modifier).power,
            3
        );
        let source = world.resources.iter_mut().find(|r| r.id == banner).unwrap();
        source.object.generation += 1;
        assert_eq!(
            production.in_world(&world, banner),
            ManaProduction::Fixed([0; 6])
        );
        world
            .resources
            .iter_mut()
            .find(|r| r.id == kinship)
            .unwrap()
            .object
            .zone = Zone::Graveyard;
        assert_eq!(
            entry_choices::modifier_bonus(&world, kinship, creature, modifier),
            Stats::default()
        );
        assert_eq!(g.snapshot(), original);
    }
}
