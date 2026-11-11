//! Source-independent attack prohibitions and live cast restrictions by defender seat.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const PROHIBIT: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::OpponentsWhoAttackedControllerOrPlaneswalkerCannotCast],
}];
const WANDERGLYPH: CardRules = CardRules {
    effect: Effect::Creature(5, 3),
    static_groups: PROHIBIT,
    extras: &CardExtras {
        static_ability_groups: PROHIBIT,
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::DuringOpponentsTurn(&TriggerCondition::SpellCast {
            relation: CastRelation::Opponent,
            filter: CastFilter::Any,
            ordinal: None,
        }),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ProhibitAttacksOnControllerOrPlaneswalkersThisTurn(
                PlayerSet::CastingPlayer,
            ),
        ],
    }],
    ..CardRules::EMPTY
};
const DISCOVER: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        cost: ActivationCost::ZERO,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        once_per_turn: false,
        sorcery: false,
        targets: TargetPlan::NONE,
        program: &[Instruction::Discover(2)],
    }],
    ..CardRules::EMPTY
};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn attackers(game: &mut Game) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
}

fn siege(game: &mut Game, player: Player) -> ObjectRef {
    let definition = Card::from_name("@fixture/siege-front").unwrap();
    let id = spawn(game, player, definition, Zone::Battlefield);
    game.add_counter(id, CounterKind::Defense, 9);
    let object = game.object_ref(id);
    game.set_battle_protector(object, player.other());
    object
}

fn walker(game: &mut Game, player: Player) -> ObjectRef {
    let id = put(
        game,
        player,
        "Ajani, Caller of the Pride",
        Zone::Battlefield,
    );
    game.add_counter(id, CounterKind::Loyalty, 4);
    game.object_ref(id)
}

fn hand_cast(game: &Game, card: CardId) -> Option<Action> {
    game.legal_actions().into_iter().find(|action| {
        matches!(action, Action::Cast { card: id, .. }
            | Action::BeginCast { card: id, .. }
            | Action::AnnounceCast { card: id } if *id == card)
    })
}

#[test]
fn resolved_cast_trigger_protects_the_player_and_planeswalkers_after_source_departure() {
    let definition = Card::from_name("Cartographer's Companion").unwrap();
    let _rules = fixture_rules::activate(&[(definition, &WANDERGLYPH)]);
    let player = Player::P0;
    let defender = player.other();
    let mut game = scenario(player);
    let source = spawn(&mut game, defender, definition, Zone::Battlefield);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let walker = walker(&mut game, defender);
    let battle = siege(&mut game, player);
    let spell = put(&mut game, player, "Healer's Hawk", Zone::Hand);
    let instant = put(&mut game, player, "Giant Growth", Zone::Hand);
    cast(&mut game, spell, None);
    assert_eq!(game.state.stack.len(), 2);
    let trigger = game.state.stack.last().unwrap();
    assert_eq!(trigger.controller, defender);
    assert_eq!(trigger.cast_event.as_ref().unwrap().controller, player);
    assert!(hand_cast(&game, instant).is_some());
    assert!(game.can_attack_defender(creature, CombatDefender::Player(defender)));

    let mut left_early = game.clone();
    left_early.move_object(source, Zone::Graveyard);
    settle(&mut left_early);
    settle(&mut game);
    game.move_object(source, Zone::Graveyard);
    for mut branch in [game, left_early] {
        attackers(&mut branch);
        reject(&mut branch, Action::Attack { card: creature });
        reject(
            &mut branch,
            Action::AttackPlaneswalker {
                card: creature,
                defender: walker,
            },
        );
        let saved = branch.snapshot();
        let allowed = Action::AttackBattle {
            card: creature,
            defender: battle,
        };
        let index = branch
            .legal_actions()
            .iter()
            .position(|action| *action == allowed)
            .unwrap();
        let mut indexed = branch.clone();
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        act(&mut branch, allowed.clone());
        assert_eq!(indexed.snapshot(), branch.snapshot());
        indexed.restore(&saved).unwrap();
        act(&mut indexed, allowed);
        assert_eq!(indexed.snapshot(), branch.snapshot());
        indexed.begin_combat_rules_turn();
        assert!(indexed.can_attack_defender(creature, CombatDefender::Player(defender)));

        act(&mut branch, Action::FinishAttackers);
        spawn(&mut branch, defender, definition, Zone::Battlefield);
        let entrant = put(&mut branch, player, "Grizzly Bears", Zone::Graveyard);
        let mut movement = branch.entry_move(entrant, Zone::Battlefield, true);
        movement.choose_attacking_defender = true;
        branch.stage_entries(vec![movement], None);
        assert!(branch.advance_entries());
        act(
            &mut branch,
            Action::ChooseEntryAttackingDefender {
                defender: CombatDefender::Planeswalker(walker),
            },
        );
        assert!(branch.state.attackers.iter().any(|row| {
            row.card == entrant && row.defender == Some(CombatDefender::Planeswalker(walker))
        }));
        assert!(!branch.observe(player).casting_prohibited);
        assert!(hand_cast(&branch, instant).is_some());
    }

    let mut off_turn = scenario(defender);
    spawn(&mut off_turn, defender, definition, Zone::Battlefield);
    let creature = put(&mut off_turn, player, "Grizzly Bears", Zone::Battlefield);
    let instant = put(&mut off_turn, player, "Giant Growth", Zone::Hand);
    act(&mut off_turn, Action::Pass);
    cast(&mut off_turn, instant, Some(Target::Card(creature)));
    assert_eq!(off_turn.state.stack.len(), 1);
}

