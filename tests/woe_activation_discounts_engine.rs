//! Live creature and land activation discounts in actual and projected payment.
mod support;

use rustmage as _;
use rustmage_core::CardName;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::characteristic_layers::AnimationTypeMode;
use rustmage_core::engine::*;
use support::*;

const BLUE: Mana = [0, 1, 0, 0, 0, 0];
const ONE: Mana = [0, 0, 0, 0, 0, 1];
const CREATURE_DISCOUNTS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::ControlledActivationReduction {
        kinds: 1,
        amount: ActivationReductionAmount::ProviderPower,
        leave_one_mana: true,
    }],
}];
const LAND_DISCOUNTS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::ControlledActivationReduction {
        kinds: 2,
        amount: ActivationReductionAmount::Fixed(1),
        leave_one_mana: false,
    }],
}];

const fn ability(cost: ActivationCost, program: &'static [Instruction]) -> ActivatedAbility {
    ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost,
        targets: TargetPlan::NONE,
        program,
        sorcery: false,
        once_per_turn: false,
    }
}
const LIFE: &[Instruction] = &[Instruction::GainLife(PlayerSet::Controller, 1)];
const SAC_BLUE: ActivatedAbility = ability(
    ActivationCost {
        sacrifice: true,
        ..ActivationCost::ZERO
    },
    &[Instruction::AddMana(ManaProduction::Fixed(BLUE))],
);
const CREATURE_REDUCER: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    static_groups: CREATURE_DISCOUNTS,
    activated: &[SAC_BLUE],
    ..CardRules::EMPTY
};
const PASSIVE_REDUCER: CardRules = CardRules {
    activated: &[],
    ..CREATURE_REDUCER
};
const LAND_REDUCER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: LAND_DISCOUNTS,
    ..CardRules::EMPTY
};
const THREE: ActivatedAbility = ability(
    ActivationCost {
        generic: 3,
        mana_component: true,
        ..ActivationCost::ZERO
    },
    LIFE,
);
const TWO_BLUE: ActivatedAbility = ability(
    ActivationCost {
        colored: BLUE,
        generic: 2,
        mana_component: true,
        ..ActivationCost::ZERO
    },
    LIFE,
);
const CREATURE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[
        THREE,
        TWO_BLUE,
        ActivatedAbility {
            origin: ActivationOrigin::Graveyard,
            ..THREE
        },
    ],
    ..CardRules::EMPTY
};
const LAND: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[ability(
        ActivationCost {
            generic: 2,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        LIFE,
    )],
    ..CardRules::EMPTY
};
const TAX: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::NamedActivationGenericIncrease(1)],
    }],
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn activate(id: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card: id,
        ability,
        counter_payment: Counters::default(),
    }
}

#[test]
fn live_power_discounts_only_controlled_battlefield_creatures_and_respects_the_floor() {
    let _rules = fixture_rules::activate(&[
        (card("Bear Cub"), &PASSIVE_REDUCER),
        (card("Grizzly Bears"), &CREATURE),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let provider = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let creature = put(&mut g, player, "Grizzly Bears", Zone::Battlefield);
        let buried = put(&mut g, player, "Grizzly Bears", Zone::Graveyard);
        let action = activate(creature, 0);
        g.state.players[player.index()].mana = ONE;
        reject(&mut g, action.clone());
        g.state.objects[provider as usize].growth.power = 2;

        let mut foreign = g.clone();
        foreign.state.objects[provider as usize].controller = player.other();
        reject(&mut foreign, action.clone());

        let mut stripped = g.clone();
        let reducer = stripped.object_ref(provider);
        let source = stripped.object_ref(creature);
        stripped.remove_abilities_exact(&[reducer], source, player, AnimationDuration::EndOfTurn);
        reject(&mut stripped, action.clone());
        reject(&mut g, activate(buried, 2));

        let mut without_mana = g.clone();
        without_mana.state.players[player.index()].mana = [0; 6];
        reject(&mut without_mana, action.clone());
        act(&mut g, action);
        assert_eq!(g.mana_payment_domain().unwrap().generic, 1);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[player.index()].life, 21);
        assert_eq!(g.state.players[player.index()].mana, [0; 6]);
        assert_eq!(g.object_view(provider).zone, Zone::Battlefield);
    }
}

