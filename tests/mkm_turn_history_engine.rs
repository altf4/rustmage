//! Exact turn histories used by guards, cost reduction, and resolving effects.
mod support;

use rustmage as _;
use rustmage_core::{
    cards::{
        authored::{CardExtras, CardRules},
        *,
    },
    engine::{
        ability_world::CharacteristicWorld,
        funding::{FundingGoal, FundingState},
        *,
    },
};
use support::*;

const SACRIFICE_DISCOUNT: SpellReduction = SpellReduction {
    source_only: true,
    filter: CastFilter::Any,
    amount: SpellReductionAmount::Fixed(2),
    turn: CostTurn::Any,
    guard: StepGuard::Expression {
        value: StatValue::ArtifactSacrificedThisTurn,
        minimum: Some(1),
        maximum: None,
    },
};
const REDUCED_SPELL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::GainLife(PlayerSet::Controller, 1)],
    ),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CostReduction(0, SACRIFICE_DISCOUNT)],
    }],
    extras: &CardExtras {
        cost_reductions: &[SACRIFICE_DISCOUNT],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const ACTIVATION: ActivatedAbility = ActivatedAbility {
    origin: ActivationOrigin::Battlefield,
    cost: ActivationCost::ZERO,
    guard: ActivationGuard::Always,
    targets: TargetPlan::NONE,
    program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    exhaust: false,
    cycling: false,
    sorcery: false,
    once_per_turn: false,
};
const SACRIFICE_GUARD: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        guard: ActivationGuard::StatAtLeast {
            value: StatValue::ArtifactSacrificedThisTurn,
            minimum: 1,
        },
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        ..ACTIVATION
    }],
    ..CardRules::EMPTY
};
const MILL_BOTH: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Mill(PlayerSet::Each, MillAmount::Fixed(1))],
    ),
    ..CardRules::EMPTY
};
const EXILE_CREATURE_CARD: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 1,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[Instruction::Exile],
    ),
    ..CardRules::EMPTY
};
const DEAL_DAMAGE: ActivatedAbility = ActivatedAbility {
    program: &[Instruction::DamagePlayersAndTheirCreatures {
        players: PlayerSet::Opponent,
        amount: AmountValue::Fixed(1),
    }],
    ..ACTIVATION
};
const DAMAGE_SOURCE: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    activated: &[
        DEAL_DAMAGE,
        ActivatedAbility {
            cost: ActivationCost {
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            ..DEAL_DAMAGE
        },
    ],
    ..CardRules::EMPTY
};
const COPY_OWN_SPELL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::YourSpell)),
        &[Instruction::CopyTargetSpell {
            group: 0,
            retarget: false,
            exceptions: CopyExceptions::NONE,
        }],
    ),
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn stat(game: &Game, value: StatValue, source: CardId, owner: Player) -> i128 {
    game.stat_value(value, game.object_ref(source), owner)
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn activate(game: &mut Game, id: CardId, slot: u16) {
    choose(game, |action| {
        matches!(
            action,
            Action::AnnounceActivate { card, ability, .. }
                | Action::Activate { card, ability, .. }
                | Action::BeginActivate { card, ability, .. }
            if *card == id && *ability == slot
        )
    });
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

#[test]
fn artifact_sacrifices_qualify_the_payer_even_when_the_destination_is_exile() {
    let reducer = card("Seize the Secrets");
    let guard = card("Soul-Guide Lantern");
    let _rules = fixture_rules::activate(&[(reducer, &REDUCED_SPELL), (guard, &SACRIFICE_GUARD)]);
    let blue = [0, 1, 0, 0, 0, 0];
    for (payer, replaced) in [(Player::P0, false), (Player::P1, true)] {
        let owner = payer.other();
        let mut root = scenario(payer);
        let spell = spawn(&mut root, payer, reducer, Zone::Hand);
        let observer = spawn(&mut root, payer, guard, Zone::Battlefield);
        root.state.players[payer.index()].mana = blue;
        if replaced {
            put(&mut root, owner, "Rest in Peace", Zone::Battlefield);
        }
        let treasure = Card::from_catalogue_id(5302);
        root.create_tokens(owner, treasure, 1, false);
        let token = root
            .controlled_token_ids(owner)
            .find(|&id| root.object_view(id).card == treasure)
            .unwrap();
        let object = root.object_ref(token);
        root.record_control(object, payer, None);

        let offers_spell = |game: &Game| {
            game.legal_actions().iter().any(|action| {
                matches!(
                    action,
                    Action::Cast { card, .. } | Action::AnnounceCast { card }
                    if *card == spell
                )
            })
        };
        let offers_guard = |game: &Game| {
            game.legal_actions().iter().any(|action| {
                matches!(
                    action,
                    Action::AnnounceActivate { card, .. }
                        | Action::Activate { card, .. }
                        | Action::BeginActivate { card, .. }
                    if *card == observer
                )
            })
        };
        assert!(
            !offers_spell(&root),
            "sacrifice cannot change locked spell costs"
        );
        assert!(!offers_guard(&root));
        let saved = root.snapshot();
        let goal = FundingGoal {
            actor: payer,
            required: blue,
            generic: 0,
            reserve: None,
            source: None,
        };
        let world = FundingState::from_game(&root, &goal);
        assert!(
            world.cost_departures(&[object], Zone::Graveyard, &mut |projected| {
                let history = projected.zone_change_turn_history();
                assert_eq!(history.artifact_permanent_arrivals, u64::from(!replaced));
                assert!(history.artifact_sacrifices[payer.index()]);
                assert!(!history.artifact_sacrifices[owner.index()]);
                assert_eq!(
                    projected.value(StatValue::ArtifactSacrificedThisTurn, observer),
                    1,
                );
                true
            })
        );
        assert_eq!(root.snapshot(), saved);

        let sacrifice = Action::ActivateMana {
            card: token,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: blue,
        };
        let index = root
            .legal_actions()
            .iter()
            .position(|action| *action == sacrifice)
            .unwrap();
        let revision = root.revision();
        let mut raw = root.clone();
        let mut indexed = root.clone();
        drop(root);
        act(&mut raw, sacrifice);
        indexed.apply_index(payer, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(
            stat(&raw, StatValue::ArtifactSacrificedThisTurn, observer, payer),
            1
        );
        assert_eq!(
            stat(&raw, StatValue::ArtifactSacrificedThisTurn, observer, owner),
            0
        );
        assert_eq!(
            stat(
                &raw,
                StatValue::ArtifactsPutIntoGraveyardsFromBattlefieldThisTurn,
                observer,
                payer
            ),
            i128::from(!replaced)
        );
        assert!(offers_spell(&raw));
        assert!(offers_guard(&raw));
        indexed.restore(&saved).unwrap();
        assert_eq!(indexed.snapshot(), saved);
        assert!(!offers_spell(&indexed));
        assert!(!offers_guard(&indexed));

        cast(&mut raw, spell, None);
        settle(&mut raw);
        activate(&mut raw, observer, 0);
        settle(&mut raw);
        assert_eq!(raw.state.players[payer.index()].life, 22);
        raw.begin_turn();
        assert_eq!(
            stat(&raw, StatValue::ArtifactSacrificedThisTurn, observer, payer),
            0
        );
        assert_eq!(
            stat(
                &raw,
                StatValue::ArtifactsPutIntoGraveyardsFromBattlefieldThisTurn,
                observer,
                payer
            ),
            0
        );
    }
}

#[test]
fn creature_card_arrivals_follow_the_destination_and_remain_after_departure() {
    let _rules = fixture_rules::activate(&[
        (card("Think Twice"), &MILL_BOTH),
        (card("Opt"), &EXILE_CREATURE_CARD),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    put(&mut game, player, "Bear Cub", Zone::Graveyard);
    let own = put(&mut game, player, "Grizzly Bears", Zone::Library);
    let opposing = put(&mut game, opponent, "Hill Giant", Zone::Library);
    let mill = put(&mut game, player, "Think Twice", Zone::Hand);
    let exile = put(&mut game, player, "Opt", Zone::Hand);
    let vehicle = put(
        &mut game,
        player,
        "Unidentified Hovership",
        Zone::Battlefield,
    );
    let pilot = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let murder = put(&mut game, player, "Murder", Zone::Hand);

    cast(&mut game, mill, None);
    settle(&mut game);
    assert_eq!(game.object_view(own).zone, Zone::Graveyard);
    assert_eq!(game.object_view(opposing).zone, Zone::Graveyard);
    assert_eq!(
        stat(
            &game,
            StatValue::OwnCreatureCardsPutIntoGraveyardThisTurn,
            mill,
            player
        ),
        1
    );
    assert_eq!(
        stat(
            &game,
            StatValue::CreatureCardsPutIntoGraveyardsThisTurn,
            mill,
            player
        ),
        2
    );
    cast(&mut game, exile, Some(Target::Card(opposing)));
    settle(&mut game);
    assert_eq!(game.object_view(opposing).zone, Zone::Exile);
    assert_eq!(
        stat(
            &game,
            StatValue::CreatureCardsPutIntoGraveyardsThisTurn,
            mill,
            player
        ),
        2
    );

    activate(&mut game, vehicle, 0);
    let object = game.object_ref(pilot);
    act(&mut game, Action::ChooseCrewer { object });
    act(&mut game, Action::FinishCrew);
    settle(&mut game);
    assert!(game.is_creature(vehicle));
    cast(&mut game, murder, Some(Target::Card(vehicle)));
    settle(&mut game);
    assert_eq!(game.object_view(vehicle).zone, Zone::Graveyard);
    assert!(!game.is_creature(vehicle));
    assert_eq!(
        stat(
            &game,
            StatValue::CreatureCardsPutIntoGraveyardsThisTurn,
            mill,
            player
        ),
        2
    );
    assert_eq!(
        stat(
            &game,
            StatValue::ArtifactsPutIntoGraveyardsFromBattlefieldThisTurn,
            mill,
            player
        ),
        1
    );
    assert_eq!(
        stat(&game, StatValue::ArtifactSacrificedThisTurn, mill, player),
        0
    );

    game.begin_turn();
    assert_eq!(
        stat(
            &game,
            StatValue::CreatureCardsPutIntoGraveyardsThisTurn,
            mill,
            player
        ),
        0
    );
}

#[test]
fn meld_graveyard_history_counts_each_physical_front_after_one_permanent_dies() {
    let player = Player::P0;
    let mut game = scenario(player);
    let vanille = put(
        &mut game,
        player,
        "Vanille, Cheerful l'Cie",
        Zone::Battlefield,
    );
    let fang = put(&mut game, player, "Fang, Fearless l'Cie", Zone::Battlefield);
    game.begin_step(Step::Main1);
    game.prepare_triggers();
    pass_pair(&mut game);
    choose(&mut game, |action| {
        matches!(action, Action::PayResolutionMana { .. })
    });
    let object = game.object_ref(fang);
    act(&mut game, Action::ChooseEffectObject { object });
    settle(&mut game);
    let melded = game.observe(player).melded_permanents[0].object;
    assert_eq!(melded.card, vanille);
    let goal = FundingGoal {
        actor: player,
        required: [0; 6],
        generic: 0,
        reserve: None,
        source: None,
    };
    let world = FundingState::from_game(&game, &goal);
    assert!(
        world.cost_departures(&[melded], Zone::Graveyard, &mut |next| {
            assert_eq!(
                next.zone_change_turn_history().creature_card_arrivals[player.index()],
                2
            );
            true
        })
    );
    let murder = put(&mut game, player, "Murder", Zone::Hand);
    cast(&mut game, murder, Some(Target::Card(melded.card)));
    pass_pair(&mut game);
    assert_eq!(game.object_view(vanille).zone, Zone::Graveyard);
    assert_eq!(game.object_view(fang).zone, Zone::Graveyard);
    assert_eq!(
        stat(
            &game,
            StatValue::OwnCreatureCardsPutIntoGraveyardThisTurn,
            murder,
            player
        ),
        2
    );
}

#[test]
fn damage_history_deduplicates_sources_and_uses_event_control_and_departure_lki() {
    let _rules = fixture_rules::activate(&[(card("Hill Giant"), &DAMAGE_SOURCE)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let pinger = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let victim = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    game.add_counter(victim, CounterKind::Shield, 1);
    cast(&mut game, shock, Some(Target::Card(victim)));
    pass_pair(&mut game);
    if game
        .legal_actions()
        .iter()
        .any(|action| matches!(action, Action::ChooseReplacement { .. }))
    {
        choose(&mut game, |action| {
            matches!(action, Action::ChooseReplacement { .. })
        });
    }
    settle(&mut game);
    assert_eq!(
        stat(
            &game,
            StatValue::DistinctControlledDamageSourcesThisTurn,
            pinger,
            player
        ),
        0
    );

    for _ in 0..2 {
        activate(&mut game, pinger, 0);
        settle(&mut game);
    }
    assert_eq!(game.object_view(victim).damage, 2);
    assert_eq!(
        stat(
            &game,
            StatValue::DistinctControlledDamageSourcesThisTurn,
            pinger,
            player
        ),
        1
    );

    game.record_control(game.object_ref(pinger), opponent, None);
    act(&mut game, Action::Pass);
    activate(&mut game, pinger, 0);
    settle(&mut game);
    act(&mut game, Action::Pass);
    activate(&mut game, pinger, 1);
    settle(&mut game);
    assert_eq!(game.object_view(pinger).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].life, 18);
    for controller in [player, opponent] {
        assert_eq!(
            stat(
                &game,
                StatValue::DistinctControlledDamageSourcesThisTurn,
                pinger,
                controller
            ),
            1
        );
    }
    game.begin_turn();
    assert_eq!(
        stat(
            &game,
            StatValue::DistinctControlledDamageSourcesThisTurn,
            pinger,
            player
        ),
        0
    );
}

#[test]
fn a_damaging_spell_and_its_copy_are_distinct_sources() {
    let _rules = fixture_rules::activate(&[(card("Opt"), &COPY_OWN_SPELL)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    let copy = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(player.other())));
    cast(&mut game, copy, Some(Target::Spell(shock)));
    settle(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 16);
    assert_eq!(
        stat(
            &game,
            StatValue::DistinctControlledDamageSourcesThisTurn,
            shock,
            player
        ),
        2
    );
}

fn declare(game: &mut Game, player: Player, cards: &[CardId]) {
    game.state.active = player;
    game.state.priority = player;
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    game.state.attack_declared = false;
    game.state.attackers.clear();
    game.state.selected_attackers.clear();
    for &card in cards {
        game.state.objects[card as usize].tapped = false;
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
    settle(game);
}

#[test]
fn distinct_attacker_history_counts_actual_declarations_and_retains_departures() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let mobilizer = put(&mut game, player, "Voice of Victory", Zone::Battlefield);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    declare(&mut game, player, &[mobilizer]);
    assert_eq!(game.controlled_token_ids(player).count(), 2);
    assert_eq!(
        stat(
            &game,
            StatValue::DistinctDeclaredAttackersThisTurn,
            mobilizer,
            player
        ),
        1
    );

    declare(&mut game, player, &[mobilizer, first]);
    assert_eq!(
        stat(
            &game,
            StatValue::DistinctDeclaredAttackersThisTurn,
            mobilizer,
            player
        ),
        2
    );
    declare(&mut game, opponent, &[second]);
    assert_eq!(
        stat(
            &game,
            StatValue::DistinctDeclaredAttackersThisTurn,
            mobilizer,
            player
        ),
        3
    );
    let departed = game.object_ref(first);
    let shock = put(&mut game, opponent, "Shock", Zone::Hand);
    game.state.players[opponent.index()].mana = [8; 6];
    cast(&mut game, shock, Some(Target::Card(first)));
    settle(&mut game);
    assert_ne!(game.object_ref(first), departed);
    assert_eq!(
        stat(
            &game,
            StatValue::DistinctDeclaredAttackersThisTurn,
            mobilizer,
            player
        ),
        3
    );
}
