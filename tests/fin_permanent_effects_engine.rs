mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const TAP_OPPOSING_LANDS: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::TapPermanentGroup(&[BattlefieldFilter {
            kinds: 2,
            relation: 1,
            nonland: false,
            tapped: None,
            without: Keywords::NONE,
            subtype: None,
            max_mana: u32::MAX,
        }])],
    ),
    ..CardRules::EMPTY
};

const DOUBLE_ON_LANDFALL: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::LandEntersYou,
        targets: TargetPlan::NONE,
        program: &[Instruction::DoubleSourcePower],
    }],
    ..CardRules::EMPTY
};

#[test]
fn tap_group_can_tap_opposing_noncreature_lands() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &TAP_OPPOSING_LANDS)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let own_land = put(&mut game, player, "Island", Zone::Battlefield);
    let their_lands = [
        put(&mut game, player.other(), "Forest", Zone::Battlefield),
        put(&mut game, player.other(), "Swamp", Zone::Battlefield),
    ];
    let their_creature = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, None);
    settle(&mut game);
    assert!(
        their_lands
            .into_iter()
            .all(|id| game.object_view(id).tapped)
    );
    assert!(!game.object_view(own_land).tapped);
    assert!(!game.object_view(their_creature).tapped);
}

#[test]
fn doubling_source_power_uses_signed_resolving_snapshot() {
    let _rules =
        fixture_rules::activate(&[(Card::from_name("Bear Cub").unwrap(), &DOUBLE_ON_LANDFALL)]);
    let player = Player::P0;
    for power in [-2, 3] {
        let mut game = scenario(player);
        let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let land = put(&mut game, player, "Forest", Zone::Hand);
        game.state.objects[creature as usize].growth.power = power - 2;
        act(&mut game, Action::PlayLand { card: land });
        settle(&mut game);
        assert_eq!(stats(&game, creature), (power * 2, 2));
        let pump = put(&mut game, player, "Giant Growth", Zone::Hand);
        cast(&mut game, pump, Some(Target::Card(creature)));
        settle(&mut game);
        assert_eq!(stats(&game, creature), (power * 2 + 3, 5));
    }
}
