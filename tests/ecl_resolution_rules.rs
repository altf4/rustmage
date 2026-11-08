//! Owned Lorwyn resolution steps, exercised before admitting their card consumers.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const CREATURE: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Creature));
const GOBLIN: Card = Card::test_index(5299);
const ALL_COLORS: Keywords = Keywords::PROTECTION_WHITE
    .union(Keywords::PROTECTION_BLUE)
    .union(Keywords::PROTECTION_BLACK)
    .union(Keywords::PROTECTION_RED)
    .union(Keywords::PROTECTION_GREEN);
const PLAYER: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Player {
    opponent_only: false,
}));
const fn spell(targets: TargetPlan, program: &'static [Instruction]) -> CardRules {
    CardRules {
        effect: Effect::Spell(targets, program),
        ..CardRules::EMPTY
    }
}
const PAIR: CardRules = spell(
    TargetPlan::NONE,
    &[Instruction::ChooseObjects {
        kind: EffectChoiceKind::PowerDifferenceBoost,
        players: PlayerSet::Controller,
        filter: SacrificeFilter {
            keywords: Keywords::NONE,
            source_only: false,
            kinds: 1,
            nonland: false,
            other: false,
            tokens: None,
            artifact_enchantment_or_token: false,
            subtype: None,
        },
        amount: 2,
        optional_skip: None,
    }],
);
const HAND_ATTACK: CardRules = spell(
    TargetPlan::NONE,
    &[Instruction::PutHandCreatureAttacking {
        maximum: AmountValue::Fixed(2),
    }],
);
const MILL_TARGET: CardRules = spell(
    PLAYER,
    &[
        Instruction::Mill(PlayerSet::Targeted, MillAmount::Fixed(3)),
        Instruction::SelectTargetPlayerMilled(MilledSelectionSpec {
            filter: MilledFilter::Kinds(1),
            count: MilledCount::Range {
                minimum: 0,
                maximum: 1,
            },
            destination: MilledDestination::Hand,
        }),
    ],
);
const BLANK: CardRules = spell(
    PLAYER,
    &[Instruction::SetPlayerCreatureCharacteristics {
        players: PlayerSet::Targeted,
        additional_types: &["Coward"],
        power: 1,
        toughness: 1,
        remove_abilities: true,
    }],
);
const NEXT_TURN: CardRules = spell(
    CREATURE,
    &[Instruction::GrantKeywordsUntilNextTurn {
        subject: Subject::Targets,
        keywords: ALL_COLORS,
    }],
);
const BOUNCE: CardRules = spell(
    CREATURE,
    &[
        Instruction::ReturnToHandCounted,
        Instruction::CreateTokensScaled {
            token: GOBLIN,
            amount: AmountValue::MovedCount,
            tapped: false,
        },
    ],
);
const CRIB: CardRules = spell(
    CREATURE,
    &[
        Instruction::Exile,
        Instruction::CreateTokensFor {
            players: PlayerSet::TargetController,
            token: GOBLIN,
            amount: 1,
            tapped: false,
        },
    ],
);
const COUNTER_DRAW: CardRules = spell(
    TargetPlan::Single(TargetSpec::one(TargetRule::Spell)),
    &[Instruction::CounterAndDrawForController { amount: 1 }],
);
const COUNTER_ALL: CardRules = spell(
    TargetPlan::NONE,
    &[
        Instruction::CounterOpponentsCounted,
        Instruction::CreateTokensScaled {
            token: GOBLIN,
            amount: AmountValue::MovedCount,
            tapped: false,
        },
    ],
);
const EMPTY_SPELL: CardRules = spell(TargetPlan::NONE, &[]);
const WATCH: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ScheduleTurnTrigger {
            event: TurnCallbackEvent::ControlledCreatureCombatDamagePlayerOrPlaneswalker,
            ability: 0,
            once: false,
        }],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::DelayedTargetEvent,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

fn checked(g: &mut Game, action: Action) {
    let saved = g.snapshot();
    let actor = g.actor().unwrap();
    let rev = g.revision();
    let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
    let mut raw = g.clone();
    let mut indexed = {
        let parent = g.clone();
        parent.clone()
    };
    raw.apply(actor, action.clone()).unwrap();
    indexed.apply_index(actor, rev, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    for p in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(raw.observe(p)).unwrap(),
            serde_json::to_value(indexed.observe(p)).unwrap()
        );
    }
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(actor, rev, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, action.clone());
    act(g, action);
    assert_eq!(g.snapshot(), indexed.snapshot());
}

