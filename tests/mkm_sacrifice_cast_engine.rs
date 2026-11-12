//! A simultaneous sacrifice batch funds an exact immediate free-cast offer.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const OTHER_CREATURE: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 1,
    nonland: false,
    other: true,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};

const ATTACK: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::HASTE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MaySacrificeAnyNumberForPower(OTHER_CREATURE),
            Instruction::ExileTopCastFree {
                amount: AmountValue::SacrificedPower,
                kinds: 4 | 8,
            },
        ],
    }],
    ..CardRules::EMPTY
};

fn attack(game: &mut Game, source: CardId) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(game, Action::Attack { card: source });
    act(game, Action::FinishAttackers);
    assert_eq!(game.state.stack.len(), 1);
}

fn resolve_attack(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn sacrifice(game: &Game, card: CardId) -> Action {
    Action::Sacrifice {
        object: game.object_ref(card),
    }
}

fn offer(game: &Game, card: CardId, face: CastFace) -> Action {
    Action::CastExiledDuringResolution {
        object: game.object_ref(card),
        face,
    }
}

fn finish_cast(game: &mut Game, target: Option<Target>, sacrifice: Option<ObjectRef>) {
    for _ in 0..12 {
        match game.state.pending {
            Pending::ResolutionCasts { .. } => return,
            Pending::CastCosts { .. } => choose(game, |action| {
                matches!(action, Action::ChooseCastCost { .. })
            }),
            Pending::AdditionalCost { .. } => act(
                game,
                Action::ChooseAdditionalCost {
                    object: sacrifice.expect("mandatory creature"),
                },
            ),
            Pending::PayCosts { .. } => pay(game),
            Pending::Targets { .. } => act(
                game,
                Action::ChooseTarget {
                    target: target.expect("spell target"),
                },
            ),
            ref other => panic!("unexpected free-cast continuation {other:?}"),
        }
    }
    panic!("free cast did not commit");
}

struct ClearPrevention;

impl Drop for ClearPrevention {
    fn drop(&mut self) {
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|slot| slot.set(None));
    }
}

