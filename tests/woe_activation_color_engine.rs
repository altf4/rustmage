//! Live WUBRG substitution for creature activations and exact projected payments.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::mana_lots::{ManaRestrictionView, Quality};
use rustmage_core::engine::*;
use support::*;

const ZERO: Mana = [0; 6];
const BLUE: Mana = [0, 1, 0, 0, 0, 0];
const RED: Mana = [0, 0, 0, 1, 0, 0];
const C: Mana = [0, 0, 0, 0, 0, 1];
const RED_C: Mana = [0, 0, 0, 1, 0, 1];
const THREE_C: Mana = [0, 0, 0, 0, 0, 3];

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

const PROVIDER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::SpendAnyColorForCreatureActivations],
    }],
    activated: &[ability(
        ActivationCost {
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        &[Instruction::AddMana(ManaProduction::Fixed(RED))],
    )],
    ..CardRules::EMPTY
};
const THREE: ActivatedAbility = ability(
    ActivationCost {
        colored: [1, 1, 0, 0, 0, 1],
        mana_component: true,
        ..ActivationCost::ZERO
    },
    &[Instruction::GainLife(PlayerSet::Controller, 1)],
);
const CREATURE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[
        THREE,
        ActivatedAbility {
            origin: ActivationOrigin::Graveyard,
            ..THREE
        },
        ability(
            ActivationCost {
                colored: [1, 0, 0, 0, 0, 1],
                mana_component: true,
                tap: true,
                ..ActivationCost::ZERO
            },
            &[Instruction::AddMana(ManaProduction::Choice {
                mask: 2 | 8,
                amount: 1,
            })],
        ),
    ],
    ..CardRules::EMPTY
};
const NONCREATURE: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[THREE],
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

#[test]
fn flexibility_uses_the_live_creature_and_preserves_colorless_and_mana_lot_restrictions() {
    let _rules = fixture_rules::activate(&[
        (card("Short Bow"), &PROVIDER),
        (card("Grizzly Bears"), &CREATURE),
        (card("Swiftfoot Boots"), &NONCREATURE),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let provider = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let grave = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let noncreature = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    game.state.players[player.index()].mana = THREE_C;
    reject(&mut game, activate(noncreature, 0));
    reject(&mut game, activate(grave, 1));

    let mut foreign = game.clone();
    let object = foreign.object_ref(provider);
    foreign.change_controller(object, player.other());
    reject(&mut foreign, activate(creature, 0));
    let mut no_colorless = game.clone();
    no_colorless.state.players[player.index()].mana = [0, 0, 0, 3, 0, 0];
    reject(&mut no_colorless, activate(creature, 0));

    let mut restricted = game.clone();
    restricted.state.players[player.index()].mana = ZERO;
    restricted.add_quality_mana(
        player,
        THREE_C,
        Some(Quality {
            restriction: ManaRestrictionView::ArtifactSpell,
            ..Quality::ORDINARY
        }),
    );
    reject(&mut restricted, activate(creature, 0));

    let direct = Action::Activate {
        card: creature,
        ability: 0,
        target: None,
        payment: THREE_C,
        counter_payment: Counters::default(),
    };
    let mut raw = game.clone();
    act(&mut raw, direct);
    settle(&mut raw);
    assert_eq!(raw.state.players[player.index()].life, 21);
    assert_eq!(raw.state.players[player.index()].mana, ZERO);

    act(&mut game, activate(creature, 0));
    let domain = game.mana_payment_domain().unwrap();
    assert_eq!(domain.required, C);
    assert_eq!(domain.generic, 2);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 21);
    assert_eq!(game.state.players[player.index()].mana, ZERO);
}

#[test]
fn projected_deferred_mana_needs_the_permission_in_the_payment_world() {
    let _rules = fixture_rules::activate(&[
        (card("Short Bow"), &PROVIDER),
        (card("Grizzly Bears"), &CREATURE),
    ]);
    let player = Player::P1;
    let mut parent = scenario(player);
    let provider = put(&mut parent, player, "Short Bow", Zone::Battlefield);
    let producer = put(&mut parent, player, "Grizzly Bears", Zone::Battlefield);
    let victim = put(&mut parent, player.other(), "Bear Cub", Zone::Battlefield);
    let spell = put(&mut parent, player, "Unsummon", Zone::Hand);
    parent.state.players[player.index()].mana = C;
    let cast = Action::AnnounceCast { card: spell };

    // The only way to get a second mana would remove the spending permission.
    reject(&mut parent, cast.clone());
    let mountain = put(&mut parent, player, "Mountain", Zone::Battlefield);
    act(&mut parent, cast);
    act(
        &mut parent,
        Action::ChooseTarget {
            target: Target::Card(victim),
        },
    );
    let sacrifice = Action::ActivateMana {
        card: provider,
        ability: 0,
        payment: ZERO,
        counter_payment: Counters::default(),
        production: RED,
    };
    reject(&mut parent, sacrifice.clone());
    let mut deferred = parent.clone();
    choose(
        &mut deferred,
        |action| matches!(action, Action::AnnounceMana { card, ability: 2, .. } if *card == producer),
    );
    reject(&mut deferred, sacrifice);
    act(&mut deferred, Action::TapMana { card: mountain });
    assert_eq!(deferred.mana_payment_domain().unwrap().required, C);
    assert_eq!(deferred.mana_payment_domain().unwrap().generic, 1);
    pay(&mut deferred);
    if matches!(deferred.state.pending, Pending::EffectMana { .. }) {
        act(&mut deferred, Action::ChooseMana { production: BLUE });
    }
    pay(&mut deferred);
    settle(&mut deferred);
    assert_eq!(deferred.object_view(victim).zone, Zone::Hand);
    assert_eq!(deferred.object_view(provider).zone, Zone::Battlefield);

    act(&mut parent, Action::TapMana { card: mountain });
    let direct = Action::ActivateMana {
        card: producer,
        ability: 2,
        payment: RED_C,
        counter_payment: Counters::default(),
        production: ZERO,
    };
    let saved = parent.snapshot();
    let index = parent
        .legal_actions()
        .iter()
        .position(|action| *action == direct)
        .unwrap();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    act(&mut raw, direct.clone());
    indexed
        .apply_index(player, parent.revision(), index)
        .unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    raw.restore(&saved).unwrap();
    act(&mut raw, direct);
    for game in [&mut raw, &mut indexed] {
        act(game, Action::ChooseMana { production: BLUE });
        pay(game);
        settle(game);
        assert_eq!(game.object_view(victim).zone, Zone::Hand);
        assert_eq!(game.state.players[player.index()].mana, ZERO);
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
}
