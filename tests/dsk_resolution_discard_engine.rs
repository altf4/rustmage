//! Private resolution discards, exact batch results and a separate bounded trigger.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardRules, one};
use rustmage_core::{cards::*, engine::*};
use support::*;

const DEMON: &str = "Grizzly Bears";
const FANATIC: &str = "Bear Cub";
const AURA: &str = "Witness Protection";
const DEMON_RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::MayDiscardAnyReflexive(1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Creature,
                min: 0,
                max: usize::MAX,
            }),
            program: &[
                Instruction::UpToTargets(AmountValue::ReflexiveTargetLimit),
                Instruction::Grow {
                    subject: Subject::Targets,
                    power: -2,
                    toughness: -2,
                },
            ],
        },
    ],
    ..CardRules::EMPTY
};
const FANATIC_RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::DiscardThenDrawForOwnCount {
            players: PlayerSet::Each,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
const AURA_RULES: CardRules = CardRules {
    effect: Effect::Aura(one(TargetRule::Creature)),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    }),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: one(TargetRule::Player {
                opponent_only: false,
            }),
            program: &[Instruction::LookTargetHandMayDiscardNonland],
        },
        TriggeredAbility {
            condition: TriggerCondition::AttachedCreatureDamaged,
            targets: TargetPlan::NONE,
            program: &[Instruction::DestroyEventPermanent],
        },
    ],
    ..CardRules::EMPTY
};

struct ClearPrevention;
impl Drop for ClearPrevention {
    fn drop(&mut self) {
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|slot| slot.set(None));
    }
}

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn creature_trigger(game: &mut Game, player: Player, name: &str) {
    let source = put(game, player, name, Zone::Hand);
    cast(game, source, None);
    pass_pair(game);
    pass_pair(game);
}

fn choose_hand(object: Option<ObjectRef>) -> Action {
    Action::ChooseHandCard { object }
}

fn pick_hand(game: &mut Game, card: CardId) {
    let object = game.object_ref(card);
    act(game, choose_hand(Some(object)));
}

fn target(target: Target) -> Action {
    Action::ChooseTarget { target }
}

#[test]
fn any_discard_is_one_private_batch_and_caps_a_separate_trigger_by_actual_moves() {
    let _rules = fixture_rules::activate(&[(card(DEMON), &DEMON_RULES)]);
    let _clear = ClearPrevention;
    let player = Player::P0;
    let enemy = player.other();
    let mut game = scenario(player);
    let land = put(&mut game, player, "Island", Zone::Hand);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    let prevented = put(&mut game, player, "Shock", Zone::Hand);
    put(&mut game, enemy, "Rest in Peace", Zone::Battlefield);
    let blink = put(&mut game, enemy, "Personify", Zone::Hand);
    let first = put(&mut game, enemy, "Hill Giant", Zone::Battlefield);
    let second = put(&mut game, enemy, "Hill Giant", Zone::Battlefield);
    let third = put(&mut game, enemy, "Hill Giant", Zone::Battlefield);

    creature_trigger(&mut game, player, DEMON);
    let choice = game.observe(player).hand_choice.unwrap();
    assert!(choice.optional);
    assert_eq!(choice.candidates.len(), 3);
    assert!(game.observe(enemy).hand_choice.is_none());
    assert!(game.observe(player).stack.is_empty());
    reject(&mut game, target(Target::Card(first)));

    let mut none = game.clone();
    act(&mut none, choose_hand(None));
    assert!(none.observe(player).stack.is_empty());
    assert_eq!(none.hand_count(player), 3);

    let land_ref = game.object_ref(land);
    let pick = choose_hand(Some(land_ref));
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == pick)
        .unwrap();
    let mut indexed = game.clone();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    act(&mut game, pick.clone());
    assert_eq!(indexed.snapshot(), game.snapshot());
    reject(&mut game, pick);
    pick_hand(&mut game, spell);
    pick_hand(&mut game, prevented);
    assert_eq!(game.hand_count(player), 3);
    assert!(game.observe(enemy).hand_choice.is_none());
    assert_eq!(game.observe(player).hand_choice.unwrap().selected.len(), 3);

    let mut all_three = game.clone();
    act(&mut all_three, choose_hand(None));
    assert_eq!(
        all_three
            .observe(player)
            .announcing
            .unwrap()
            .reflexive_target_limit,
        Some(3)
    );
    for creature in [first, second, third] {
        act(&mut all_three, target(Target::Card(creature)));
    }
    assert_eq!(
        all_three
            .observe(player)
            .stack
            .last()
            .unwrap()
            .targets
            .len(),
        3
    );

    zone_replacements::TEST_PREVENT_ZONE_MOVE
        .with(|slot| slot.set(Some(game.object_ref(prevented))));
    act(&mut game, choose_hand(None));
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|slot| slot.set(None));
    assert_eq!(game.object_view(land).zone, Zone::Exile);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    assert_eq!(game.object_view(prevented).zone, Zone::Hand);
    let reflexive = game.observe(player).announcing.unwrap();
    assert_eq!(reflexive.reflexive_target_limit, Some(2));
    assert!(reflexive.targets.is_empty());

    let mut no_targets = game.clone();
    act(&mut no_targets, Action::FinishTargets);
    settle(&mut no_targets);
    assert_eq!(stats(&no_targets, first), (3, 3));
    assert_eq!(stats(&no_targets, second), (3, 3));

    act(&mut game, target(Target::Card(first)));
    reject(&mut game, target(Target::Card(first)));
    act(&mut game, target(Target::Card(second)));
    let published = game.observe(player);
    assert_eq!(published.stack.last().unwrap().targets.len(), 2);
    reject(&mut game, target(Target::Card(third)));
    let first_before = game.object_ref(first);
    act(&mut game, Action::Pass);
    cast(&mut game, blink, Some(Target::Card(first)));
    pass_pair(&mut game);
    assert_ne!(game.object_ref(first), first_before);
    settle(&mut game);
    assert_eq!(stats(&game, first), (3, 3));
    assert_eq!(stats(&game, second), (1, 1));
    assert_eq!(stats(&game, third), (3, 3));
}

