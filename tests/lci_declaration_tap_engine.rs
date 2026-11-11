//! Declaration-only subtype triggers and simultaneous permanent tap receipts.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const GNOME: Card = Card(5267);
const MERFOLK: Card = Card(5398);
const WHITE: Mana = [1, 0, 0, 0, 0, 0];

fn definition(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn declare(game: &mut Game, attackers: &[CardId]) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn non_gnome_attack_group_creates_scaled_attackers_from_live_or_departure_counters() {
    const ANIM: CardRules = CardRules {
        effect: Effect::Creature(1, 2),
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::ControlledSubtypeAttacks {
                subtype: "Gnome",
                exclude: true,
                one_or_more: true,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::PutCounters {
                    subject: Subject::Source,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
                Instruction::CreateTokensAttackingScaled {
                    token: GNOME,
                    amount: AmountValue::SourceCounterQuantity(CounterKind::PlusOne),
                },
            ],
        }],
        ..CardRules::EMPTY
    };
    let definition = definition("Anim Pakal, Thousandth Moon");
    let _rules = fixture_rules::activate(&[(definition, &ANIM)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = spawn(&mut game, player, definition, Zone::Battlefield);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let changeling = put(&mut game, player, "Mutable Explorer", Zone::Battlefield);
    let walker = put(
        &mut game,
        player.other(),
        "Ajani, Caller of the Pride",
        Zone::Battlefield,
    );
    game.add_counter(walker, CounterKind::Loyalty, 4);
    let walker = game.object_ref(walker);
    let mut gnome_only = game.clone();
    declare(&mut gnome_only, &[changeling]);
    assert!(gnome_only.state.stack.is_empty());
    assert_eq!(gnome_only.counter_count(source, CounterKind::PlusOne), 0);

    declare(&mut game, &[first, second, changeling]);
    assert_eq!(game.state.stack.len(), 1);
    let trigger = &game.state.stack[0];
    assert_eq!(trigger.source.unwrap().id, source);
    let declared = trigger.reflexive_subject.as_ref().unwrap();
    assert_eq!(declared.object, game.object_ref(first));
    assert_eq!(declared.additional_objects, vec![game.object_ref(second)]);

    let mut departed = game.clone();
    departed.add_counter(source, CounterKind::PlusOne, 3);
    departed.move_object(source, Zone::Graveyard);
    put(&mut game, player, "Doubling Season", Zone::Battlefield);
    for (mut branch, tokens) in [(game, 4), (departed, 3)] {
        pass_pair(&mut branch);
        assert_eq!(branch.state.pending, Pending::Mobilize { player });
        let saved = branch.snapshot();
        let defender = CombatDefender::Planeswalker(walker);
        let choose_defender = Action::ChooseMobilizeDefender { defender };
        let index = branch
            .legal_actions()
            .iter()
            .position(|action| *action == choose_defender)
            .unwrap();
        let mut indexed = branch.clone();
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        act(&mut branch, choose_defender.clone());
        assert_eq!(indexed.snapshot(), branch.snapshot());
        indexed.restore(&saved).unwrap();
        act(&mut indexed, choose_defender);
        assert_eq!(indexed.snapshot(), branch.snapshot());
        for _ in 1..tokens {
            act(
                &mut branch,
                Action::ChooseMobilizeDefender {
                    defender: CombatDefender::Player(player.other()),
                },
            );
        }
        let token_attackers: Vec<_> = branch
            .state
            .attackers
            .iter()
            .filter(|row| {
                let object = branch.object_view(row.card);
                object.is_token() && object.card == GNOME
            })
            .collect();
        assert_eq!(token_attackers.len(), tokens);
        assert_eq!(
            token_attackers
                .iter()
                .filter(|row| row.defender == Some(defender))
                .count(),
            1
        );
        assert!(
            token_attackers
                .iter()
                .all(|row| branch.object_view(row.card).tapped)
        );
        assert!(branch.state.stack.is_empty());
        if branch.object_view(source).zone == Zone::Battlefield {
            assert_eq!(branch.counter_count(source, CounterKind::PlusOne), 2);
        }
    }
}

#[test]
fn each_declared_pirate_reserves_a_separate_unused_mode() {
    const MODES: &[SpellMode] = &[
        SpellMode {
            text: "Gain 1 life",
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        },
        SpellMode {
            text: "Target creature can't block this turn",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            program: &[Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::CANT_BLOCK,
            }],
        },
        SpellMode {
            text: "Draw a card",
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
    ];
    const BREECHES: CardRules = CardRules {
        effect: Effect::Creature(3, 3),
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::ControlledSubtypeAttacks {
                subtype: "Pirate",
                exclude: false,
                one_or_more: false,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::ModalOncePerTurn(MODES)],
        }],
        ..CardRules::EMPTY
    };
    let definition = definition("Breeches, Eager Pillager");
    let _rules = fixture_rules::activate(&[(definition, &BREECHES)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = spawn(&mut game, player, definition, Zone::Battlefield);
    let first = put(&mut game, player, "Goblin Boarders", Zone::Battlefield);
    let second = put(&mut game, player, "Goblin Boarders", Zone::Battlefield);
    let changeling = put(&mut game, player, "Mutable Explorer", Zone::Battlefield);
    let nonpirate = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let blocker = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let life = game.state.players[player.index()].life;
    declare(&mut game, &[source, first, second, changeling, nonpirate]);
    let pirates = game
        .state
        .ordering_triggers
        .iter()
        .map(|packet| packet.event_object.as_ref().unwrap().object.card)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        pirates,
        [source, first, second, changeling].into_iter().collect()
    );
    for mode in 0..3 {
        choose(&mut game, |action| {
            matches!(action, Action::PutTrigger { .. })
        });
        for used in 0..mode {
            reject(&mut game, Action::ChooseMode { mode: used });
        }
        act(&mut game, Action::ChooseMode { mode });
        if mode == 1 {
            act(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(blocker),
                },
            );
            if matches!(game.state.pending, Pending::Targets { .. }) {
                act(&mut game, Action::FinishTargets);
            }
        }
    }
    assert_eq!(game.state.pending, Pending::Priority);
    assert_eq!(game.state.stack.len(), 3);
    assert!(game.state.ordering_triggers.is_empty());
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, life + 1);
    assert_eq!(game.hand_count(player), 1);
    assert!(
        game.object_view(blocker)
            .keywords
            .contains(Keywords::CANT_BLOCK)
    );
}

