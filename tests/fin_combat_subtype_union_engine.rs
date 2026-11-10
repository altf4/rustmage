mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const CONTROLLED: BattlefieldFilter = BattlefieldFilter {
    kinds: 0,
    relation: 0,
    nonland: false,
    tapped: None,
    without: Keywords::NONE,
    subtype: None,
    max_mana: u32::MAX,
};
const SOURCE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::PlayerDamaged {
                subject: DamageSubject::ControlledSubtypes(&["Scout", "Pirate", "Rogue"]),
                combat_only: true,
                opponent_only: false,
                one_or_more: true,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::ExileTopMayCastPaidFrom(PlayerSet::DamagedPlayer),
                Instruction::IfOfferedCastCommitted { skip: 1 },
                Instruction::CreateTokens {
                    token: Card(5302),
                    amount: 1,
                    tapped: false,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::NotOwnedByYou,
                ordinal: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::PutPermanentGroupCounters {
                filters: &[
                    BattlefieldFilter {
                        subtype: Some("Scout"),
                        ..CONTROLLED
                    },
                    BattlefieldFilter {
                        subtype: Some("Pirate"),
                        ..CONTROLLED
                    },
                    BattlefieldFilter {
                        subtype: Some("Rogue"),
                        ..CONTROLLED
                    },
                ],
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        },
    ],
    ..CardRules::EMPTY
};

fn passes(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn treasures(game: &Game, player: Player) -> usize {
    game.observe(player)
        .objects
        .iter()
        .filter(|object| {
            object.zone == Zone::Battlefield
                && object.controller == player
                && object.card.has_subtype("Treasure")
        })
        .count()
}

#[test]
fn actual_damage_aggregates_one_subtype_union_per_combat_step_and_paid_cast_grows_each_once() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Grizzly Bears").unwrap(), &SOURCE)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let island = put(&mut game, player, "Island", Zone::Battlefield);
    let scout = put(&mut game, player, "Ainok Wayfarer", Zone::Battlefield);
    let pirate = put(&mut game, player, "Bigfin Bouncer", Zone::Battlefield);
    let changeling = put(&mut game, player, "Barkform Harvester", Zone::Battlefield);
    let kindred = put(&mut game, player, "Firdoch Core", Zone::Battlefield);
    let unrelated = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let enemy = put(&mut game, opponent, "Charming Scoundrel", Zone::Battlefield);
    let land = put(&mut game, opponent, "Mountain", Zone::Library);
    let offered = put(&mut game, opponent, "Opt", Zone::Library);
    game.state.objects[changeling as usize].gained_keywords = Keywords::DOUBLE_STRIKE;
    for _ in 0..8 {
        if game.state.pending == Pending::Attackers {
            break;
        }
        act(&mut game, Action::Pass);
    }
    for card in [scout, pirate, changeling, unrelated] {
        act(&mut game, Action::Attack { card });
    }
    act(&mut game, Action::FinishAttackers);
    passes(&mut game);
    act(&mut game, Action::FinishBlockers);
    passes(&mut game);
    assert_eq!(game.state.step, Step::FirstStrikeDamage);
    assert_eq!(game.state.stack.len(), 1);
    let hit = game.state.stack[0].damage_event.unwrap();
    assert_eq!((hit.player, hit.amount, hit.combat), (opponent, 2, true));
    assert!(game.state.stack[0].targets.is_empty());
    passes(&mut game);
    assert_eq!(game.object_view(offered).zone, Zone::Exile);
    let object = game.object_ref(offered);
    act(
        &mut game,
        Action::CastExiledDuringResolution {
            object,
            face: CastFace::Normal,
        },
    );
    act(&mut game, Action::TapMana { card: island });
    pay(&mut game);
    act(&mut game, Action::FinishResolutionCasts);
    assert_eq!(game.object_view(offered).owner, opponent);
    assert_eq!(game.object_view(offered).controller, player);
    assert_eq!(game.state.stack.len(), 2);
    assert_eq!(treasures(&game, player), 0);
    passes(&mut game);
    for id in [scout, pirate, changeling, kindred] {
        assert_eq!(game.counter_count(id, CounterKind::PlusOne), 1);
    }
    for id in [unrelated, enemy] {
        assert_eq!(game.counter_count(id, CounterKind::PlusOne), 0);
    }
    passes(&mut game);
    choose(&mut game, |action| {
        matches!(
            action,
            Action::ArrangeLibrary {
                destination: LibraryDestination::Top,
                ..
            }
        )
    });
    let expected: u128 = [scout, pirate, changeling]
        .into_iter()
        .map(|id| u128::try_from(game.object_view(id).power).unwrap())
        .sum();
    passes(&mut game);
    assert_eq!(game.state.step, Step::CombatDamage);
    assert_eq!(game.state.stack.len(), 1);
    let hit = game.state.stack[0].damage_event.unwrap();
    assert_eq!((hit.player, hit.amount), (opponent, expected));
    passes(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Exile);
    let object = game.object_ref(land);
    reject(
        &mut game,
        Action::CastExiledDuringResolution {
            object,
            face: CastFace::Normal,
        },
    );
    let finish = Action::FinishResolutionCasts;
    let saved = game.snapshot();
    let rev = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == finish)
        .unwrap();
    let mut indexed = game.clone();
    act(&mut game, finish);
    indexed.apply_index(player, rev, index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    assert_eq!(indexed.snapshot(), saved);
    assert_eq!(treasures(&game, player), 1);
}