#[test]
fn an_animated_land_can_combine_independent_reductions_to_zero() {
    let _rules = fixture_rules::activate(&[
        (card("Bear Cub"), &PASSIVE_REDUCER),
        (card("Hill Giant"), &LAND_REDUCER),
        (card("Crystal Grotto"), &LAND),
    ]);
    let player = Player::P0;
    let mut g = scenario(player);
    let land = put(&mut g, player, "Crystal Grotto", Zone::Battlefield);
    put(&mut g, player, "Hill Giant", Zone::Battlefield);
    g.state.players[player.index()].mana = [0; 6];
    reject(&mut g, activate(land, 0));

    let provider = put(&mut g, player, "Bear Cub", Zone::Battlefield);
    g.state.objects[provider as usize].growth.power = 3;
    let object = g.object_ref(land);
    g.change_object_form(
        object,
        AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Named(&["Elemental"]),
            base: Some((3, 3)),
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        AnimationTypeMode::AddCreature,
    );
    act(&mut g, activate(land, 0));
    assert_eq!(g.mana_payment_domain().unwrap().generic, 0);
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.state.players[player.index()].life, 21);
    assert_eq!(g.object_view(provider).zone, Zone::Battlefield);
}

#[test]
fn paying_mana_can_remove_the_reducer_after_the_discount_is_locked() {
    let _rules = fixture_rules::activate(&[
        (card("Bear Cub"), &CREATURE_REDUCER),
        (card("Grizzly Bears"), &CREATURE),
    ]);
    let player = Player::P1;
    let mut parent = scenario(player);
    let provider = put(&mut parent, player, "Bear Cub", Zone::Battlefield);
    parent.state.objects[provider as usize].growth.power = 1;
    let creature = put(&mut parent, player, "Grizzly Bears", Zone::Battlefield);
    parent.state.players[player.index()].mana = [0; 6];
    let activation = activate(creature, 1);
    act(&mut parent, activation.clone());
    let before = parent.snapshot();
    let mana = Action::ActivateMana {
        card: provider,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production: BLUE,
    };
    let index = parent
        .legal_actions()
        .iter()
        .position(|action| *action == mana)
        .unwrap();
    let revision = parent.revision();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    drop(parent);
    act(&mut raw, mana.clone());
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    for branch in [&mut raw, &mut indexed] {
        assert_eq!(branch.object_view(provider).zone, Zone::Graveyard);
        let locked = branch
            .state
            .announcing
            .as_ref()
            .unwrap()
            .total_cost
            .unwrap();
        assert_eq!(locked.colored, BLUE);
        assert_eq!(locked.generic, 0);
        pay(branch);
        settle(branch);
        assert_eq!(branch.state.players[player.index()].life, 21);
        reject(branch, activation.clone());
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
    indexed.restore(&before).unwrap();
    act(&mut indexed, mana);
    pay(&mut indexed);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}

#[test]
fn projected_cast_funding_sees_a_discounted_printed_or_intrinsic_mana_ability() {
    let _rules = fixture_rules::activate(&[
        (card("Hill Giant"), &LAND_REDUCER),
        (card("Ghost Vacuum"), &TAX),
    ]);
    for taxed_basic in [false, true] {
        let player = Player::P0;
        let mut g = scenario(player);
        let name = if taxed_basic {
            "Island"
        } else {
            "Crystal Grotto"
        };
        let land = put(&mut g, player, name, Zone::Battlefield);
        if taxed_basic {
            let tax = put(&mut g, player.other(), "Ghost Vacuum", Zone::Battlefield);
            let source = g.object_ref(tax);
            g.set_entry_value(
                source,
                EntryChoiceValue::CardName(CardName::from_name(name).unwrap()),
            );
        }
        let spell = put(&mut g, player, "Unsummon", Zone::Hand);
        let target = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        g.state.players[player.index()].mana = [0; 6];
        let announce = Action::AnnounceCast { card: spell };
        reject(&mut g, announce.clone());
        put(&mut g, player, "Hill Giant", Zone::Battlefield);
        act(&mut g, announce);
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        choose(&mut g, |action| match action {
            Action::ActivateMana {
                card,
                production,
                payment,
                ..
            } => *card == land && *production == BLUE && *payment == [0; 6],
            Action::AnnounceMana {
                card, production, ..
            } => *card == land && *production == BLUE,
            _ => false,
        });
        if g.state
            .announcing
            .as_ref()
            .is_some_and(|a| a.item.source.is_some_and(|s| s.id == land))
        {
            assert_eq!(g.mana_payment_domain().unwrap().generic, 0);
            pay(&mut g);
        }
        assert_eq!(g.state.players[player.index()].mana, BLUE);
        assert!(g.object_view(land).tapped);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Hand);
    }
}