fn start(g: &mut Game, player: Player, target: Option<Target>) -> CardId {
    let id = put(g, player, "Shock", Zone::Hand);
    g.state.priority = player;
    cast(g, id, target);
    id
}
fn resolve_top(g: &mut Game) {
    act(g, Action::Pass);
    checked(g, Action::Pass);
}
fn tokens(g: &Game, player: Player) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| o.zone == Zone::Battlefield && o.controller == player && o.is_token())
        .count()
}

#[test]
fn power_difference_waits_for_private_draw_choice_and_preserves_owned_recipients() {
    let _scope = fixture_rules::activate(&[(Card::Shock, &PAIR)]);
    let mut g = scenario(Player::P1);
    let a = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let b = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    put(
        &mut g,
        Player::P1,
        "Lorehold, the Historian",
        Zone::Battlefield,
    );
    g.state.objects[a as usize].growth.power = -3;
    start(&mut g, Player::P1, None);
    resolve_top(&mut g);
    let first = Action::ChoosePowerDifferenceCreature {
        object: g.object_ref(a),
    };
    checked(&mut g, first.clone());
    reject(&mut g, first);
    let second = Action::ChoosePowerDifferenceCreature {
        object: g.object_ref(b),
    };
    checked(&mut g, second);
    assert!(matches!(g.state.pending, Pending::MiracleReveal { .. }));
    assert!(g.observe(Player::P1).miracle.is_some());
    assert!(g.observe(Player::P0).miracle.is_none());
    assert_eq!(stats(&g, a), (-1, 2), "growth waits until drawing finishes");
    checked(&mut g, Action::DeclineMiracle);
    settle(&mut g);
    assert_eq!(g.state.players[1].cards_drawn_this_turn, 4);
    assert_eq!(stats(&g, a), (3, 6));
    assert_eq!(stats(&g, b), (7, 7));
    assert!(g.keywords(a).contains(Keywords::TRAMPLE));
}

#[test]
fn power_difference_with_only_one_creature_does_nothing() {
    let _scope = fixture_rules::activate(&[(Card::Shock, &PAIR)]);
    let mut g = scenario(Player::P0);
    let a = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    start(&mut g, Player::P0, None);
    settle(&mut g);
    assert_eq!(stats(&g, a), (2, 2));
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
}

#[test]
fn hand_entry_stays_private_when_empty_and_filters_mana_before_attacking_entry() {
    let _scope = fixture_rules::activate(&[(Card::Shock, &HAND_ATTACK)]);
    for eligible in [false, true] {
        let mut g = scenario(Player::P0);
        g.state.step = Step::Attackers;
        let large = put(&mut g, Player::P0, "Hill Giant", Zone::Hand);
        let small = eligible.then(|| put(&mut g, Player::P0, "Grizzly Bears", Zone::Hand));
        start(&mut g, Player::P0, None);
        resolve_top(&mut g);
        assert_eq!(
            g.state.pending,
            Pending::EffectChoice { player: Player::P0 }
        );
        assert!(g.observe(Player::P1).effect_choice.is_none());
        let bad = Action::PutHandCreature {
            object: g.object_ref(large),
        };
        reject(&mut g, bad);
        if let Some(small) = small {
            let action = Action::PutHandCreature {
                object: g.object_ref(small),
            };
            checked(&mut g, action);
            settle(&mut g);
            assert_eq!(g.state.objects[small as usize].zone, Zone::Battlefield);
            assert!(g.state.objects[small as usize].tapped);
            assert!(
                g.state
                    .attackers
                    .iter()
                    .any(|a| a.card == small
                        && a.defender == Some(CombatDefender::Player(Player::P1)))
            );
        } else {
            assert_eq!(
                g.legal_actions().into_iter().filter(|a| *a != Action::Concede).collect::<Vec<_>>(),
                vec![Action::DeclineHandEntry]
            );
            checked(&mut g, Action::DeclineHandEntry);
            settle(&mut g);
        }
    }
}