#[test]
fn signed_last_known_power_exiles_the_exact_top_cohort_and_offers_only_spell_faces() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Bear Cub").unwrap(), &ATTACK)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        game.state.players[player.index()].mana = [0; 6];
        let attacker = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let lord = put(&mut game, player, "Imperious Perfect", Zone::Battlefield);
        let buffed = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
        let negative = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
        game.state.objects[negative as usize].growth.power = -3;
        let boosted = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        game.state.objects[boosted as usize].growth.power = 1;
        let prevented = put(&mut game, player, "Earthshaker Dreadmaw", Zone::Battlefield);
        let reserve = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let foreign = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
        put(&mut game, opponent, "Rest in Peace", Zone::Battlefield);
        let unrelated = put(&mut game, player, "Shock", Zone::Exile);
        let [
            outside,
            land,
            adventure,
            target_spell,
            x_spell,
            mandatory,
            creature,
        ] = [
            "Divination",
            "Forest",
            "Frolicking Familiar",
            "Shock",
            "Exsanguinate",
            "Corrupted Conviction",
            "Grizzly Bears",
        ]
        .map(|name| put(&mut game, player, name, Zone::Library));

        assert_eq!(stats(&game, buffed).0, 2);
        assert_eq!(stats(&game, negative).0, -1);
        attack(&mut game, attacker);
        resolve_attack(&mut game);
        for card in [attacker, foreign] {
            let illegal = sacrifice(&game, card);
            reject(&mut game, illegal);
        }

        let mut zero = game.clone();
        act(&mut zero, Action::FinishEffectChoice);
        assert_eq!(zero.object_view(creature).zone, Zone::Library);
        assert!(zero.observe(player).resolution_free_casts.is_none());

        let mut below_zero = game.clone();
        let choose_negative = sacrifice(&below_zero, negative);
        act(&mut below_zero, choose_negative);
        act(&mut below_zero, Action::FinishEffectChoice);
        assert_eq!(below_zero.object_view(negative).zone, Zone::Exile);
        assert_eq!(below_zero.object_view(creature).zone, Zone::Library);
        assert!(below_zero.observe(player).resolution_free_casts.is_none());

        for card in [lord, buffed, negative, boosted, prevented] {
            let action = sacrifice(&game, card);
            act(&mut game, action.clone());
            reject(&mut game, action);
            assert_eq!(game.object_view(card).zone, Zone::Battlefield);
        }
        {
            let _clear = ClearPrevention;
            zone_replacements::TEST_PREVENT_ZONE_MOVE
                .with(|slot| slot.set(Some(game.object_ref(prevented))));
            act(&mut game, Action::FinishEffectChoice);
            assert_eq!(game.state.pending, Pending::ZoneReplacement { player });
            act(
                &mut game,
                Action::ChooseZoneReplacement {
                    effect: ZoneReplacementKind::Prevent,
                },
            );
        }
        for card in [
            lord,
            buffed,
            negative,
            boosted,
            land,
            adventure,
            target_spell,
            x_spell,
            mandatory,
            creature,
        ] {
            assert_eq!(game.object_view(card).zone, Zone::Exile);
        }
        assert_eq!(game.object_view(prevented).zone, Zone::Battlefield);
        assert_eq!(game.object_view(outside).zone, Zone::Library);
        assert!(game.state.stack.is_empty());
        let cohort = game.observe(opponent).resolution_free_casts.unwrap();
        assert_eq!(cohort.remaining.len(), 6);
        assert_eq!(cohort.spell_kinds, 4 | 8);
        for card in [creature, land, outside, unrelated, adventure] {
            let illegal = offer(&game, card, CastFace::Normal);
            reject(&mut game, illegal);
        }
        assert!(
            game.legal_actions()
                .contains(&offer(&game, adventure, CastFace::Adventure))
        );
        reject(&mut game, Action::PlayLand { card: land });

        let mut stale = game.clone();
        stale.move_object(target_spell, Zone::Hand);
        stale.move_object(target_spell, Zone::Exile);
        let stale_offer = offer(&stale, target_spell, CastFace::Normal);
        reject(&mut stale, stale_offer);

        let mut declined = game.clone();
        act(&mut declined, Action::FinishResolutionCasts);
        assert_eq!(declined.object_view(target_spell).zone, Zone::Exile);
        let declined_offer = offer(&declined, target_spell, CastFace::Normal);
        reject(&mut declined, declined_offer);

        let cast_x = offer(&game, x_spell, CastFace::Normal);
        let index = game
            .legal_actions()
            .iter()
            .position(|action| *action == cast_x)
            .unwrap();
        let saved = game.snapshot();
        let mut indexed = game.clone();
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        act(&mut game, cast_x.clone());
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&saved).unwrap();
        act(&mut indexed, cast_x);
        assert_eq!(game.snapshot(), indexed.snapshot());
        reject(&mut game, Action::ChooseX { value: 1 });
        finish_cast(&mut game, None, None);
        assert_eq!(game.state.stack.last().unwrap().x, Some(0));

        let cast_target = offer(&game, target_spell, CastFace::Normal);
        act(&mut game, cast_target);
        finish_cast(&mut game, Some(Target::Player(opponent)), None);

        let cast_mandatory = offer(&game, mandatory, CastFace::Normal);
        act(&mut game, cast_mandatory);
        let exact_reserve = game.object_ref(reserve);
        finish_cast(&mut game, None, Some(exact_reserve));
        assert_eq!(game.object_view(reserve).zone, Zone::Exile);
        assert_eq!(game.state.stack.len(), 3);
        for spell in &game.state.stack {
            let cast = spell.cast.unwrap();
            assert_eq!(cast.method, CastMethod::Free);
            assert_eq!(cast.payment, [0; 6]);
        }

        act(&mut game, Action::FinishResolutionCasts);
        settle(&mut game);
        assert_eq!(game.state.players[opponent.index()].life, STARTING_LIFE - 2);
        assert_eq!(game.object_view(adventure).zone, Zone::Exile);
        assert_eq!(game.object_view(land).zone, Zone::Exile);
        assert_eq!(game.object_view(attacker).zone, Zone::Battlefield);
    }
}

#[test]
fn other_creature_exclusion_is_qualified_by_the_attackers_exact_incarnation() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Bear Cub").unwrap(), &ATTACK)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let attacker = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    attack(&mut game, attacker);
    let prior = game.object_ref(attacker);
    game.move_object(attacker, Zone::Hand);
    game.move_object(attacker, Zone::Battlefield);
    assert_ne!(game.object_ref(attacker), prior);
    resolve_attack(&mut game);
    reject(&mut game, Action::Sacrifice { object: prior });
    let current = sacrifice(&game, attacker);
    act(&mut game, current);
    act(&mut game, Action::FinishEffectChoice);
    assert_eq!(game.object_view(attacker).zone, Zone::Graveyard);
    assert_eq!(
        game.observe(player)
            .resolution_free_casts
            .unwrap()
            .remaining
            .len(),
        4
    );
    act(&mut game, Action::FinishResolutionCasts);
}