#[test]
fn simultaneous_mana_cost_taps_trigger_once_for_only_controlled_nontoken_merfolk() {
    const PILGRIMAGE: CardRules = CardRules {
        effect: Effect::Permanent,
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::ControlledPermanentsOfSubtypeTapped {
                subtype: "Merfolk",
                nontoken: true,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: MERFOLK,
                amount: 1,
                tapped: false,
            }],
        }],
        ..CardRules::EMPTY
    };
    let definition = definition("Deeproot Pilgrimage");
    let _rules = fixture_rules::activate(&[(definition, &PILGRIMAGE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    spawn(&mut game, player, definition, Zone::Battlefield);
    let parents = put(&mut game, player, "Supportive Parents", Zone::Battlefield);
    let first = put(&mut game, player, "Brineborn Cutthroat", Zone::Battlefield);
    let second = put(&mut game, player, "Mutable Explorer", Zone::Battlefield);
    let opponent = put(
        &mut game,
        player.other(),
        "Brineborn Cutthroat",
        Zone::Battlefield,
    );
    let unrelated = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enters_tapped = put(&mut game, player, "Brineborn Cutthroat", Zone::Graveyard);
    let movement = game.entry_move(enters_tapped, Zone::Battlefield, true);
    game.stage_entries(vec![movement], None);
    assert!(!game.advance_entries());
    assert!(game.object_view(enters_tapped).tapped);
    assert!(game.state.pending_triggers.is_empty());
    let own_token = game.create_token_batch_capture(player, MERFOLK, 1, false, None, true)[0];
    game.tap_objects(&[opponent, own_token.card, unrelated]);
    assert!(game.state.pending_triggers.is_empty());

    game.state.players[player.index()].mana = [0; 6];
    let spell = put(&mut game, player, "Healer's Hawk", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    choose(&mut game, |action| {
        matches!(action, Action::AnnounceMana { card, production, .. }
            if *card == parents && [WHITE, [0; 6]].contains(production))
    });
    for id in [first, second] {
        let object = game.object_ref(id);
        act(&mut game, Action::ChooseAdditionalCost { object });
    }
    pay(&mut game);
    if matches!(game.state.pending, Pending::EffectMana { .. }) {
        act(&mut game, Action::ChooseMana { production: WHITE });
    }
    assert_eq!(game.state.pending_triggers.len(), 1);
    let event = game.state.pending_triggers[0]
        .event_object
        .as_ref()
        .unwrap();
    assert_eq!(event.object, game.object_ref(first));
    assert_eq!(
        event.semantic_objects.as_deref().unwrap().as_slice(),
        &[DamageSourceRef::Object(game.object_ref(second))]
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
    assert_eq!(
        game.state
            .objects
            .iter()
            .filter(|object| {
                object.zone == Zone::Battlefield && object.is_token() && object.card == MERFOLK
            })
            .count(),
        2
    );

    let third = put(&mut game, player, "Brineborn Cutthroat", Zone::Battlefield);
    let fourth = put(&mut game, player, "Brineborn Cutthroat", Zone::Battlefield);
    game.tap_objects(&[third]);
    game.tap_objects(&[fourth]);
    assert_eq!(game.state.pending_triggers.len(), 2);
}