#[test]
fn each_player_discards_together_and_only_the_controllers_actual_discard_draws() {
    let _rules = fixture_rules::activate(&[(card(FANATIC), &FANATIC_RULES)]);
    let _clear = ClearPrevention;
    let player = Player::P1;
    let enemy = player.other();
    for (own_card, prevent_own) in [(true, false), (true, true), (false, false)] {
        let mut game = scenario(player);
        put(&mut game, enemy, "Rest in Peace", Zone::Battlefield);
        let own = own_card.then(|| put(&mut game, player, "Forest", Zone::Hand));
        let opposing = put(&mut game, enemy, "Opt", Zone::Hand);
        let drawn = *game.state.players[player.index()].library.last().unwrap();
        creature_trigger(&mut game, player, FANATIC);
        if let Some(own) = own {
            assert_eq!(game.actor(), Some(player));
            act(&mut game, Action::Discard { card: own });
            assert_eq!(game.object_view(own).zone, Zone::Hand);
            assert!(
                !game
                    .observe(enemy)
                    .objects
                    .iter()
                    .any(|object| object.id == own)
            );
            if prevent_own {
                zone_replacements::TEST_PREVENT_ZONE_MOVE
                    .with(|slot| slot.set(Some(game.object_ref(own))));
            }
        }
        assert_eq!(game.actor(), Some(enemy));
        assert_eq!(game.object_view(opposing).zone, Zone::Hand);
        act(&mut game, Action::Discard { card: opposing });
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|slot| slot.set(None));
        settle(&mut game);
        assert_eq!(game.object_view(opposing).zone, Zone::Exile);
        if let Some(own) = own {
            assert_eq!(
                game.object_view(own).zone,
                if prevent_own { Zone::Hand } else { Zone::Exile }
            );
        }
        assert_eq!(
            game.object_view(drawn).zone,
            if own_card && !prevent_own {
                Zone::Hand
            } else {
                Zone::Library
            }
        );
        assert_eq!(game.state.players[enemy.index()].cards_drawn_this_turn, 0);
    }
}

#[test]
fn the_private_look_allows_only_a_nonland_discard_without_a_draw() {
    let _rules = fixture_rules::activate(&[(card(AURA), &AURA_RULES)]);
    let player = Player::P0;
    let enemy = player.other();
    for victim in [player, enemy] {
        let mut game = scenario(player);
        let bearer = put(&mut game, enemy, "Hill Giant", Zone::Battlefield);
        let aura = put(&mut game, player, AURA, Zone::Hand);
        let land = put(&mut game, victim, "Forest", Zone::Hand);
        let stale = put(&mut game, victim, "Shock", Zone::Hand);
        let picked = put(&mut game, victim, "Opt", Zone::Hand);
        cast(&mut game, aura, Some(Target::Card(bearer)));
        pass_pair(&mut game);
        act(&mut game, target(Target::Player(victim)));
        pass_pair(&mut game);

        let choice = game.observe(player).hand_choice.unwrap();
        assert!(choice.optional);
        assert_eq!(choice.victim, Some(victim));
        assert_eq!(choice.inspected_cards.unwrap().len(), 3);
        assert_eq!(
            choice.candidates,
            [game.object_ref(stale), game.object_ref(picked)]
        );
        assert!(game.observe(enemy).hand_choice.is_none());
        assert!(
            !game
                .observe(enemy)
                .known_hand
                .contains(&game.object_ref(stale))
        );
        let land_action = choose_hand(Some(game.object_ref(land)));
        reject(&mut game, land_action);

        let saved = game.snapshot();
        act(&mut game, choose_hand(None));
        assert_eq!(game.hand_count(victim), 3);
        assert_eq!(game.state.players[victim.index()].cards_drawn_this_turn, 0);
        assert!(game.observe(player).hand_choice.is_none());

        game.restore(&saved).unwrap();
        let stale_ref = game.object_ref(stale);
        game.move_object(stale, Zone::Library);
        game.move_object(stale, Zone::Hand);
        reject(&mut game, choose_hand(Some(stale_ref)));
        pick_hand(&mut game, picked);
        assert_eq!(game.object_view(picked).zone, Zone::Graveyard);
        assert_eq!(game.hand_count(victim), 2);
        assert_eq!(game.state.players[victim.index()].cards_drawn_this_turn, 0);
        assert!(game.observe(player).hand_choice.is_none());
    }
}

#[test]
fn the_attached_creature_damage_trigger_keeps_its_exact_recipient() {
    let _rules = fixture_rules::activate(&[(card(AURA), &AURA_RULES)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let bearer = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let aura = put(&mut game, player, AURA, Zone::Hand);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, aura, Some(Target::Card(bearer)));
    pass_pair(&mut game);
    act(&mut game, target(Target::Player(player.other())));
    pass_pair(&mut game);
    assert!(
        game.observe(player)
            .hand_choice
            .unwrap()
            .inspected_cards
            .unwrap()
            .is_empty()
    );
    act(&mut game, choose_hand(None));

    cast(&mut game, shock, Some(Target::Card(bearer)));
    pass_pair(&mut game);
    assert_eq!(game.object_view(bearer).damage, 2);
    assert!(
        game.observe(player)
            .stack
            .last()
            .unwrap()
            .targets
            .is_empty()
    );
    game.move_object(aura, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(bearer).zone, Zone::Graveyard);
}
