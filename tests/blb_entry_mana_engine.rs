//! Live entrant qualifiers and creature-only restricted mana.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const ENTRY_PROVIDER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[
            StaticAbilityFragment::OtherControlledCreatureEntryCountersByStat {
                kind: CounterKind::PlusOne,
                amount: StatValue::OpponentLostLifeThisTurn,
            },
        ],
    }],
    ..CardRules::EMPTY
};

const RESTRICTED_MANA: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Tagged {
            production: &ManaProduction::TapSourcePowerChoice { mask: 31 },
            quality: &ManaQuality {
                restriction: ManaRestriction::CreatureSpellManaValueAtLeastOrX(4),
                spell_bonus: ManaSpellBonus::None,
                retention: ManaRetention::Step,
            },
        })],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

#[test]
fn only_existing_entry_providers_apply_and_net_life_does_not_hide_a_loss() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &ENTRY_PROVIDER)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let source = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
        let other = put(&mut game, player, "Savannah Lions", Zone::Graveyard);
        let raise = put(&mut game, player, "Raise the Past", Zone::Hand);
        game.lose_life(player.other(), 2);
        game.gain_life(player.other(), 3);
        cast(&mut game, raise, None);
        settle(&mut game);
        for id in [source, other] {
            assert_eq!(game.object_view(id).zone, Zone::Battlefield);
            assert_eq!(game.counter_count(id, CounterKind::PlusOne), 0);
        }
        let entrant = put(&mut game, player, "Hill Giant", Zone::Hand);
        cast(&mut game, entrant, None);
        settle(&mut game);
        assert_eq!(game.counter_count(entrant, CounterKind::PlusOne), 1);
    }
}

#[test]
fn mana_pays_for_big_creatures_and_creature_x_even_when_chosen_x_is_zero() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &RESTRICTED_MANA)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let big = put(&mut game, player, "Gigantosaurus", Zone::Hand);
    let small = put(&mut game, player, "Llanowar Elves", Zone::Hand);
    let noncreature = put(&mut game, player, "Overrun", Zone::Hand);
    let x_noncreature = put(&mut game, player, "Genesis Wave", Zone::Hand);
    let x_creature = put(&mut game, player, "Goldvein Hydra", Zone::Hand);
    game.state.players[player.index()].mana = [0; 6];
    act(
        &mut game,
        Action::ActivateMana {
            card: source,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 6, 0],
        },
    );
    for card in [small, noncreature, x_noncreature] {
        reject(&mut game, Action::AnnounceCast { card });
    }
    let mut high = game.clone();
    cast(&mut high, big, None);
    settle(&mut high);
    assert_eq!(high.object_view(big).zone, Zone::Battlefield);
    assert_eq!(high.state.players[player.index()].mana[4], 1);

    act(&mut game, Action::AnnounceCast { card: x_creature });
    act(&mut game, Action::ChooseX { value: 0 });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(x_creature).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].mana[4], 5);
}