#[test]
fn attacking_hand_entry_can_choose_an_opposing_planeswalker() {
    let _scope = fixture_rules::activate(&[(Card::Shock, &HAND_ATTACK)]);
    let mut g = scenario(Player::P1);
    g.state.step = Step::Attackers;
    let walker = put(
        &mut g,
        Player::P0,
        "Elspeth, Storm Slayer",
        Zone::Battlefield,
    );
    g.add_counter(walker, CounterKind::Loyalty, 5);
    let small = put(&mut g, Player::P1, "Grizzly Bears", Zone::Hand);
    start(&mut g, Player::P1, None);
    resolve_top(&mut g);
    let action = Action::PutHandCreature {
        object: g.object_ref(small),
    };
    checked(&mut g, action);
    assert_eq!(
        g.state.pending,
        Pending::HandDefender { player: Player::P1 }
    );
    let defender = CombatDefender::Planeswalker(g.object_ref(walker));
    checked(&mut g, Action::ChooseHandDefender { defender });
    settle(&mut g);
    assert!(
        g.state
            .attackers
            .iter()
            .any(|a| a.card == small && a.defender == Some(defender))
    );
}

#[test]
fn milled_creature_choice_belongs_to_the_targeted_player_and_exact_batch() {
    let _scope = fixture_rules::activate(&[(Card::Shock, &MILL_TARGET)]);
    for target in [Player::P0, Player::P1] {
        let mut g = scenario(Player::P0);
        let old = put(&mut g, target, "Grizzly Bears", Zone::Graveyard);
        let creature = put(&mut g, target, "Hill Giant", Zone::Library);
        put(&mut g, target, "Forest", Zone::Library);
        put(&mut g, target, "Forest", Zone::Library);
        start(&mut g, Player::P0, Some(Target::Player(target)));
        resolve_top(&mut g);
        assert_eq!(g.state.pending, Pending::MilledSelection { player: target });
        let old_action = Action::SelectMilled {
            object: g.object_ref(old),
        };
        reject(&mut g, old_action);
        let action = Action::SelectMilled {
            object: g.object_ref(creature),
        };
        checked(&mut g, action);
        settle(&mut g);
        assert_eq!(g.state.objects[creature as usize].zone, Zone::Hand);
        assert_eq!(g.state.objects[old as usize].zone, Zone::Graveyard);
    }
}

#[test]
fn permanent_group_change_keeps_types_and_later_keywords_and_expires_on_blink() {
    const BODY: CardRules = CardRules {
        effect: Effect::Creature(3, 3),
        keywords: Keywords::FLYING.union(Keywords::HASTE),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(Card::Shock, &BLANK), (Card::HillGiant, &BODY)]);
    let mut g = scenario(Player::P0);
    let target = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    let own = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let original_kinds = g.current_kinds(target);
    start(&mut g, Player::P0, Some(Target::Player(Player::P1)));
    settle(&mut g);
    assert_eq!(stats(&g, target), (1, 1));
    assert_eq!(g.current_kinds(target), original_kinds);
    assert!(g.has_subtype(target, "Giant") && g.has_subtype(target, "Coward"));
    assert!(!g.keywords(target).contains(Keywords::FLYING));
    assert_eq!(stats(&g, own), (2, 2));
    g.add_counter(target, CounterKind::Flying, 1);
    assert!(g.keywords(target).contains(Keywords::FLYING));
    g.expire_animations();
    assert_eq!(stats(&g, target), (1, 1));
    g.move_object(target, Zone::Graveyard);
    g.move_object(target, Zone::Battlefield);
    assert_eq!(stats(&g, target), (3, 3));
    assert!(!g.has_subtype(target, "Coward"));
    assert!(g.keywords(target).contains(Keywords::HASTE));
}

#[test]
fn next_turn_keyword_duration_uses_original_controller_and_exact_incarnation() {
    let _scope = fixture_rules::activate(&[(Card::Shock, &NEXT_TURN)]);
    let mut g = scenario(Player::P1);
    let target = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    start(&mut g, Player::P1, Some(Target::Card(target)));
    settle(&mut g);
    g.state.objects[target as usize].controller = Player::P0;
    g.finish_cleanup();
    assert_eq!(g.state.active, Player::P0);
    assert!(g.keywords(target).contains(ALL_COLORS));
    let saved = g.snapshot();
    g.finish_cleanup();
    assert_eq!(g.state.active, Player::P1);
    assert!(!g.keywords(target).contains(ALL_COLORS));
    g.restore(&saved).unwrap();
    g.move_object(target, Zone::Graveyard);
    g.move_object(target, Zone::Battlefield);
    assert!(!g.keywords(target).contains(ALL_COLORS));
}

