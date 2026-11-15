//! Shared typed rules for the Star Trek Captain and Combadge cards.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const OFFICERS: StatValue = StatValue::Count(CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 0,
    exclude: 0,
    subtype: Some("Officer"),
    basic: false,
    other: false,
    cards_only: false,
});
const EXILE_CAST: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ExileTopLibraryRemember {
                players: PlayerSet::Controller,
                amount: AmountValue::Fixed(3),
            },
            Instruction::CastRememberedExileFreeUpTo(OFFICERS),
        ],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        sorcery: false,
        once_per_turn: false,
        cycling: false,
        exhaust: false,
    }],
    ..CardRules::EMPTY
};
const SIMPLE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::GainLife(PlayerSet::Controller, 1)],
    ),
    ..CardRules::EMPTY
};
const SEARCH: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SearchWithEntryCounters {
            spec: SearchSpec {
                filter: SearchFilter::BasicLand,
                maximum: 1,
                destination: SearchDestination::Battlefield,
                tapped: false,
                reveal: true,
            },
            amounts: &[(CounterKind::Charge, 7)],
        }],
    ),
    ..CardRules::EMPTY
};
const DOUBLE: ReplacementSpec = ReplacementSpec::DoubleCountersPlacedByController {
    condition: StatValue::Fixed(1),
};
const DOUBLER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, DOUBLE)],
    }],
    extras: &CardExtras {
        replacement_effects: &[DOUBLE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const COMMUNICATOR: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            guard: StepGuard::OtherControlledCreatureWithAttachedSubtype("Communicator"),
            event: &TriggerCondition::GrantedAttacks,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GrantKeywords {
            subject: Subject::Source,
            keywords: Keywords::CANT_BE_BLOCKED,
        }],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::AttachmentTrigger(0)],
    }],
    extras: &CardExtras {
        equipment: Some(EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn advance(game: &mut Game, wanted: impl Fn(&Game) -> bool) {
    for _ in 0..48 {
        if wanted(game) {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::ChooseReplacement { .. }
                    | Action::ChooseEntryTimestamp { .. }
            )
        });
    }
    panic!("Captain engine action budget exceeded");
}

#[test]
fn exact_exiled_cards_offer_multiple_free_casts_under_a_live_officer_ceiling() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Short Bow").unwrap(), &EXILE_CAST),
        (Card::from_name("Opt").unwrap(), &SIMPLE),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let officer = put(&mut game, player, "Tactical Officer", Zone::Battlefield);
    put(&mut game, player, "Tactical Officer", Zone::Battlefield);
    let source = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let unrelated = put(&mut game, player, "Opt", Zone::Exile);
    let opt = put(&mut game, player, "Opt", Zone::Library);
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Library);
    let giant = put(&mut game, player, "Hill Giant", Zone::Library);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(&mut game);
    advance(&mut game, |g| {
        matches!(g.state.pending, Pending::ResolutionCasts { .. })
    });
    assert_eq!(
        game.observe(player)
            .resolution_free_casts
            .as_ref()
            .unwrap()
            .remaining
            .len(),
        3
    );
    let first = Action::CastExiledDuringResolution {
        object: game.object_ref(opt),
        face: CastFace::Normal,
    };
    let second = Action::CastExiledDuringResolution {
        object: game.object_ref(bear),
        face: CastFace::Normal,
    };
    let unrelated_ref = game.object_ref(unrelated);
    let giant_ref = game.object_ref(giant);
    reject(
        &mut game,
        Action::CastExiledDuringResolution {
            object: unrelated_ref,
            face: CastFace::Normal,
        },
    );
    reject(
        &mut game,
        Action::CastExiledDuringResolution {
            object: giant_ref,
            face: CastFace::Normal,
        },
    );
    let checkpoint = game.snapshot();
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == first)
        .unwrap();
    let mut indexed = game.clone();
    act(&mut game, first);
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&checkpoint).unwrap();
    assert_eq!(
        indexed.apply_index(player, revision, index),
        Err(Error::StaleRevision)
    );
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    let mut fewer_officers = game.clone();
    fewer_officers.move_object(officer, Zone::Graveyard);
    reject(&mut fewer_officers, second.clone());
    assert!(game.legal_actions().contains(&second));
    act(&mut game, second);
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    act(&mut game, Action::FinishResolutionCasts);
    settle(&mut game);
    assert_eq!(game.object_view(giant).zone, Zone::Exile);
    assert_eq!(game.object_view(unrelated).zone, Zone::Exile);
    assert_eq!(game.object_view(bear).zone, Zone::Battlefield);
    assert_eq!(game.state.players[player.index()].life, 21);
}

#[test]
fn untargeted_search_enters_with_replacement_adjusted_counters_and_private_choice() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Opt").unwrap(), &SEARCH),
        (Card::from_name("Short Bow").unwrap(), &DOUBLER),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        put(&mut game, player, "Short Bow", Zone::Battlefield);
        let forest = put(&mut game, player, "Forest", Zone::Library);
        let spell = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, spell, None);
        advance(&mut game, |g| {
            matches!(g.state.pending, Pending::EffectSearch { .. })
        });
        assert_eq!(game.actor(), Some(player));
        assert!(game.observe(player.other()).search_choice.is_none());
        reject(&mut game, Action::ChooseSearch { card: spell });
        let checkpoint = game.snapshot();
        let mut declined = game.clone();
        act(&mut declined, Action::FinishSearch);
        settle(&mut declined);
        assert_eq!(declined.object_view(forest).zone, Zone::Library);
        game.restore(&checkpoint).unwrap();
        act(&mut game, Action::ChooseSearch { card: forest });
        act(&mut game, Action::FinishSearch);
        advance(&mut game, |g| {
            g.state.stack.is_empty()
                && g.state.resolving.is_none()
                && g.state.pending == Pending::Priority
        });
        assert_eq!(game.object_view(forest).zone, Zone::Battlefield);
        assert_eq!(game.counter_count(forest, CounterKind::Charge), 14);
    }
}

#[test]
fn communicator_attack_guard_uses_another_creature_and_freezes_at_declaration() {
    let badge = Card::from_name("Combadge").unwrap();
    let _rules = fixture_rules::activate(&[(badge, &COMMUNICATOR)]);
    for (recipient, qualified) in [(0, false), (1, false), (2, false), (3, true)] {
        let player = Player::P0;
        let mut game = scenario(player);
        let attacker = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let other = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let opponent = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let first = spawn(&mut game, player, badge, Zone::Battlefield);
        assert!(game.attach_equipment(game.object_ref(first), game.object_ref(attacker)));
        let second = if recipient != 0 {
            let second = spawn(&mut game, player, badge, Zone::Battlefield);
            let target = match recipient {
                1 => attacker,
                2 => opponent,
                _ => other,
            };
            assert!(game.attach_equipment(game.object_ref(second), game.object_ref(target)));
            Some(second)
        } else {
            None
        };
        game.state.step = Step::Attackers;
        game.state.pending = Pending::Attackers;
        act(&mut game, Action::Attack { card: attacker });
        act(&mut game, Action::FinishAttackers);
        if let Some(second) = second {
            game.move_object(second, Zone::Graveyard);
        }
        advance(&mut game, |g| {
            g.state.pending == Pending::Priority
                && g.state.pending_triggers.is_empty()
                && g.state.stack.is_empty()
        });
        assert_eq!(
            game.keywords(attacker).contains(Keywords::CANT_BE_BLOCKED),
            qualified
        );
    }
}