#[test]
fn player_and_planeswalker_declarations_block_casts_while_a_later_provider_is_live() {
    let definition = Card::from_name("Cartographer's Companion").unwrap();
    let discovery = Card::from_name("Short Bow").unwrap();
    let _rules = fixture_rules::activate(&[(definition, &WANDERGLYPH), (discovery, &DISCOVER)]);
    let player = Player::P0;
    let defender = player.other();
    let mut game = scenario(player);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let walker = walker(&mut game, defender);
    let battle = siege(&mut game, player);
    let instant = put(&mut game, player, "Opt", Zone::Hand);
    let land = put(&mut game, player, "Forest", Zone::Hand);
    let found = put(&mut game, player, "Opt", Zone::Library);
    let discover = spawn(&mut game, player, discovery, Zone::Battlefield);
    attackers(&mut game);
    for (attack, prohibited) in [
        (Action::Attack { card: creature }, true),
        (
            Action::AttackPlaneswalker {
                card: creature,
                defender: walker,
            },
            true,
        ),
        (
            Action::AttackBattle {
                card: creature,
                defender: battle,
            },
            false,
        ),
    ] {
        let mut branch = game.clone();
        act(&mut branch, attack);
        act(&mut branch, Action::FinishAttackers);
        let ordinary = hand_cast(&branch, instant).unwrap();
        branch.move_object(walker.card, Zone::Graveyard);
        let provider = spawn(&mut branch, defender, definition, Zone::Battlefield);
        assert_eq!(branch.observe(player).casting_prohibited, prohibited);
        if !prohibited {
            assert!(branch.legal_actions().contains(&ordinary));
            continue;
        }
        reject(&mut branch, ordinary.clone());
        branch.state.step = Step::Main2;
        assert!(
            branch
                .legal_actions()
                .contains(&Action::PlayLand { card: land })
        );
        act(
            &mut branch,
            Action::AnnounceActivate {
                card: discover,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        if matches!(branch.state.pending, Pending::PayCosts { .. }) {
            pay(&mut branch);
        }
        pass_pair(&mut branch);
        assert_eq!(branch.state.pending, Pending::ResolutionCasts { player });
        assert_eq!(
            branch
                .legal_actions()
                .into_iter()
                .filter(|action| *action != Action::Concede)
                .collect::<Vec<_>>(),
            vec![Action::FinishResolutionCasts]
        );
        act(&mut branch, Action::FinishResolutionCasts);
        assert_eq!(branch.object_view(found).zone, Zone::Hand);
        branch.move_object(provider, Zone::Graveyard);
        assert!(!branch.observe(player).casting_prohibited);
        assert!(hand_cast(&branch, instant).is_some());
    }
}