#[test]
fn returned_token_counts_before_it_ceases_to_exist() {
    let _scope = fixture_rules::activate(&[(Card::Shock, &BOUNCE)]);
    let mut g = scenario(Player::P0);
    g.create_tokens(Player::P0, GOBLIN, 1, false);
    g.advance_replacements(true);
    let token = g
        .state
        .objects
        .iter()
        .enumerate()
        .find(|(_, o)| o.is_token())
        .unwrap()
        .0 as CardId;
    start(&mut g, Player::P0, Some(Target::Card(token)));
    resolve_top(&mut g);
    settle(&mut g);
    assert_eq!(tokens(&g, Player::P0), 1);
    assert!(!g.on_battlefield(token));
}

#[test]
fn exile_then_token_creation_keeps_the_exiled_permanents_controller() {
    let _scope = fixture_rules::activate(&[(Card::Shock, &CRIB)]);
    let mut g = scenario(Player::P0);
    let target = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
    g.state.objects[target as usize].controller = Player::P1;
    start(&mut g, Player::P0, Some(Target::Card(target)));
    settle(&mut g);
    assert_eq!(g.state.objects[target as usize].zone, Zone::Exile);
    assert_eq!(tokens(&g, Player::P1), 1);
    assert_eq!(tokens(&g, Player::P0), 0);
}

#[test]
fn counter_and_draw_still_draws_for_an_uncounterable_spells_controller() {
    let _scope = fixture_rules::activate(&[
        (Card::Shock, &COUNTER_DRAW),
        (Card::LightningBolt, &EMPTY_SPELL),
    ]);
    let mut g = scenario(Player::P1);
    let victim = put(&mut g, Player::P1, "Lightning Bolt", Zone::Hand);
    cast(&mut g, victim, None);
    let id = g.state.stack.last().unwrap().id;
    g.state
        .uncounterable_spell_ids
        .get_or_insert_with(Default::default)
        .push(id);
    start(&mut g, Player::P0, Some(Target::Spell(victim)));
    resolve_top(&mut g);
    assert!(g.state.stack.iter().any(|item| item.id == id));
    assert_eq!(g.state.players[1].cards_drawn_this_turn, 1);
}

#[test]
fn simultaneous_counter_counts_only_opposing_counterable_stack_items() {
    let _scope = fixture_rules::activate(&[
        (Card::Shock, &COUNTER_ALL),
        (Card::LightningBolt, &EMPTY_SPELL),
    ]);
    let mut g = scenario(Player::P0);
    let mut ids = Vec::new();
    for player in [Player::P0, Player::P1, Player::P1] {
        let card = put(&mut g, player, "Lightning Bolt", Zone::Hand);
        g.state.priority = player;
        cast(&mut g, card, None);
        ids.push(g.state.stack.last().unwrap().id);
    }
    g.state
        .uncounterable_spell_ids
        .get_or_insert_with(Default::default)
        .push(ids[2]);
    start(&mut g, Player::P0, None);
    resolve_top(&mut g);
    assert_eq!(tokens(&g, Player::P0), 1);
    assert_eq!(
        g.state.stack.iter().map(|item| item.id).collect::<Vec<_>>(),
        vec![ids[0], ids[2]]
    );
}

#[test]
fn combat_callback_survives_its_source_and_uses_actual_dealer_controller() {
    let _scope = fixture_rules::activate(&[(Card::Shock, &WATCH)]);
    let mut g = scenario(Player::P0);
    let source = start(&mut g, Player::P0, None);
    settle(&mut g);
    assert_eq!(g.state.objects[source as usize].zone, Zone::Graveyard);
    let dealer = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let event = DamageEvent {
        source: g.damage_source(dealer),
        target: Target::Player(Player::P1),
        amount: 1,
        preventable: true,
        combat: true,
    };
    g.state.objects[dealer as usize].controller = Player::P1;
    g.apply_damage_batch(&[event]);
    g.advance_replacements(true);
    assert_eq!(g.state.pending_triggers.len(), 1);
    g.prepare_triggers();
    settle(&mut g);
    assert_eq!(g.state.players[0].life, 21);
    let life = g.state.players[0].life;
    let mut noncombat = event;
    noncombat.combat = false;
    g.apply_damage_batch(&[noncombat]);
    g.advance_replacements(true);
    g.prepare_triggers();
    settle(&mut g);
    assert_eq!(g.state.players[0].life, life);
}
