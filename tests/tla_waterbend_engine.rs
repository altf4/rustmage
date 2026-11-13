//! Waterbend funding routes, exact permissions and owned payment windows.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::engine::mana_lots::{ManaRestrictionView, Quality};
use rustmage_core::{cards::*, engine::*};
use support::*;

const DRAW: &[Instruction] = &[Instruction::Draw(PlayerSet::Controller, 1)];
const FIXED: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, DRAW),
    extras: &CardExtras {
        waterbend_additional: Some(WaterbendAdditional::Required(2)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const OPTIONAL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::IfWaterbendPaid { skip: 1 }, DRAW[0]],
    ),
    extras: &CardExtras {
        waterbend_additional: Some(WaterbendAdditional::Optional(2)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const WITH_CONVOKE: CardRules = CardRules {
    extras: &CardExtras {
        has_convoke: true,
        ..*FIXED.extras
    },
    ..FIXED
};
const X_CAST: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 1,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 0,
            max: usize::MAX,
        }),
        &[
            Instruction::TargetsExactlyX,
            Instruction::DrawScaled(PlayerSet::Controller, AmountValue::ChosenX),
        ],
    ),
    extras: &CardExtras {
        waterbend_additional: Some(WaterbendAdditional::RequiredX),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const X_ACTIVATION: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::OwnTurn,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::WaterbendXCost { minimum: 1 },
            Instruction::DrawScaled(PlayerSet::Controller, AmountValue::ChosenX),
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const X_STATS_ACTIVATION: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    activated: &[ActivatedAbility {
        program: &[
            Instruction::WaterbendXCost { minimum: 1 },
            Instruction::SetPermanentGroupBaseStats {
                filters: &[BattlefieldFilter {
                    kinds: 1,
                    relation: 0,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: u32::MAX,
                }],
                exclude_source: false,
                power: Some(ResolvedStatValue::ChosenX),
                toughness: Some(ResolvedStatValue::ChosenX),
            },
        ],
        ..X_ACTIVATION.activated[0]
    }],
    ..CardRules::EMPTY
};
const SACRIFICE_MANA: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        cost: ActivationCost {
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 0, 0, 0, 1,
        ]))],
        ..X_ACTIVATION.activated[0]
    }],
    ..CardRules::EMPTY
};
const RESOLUTION: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::Draw(PlayerSet::Controller, 3),
            Instruction::MayPayWaterbend {
                amount: 2,
                unpaid_skip: 1,
            },
            Instruction::Skip(1),
            Instruction::Discard(PlayerSet::Controller, 1),
        ],
    ),
    ..CardRules::EMPTY
};
const WARD: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Ward,
        targets: TargetPlan::NONE,
        program: &[Instruction::CounterRelatedUnlessWaterbend(2)],
    }],
    ..CardRules::EMPTY
};
const LINKED: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: true,
        })),
        program: &[
            Instruction::Mill(PlayerSet::Targeted, MillAmount::Fixed(3)),
            Instruction::ExileTargetGraveyardForWaterbend,
        ],
    }],
    ..CardRules::EMPTY
};
const BEND_WATCHER: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::BendsYou,
        targets: TargetPlan::NONE,
        program: DRAW,
    }],
    ..CardRules::EMPTY
};
const COPY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Spell)),
        &[Instruction::CopyTargetSpell {
            group: 0,
            retarget: false,
            exceptions: CopyExceptions::NONE,
        }],
    ),
    ..CardRules::EMPTY
};

fn waterbender(game: &Game, card: CardId) -> Action {
    Action::ChooseWaterbender {
        object: game.object_ref(card),
    }
}

fn choose_waterbender(game: &mut Game, card: CardId) {
    let action = waterbender(game, card);
    act(game, action);
}

fn activate(source: CardId) -> Action {
    Action::AnnounceActivate {
        card: source,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

#[test]
fn mandatory_cast_addition_only_substitutes_its_own_generic_mana() {
    let spell = Card::from_name("Quick Study").unwrap();
    let _rules = fixture_rules::activate(&[(spell, &FIXED)]);
    for actor in [Player::P0, Player::P1] {
        let mut game = scenario(actor);
        let spell = spawn(&mut game, actor, spell, Zone::Hand);
        let artifact = put(&mut game, actor, "Short Bow", Zone::Battlefield);
        let creature = put(&mut game, actor, "Bear Cub", Zone::Battlefield);
        let artifact_creature = put(&mut game, actor, "Gingerbrute", Zone::Battlefield);
        let land = put(&mut game, actor, "Mountain", Zone::Battlefield);
        game.state.objects[land as usize].tapped = true;
        let foreign = put(&mut game, actor.other(), "Bear Cub", Zone::Battlefield);
        game.state.objects[creature as usize].summoning_sick = true;
        let announcement = Action::AnnounceCast { card: spell };
        game.state.players[actor.index()].mana = [0, 1, 0, 0, 0, 1];
        reject(&mut game, announcement.clone());
        game.state.players[actor.index()].mana[5] = 2;
        act(&mut game, announcement);
        act(&mut game, Action::BeginWaterbend);
        assert!(game.observe(actor.other()).waterbend_choice.is_none());
        for illegal in [land, foreign] {
            let action = waterbender(&game, illegal);
            reject(&mut game, action);
        }
        let stale = Action::ChooseWaterbender {
            object: ObjectRef {
                generation: game.object_ref(creature).generation + 1,
                ..game.object_ref(creature)
            },
        };
        reject(&mut game, stale);
        choose_waterbender(&mut game, artifact);
        assert!(!game.object_view(artifact).tapped);
        let checkpoint = game.snapshot();
        let mut raw = game.clone();
        let mut indexed = game.clone();
        drop(game);
        let choose_creature = waterbender(&raw, creature);
        let index = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == choose_creature)
            .unwrap();
        act(&mut raw, choose_creature);
        indexed
            .apply_index(actor, indexed.revision(), index)
            .unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        let third = waterbender(&raw, artifact_creature);
        reject(&mut raw, third);
        assert_eq!(raw.waterbend_choice().unwrap().remaining.generic, 2);
        pay(&mut raw);
        pay(&mut indexed);
        settle(&mut raw);
        settle(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert!(raw.object_view(artifact).tapped && raw.object_view(creature).tapped);
        assert_eq!(raw.state.players[actor.index()].cards_drawn_this_turn, 1);
        assert_eq!(raw.state.players[actor.index()].mana, [0; 6]);

        indexed.restore(&checkpoint).unwrap();
        choose_waterbender(&mut indexed, artifact_creature);
        pay(&mut indexed);
        settle(&mut indexed);
        assert!(indexed.object_view(artifact_creature).tapped);
        assert!(!indexed.object_view(creature).tapped);
        assert!(!raw.object_view(artifact_creature).tapped);
    }
}

#[test]
fn optional_addition_retains_the_paid_declaration_for_resolution() {
    let spell = Card::from_name("Quick Study").unwrap();
    let _rules = fixture_rules::activate(&[(spell, &OPTIONAL)]);
    let actor = Player::P0;
    let mut game = scenario(actor);
    let spell = spawn(&mut game, actor, spell, Zone::Hand);
    game.state.players[actor.index()].mana = [0, 1, 0, 0, 0, 3];
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, Action::ChooseWaterbendCost { pay: true });
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseWaterbendCost { pay: false });
    pay(&mut declined);
    settle(&mut declined);
    assert_eq!(
        declined.state.players[actor.index()].cards_drawn_this_turn,
        0
    );

    game.state.players[actor.index()].mana[5] = 4;
    act(&mut game, Action::ChooseWaterbendCost { pay: true });
    pay(&mut game);
    let cast = game.state.stack.last().unwrap().cast.unwrap();
    assert_eq!(cast.waterbend, Some(true));
    settle(&mut game);
    assert_eq!(game.state.players[actor.index()].cards_drawn_this_turn, 1);
}

#[test]
fn convoke_and_waterbend_share_one_funding_domain() {
    let spell = Card::from_name("Quick Study").unwrap();
    let _rules = fixture_rules::activate(&[(spell, &WITH_CONVOKE)]);
    let actor = Player::P0;
    let mut game = scenario(actor);
    let spell = spawn(&mut game, actor, spell, Zone::Hand);
    let blue = put(&mut game, actor, "Mystic Archaeologist", Zone::Battlefield);
    let first = put(&mut game, actor, "Short Bow", Zone::Battlefield);
    let second = put(&mut game, actor, "Short Bow", Zone::Battlefield);
    game.state.players[actor.index()].mana = [0, 0, 0, 0, 0, 2];
    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, Action::BeginWaterbend);
    let blue_ref = game.object_ref(blue);
    reject(&mut game, Action::ChooseWaterbender { object: blue_ref });
    act(
        &mut game,
        Action::ChooseConvoker {
            object: blue_ref,
            contribution: ConvokeContribution::Blue,
        },
    );
    choose_waterbender(&mut game, first);
    choose_waterbender(&mut game, second);
    pay(&mut game);
    settle(&mut game);
    for card in [blue, first, second] {
        assert!(game.object_view(card).tapped);
    }
    assert_eq!(game.state.players[actor.index()].cards_drawn_this_turn, 1);
}

#[test]
fn additional_x_respects_target_capacity_and_leaves_mana_value_fixed() {
    let spell = Card::from_name("Quick Study").unwrap();
    let _rules = fixture_rules::activate(&[(spell, &X_CAST)]);
    let actor = Player::P0;
    let mut game = scenario(actor);
    let spell = spawn(&mut game, actor, spell, Zone::Hand);
    let first = put(&mut game, actor, "Bear Cub", Zone::Graveyard);
    let second = put(&mut game, actor.other(), "Bear Cub", Zone::Graveyard);
    let first_helper = put(&mut game, actor, "Short Bow", Zone::Battlefield);
    let second_helper = put(&mut game, actor, "Bear Cub", Zone::Battlefield);
    game.state.players[actor.index()].mana = [0, 1, 0, 0, 0, 2];
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, Action::ChooseX { value: 3 });
    act(&mut game, Action::ChooseX { value: 2 });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(first),
        },
    );
    reject(&mut game, Action::FinishTargets);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(second),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    act(&mut game, Action::BeginWaterbend);
    choose_waterbender(&mut game, first_helper);
    choose_waterbender(&mut game, second_helper);
    pay(&mut game);
    assert_eq!(game.spell_characteristics()[0].mana_value, 3);
    settle(&mut game);
    assert_eq!(game.state.players[actor.index()].cards_drawn_this_turn, 2);
}

#[test]
fn activation_x_enforces_the_printed_minimum_and_reserves_the_tap_symbol() {
    let watcher = Card::from_name("Impact Tremors").unwrap();
    let _rules =
        fixture_rules::activate(&[(Card::GoblinPiker, &X_ACTIVATION), (watcher, &BEND_WATCHER)]);
    let actor = Player::P0;
    let mut game = scenario(actor);
    let source = put(&mut game, actor, "Goblin Piker", Zone::Battlefield);
    spawn(&mut game, actor, watcher, Zone::Battlefield);
    game.state.players[actor.index()].mana = [0; 6];
    reject(&mut game, activate(source));
    let artifact = put(&mut game, actor, "Short Bow", Zone::Battlefield);
    let creature = put(&mut game, actor, "Bear Cub", Zone::Battlefield);
    act(&mut game, activate(source));
    reject(&mut game, Action::ChooseX { value: 0 });
    reject(&mut game, Action::ChooseX { value: 3 });
    act(&mut game, Action::ChooseX { value: 2 });
    act(&mut game, Action::BeginWaterbend);
    let own_tap = waterbender(&game, source);
    reject(&mut game, own_tap);
    choose_waterbender(&mut game, artifact);
    choose_waterbender(&mut game, creature);
    pay(&mut game);
    settle(&mut game);
    assert!(game.object_view(source).tapped);
    assert_eq!(game.state.players[actor.index()].cards_drawn_this_turn, 3);
}

#[test]
fn large_waterbend_x_counts_each_mana_or_waterbend_tap_once_with_a_live_exile_link() {
    let actor = Player::P0;
    let mut game = scenario(actor);
    game.state.players[actor.index()].mana = [0; 6];
    let source = put(
        &mut game,
        actor,
        "Katara, Water Tribe's Hope",
        Zone::Battlefield,
    );
    let iceberg = put(&mut game, actor, "Aang's Iceberg", Zone::Battlefield);
    let exiled = put(&mut game, actor, "Gilded Lotus", Zone::Exile);
    put(&mut game, actor, "Gilded Lotus", Zone::Battlefield);
    game.state.exile_links.push(ExileLink {
        ability_origin: None,
        source: game.object_ref(iceberg),
        exiled: game.object_ref(exiled),
        return_zone: Zone::Battlefield,
    });
    for i in 0..18 {
        let name = if i % 2 == 0 {
            "Island"
        } else {
            "North Pole Gates"
        };
        put(&mut game, actor, name, Zone::Battlefield);
    }
    for _ in 0..4 {
        put(&mut game, actor, "Bear Cub", Zone::Battlefield);
    }
    let tapped = put(&mut game, actor, "Gilded Lotus", Zone::Battlefield);
    game.state.objects[tapped as usize].tapped = true;
    put(&mut game, actor.other(), "Gilded Lotus", Zone::Battlefield);

    act(&mut game, activate(source));
    assert_eq!(
        game.x_choice().unwrap().intervals,
        [XInterval {
            minimum: 1,
            maximum: 26
        }]
    );
    assert!(game.observe(actor.other()).x_choice.is_none());
    reject(&mut game, Action::ChooseX { value: 27 });
    let choose_maximum = Action::ChooseX { value: 26 };
    let checkpoint = game.snapshot();
    let mut indexed = game.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|action| *action == choose_maximum)
        .unwrap();
    indexed
        .apply_index(actor, indexed.revision(), index)
        .unwrap();
    act(&mut game, choose_maximum);
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&checkpoint).unwrap();
    assert_eq!(indexed.snapshot(), checkpoint);
}

#[test]
fn waterbend_x_keeps_a_mana_sacrifice_that_returns_linked_contributors() {
    let land = Card::from_name("North Pole Gates").unwrap();
    let _rules = fixture_rules::activate(&[(land, &SACRIFICE_MANA)]);
    let actor = Player::P1;
    let mut game = scenario(actor);
    game.state.players[actor.index()].mana = [0; 6];
    let katara = put(
        &mut game,
        actor,
        "Katara, Water Tribe's Hope",
        Zone::Battlefield,
    );
    let source = spawn(&mut game, actor, land, Zone::Battlefield);
    for _ in 0..2 {
        let exiled = put(&mut game, actor, "Short Bow", Zone::Exile);
        game.state.exile_links.push(ExileLink {
            ability_origin: None,
            source: game.object_ref(source),
            exiled: game.object_ref(exiled),
            return_zone: Zone::Battlefield,
        });
    }
    act(&mut game, activate(katara));
    assert_eq!(
        game.x_choice().unwrap().intervals,
        [XInterval {
            minimum: 1,
            maximum: 4
        }]
    );
    reject(&mut game, Action::ChooseX { value: 5 });
    act(&mut game, Action::ChooseX { value: 4 });
    choose(
        &mut game,
        |action| matches!(action, Action::ActivateMana { card, .. } if *card == source),
    );
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    choose(&mut game, |action| {
        matches!(action, Action::ChooseEntryTimestamp { .. })
    });
    act(&mut game, Action::BeginWaterbend);
    for _ in 0..3 {
        choose(&mut game, |action| {
            matches!(action, Action::ChooseWaterbender { .. })
        });
    }
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(katara).power, 4);
}

#[test]
fn scalar_x_counts_one_tap_per_land_with_flexible_grants_and_earthbending() {
    let actor = Player::P0;
    let mut game = scenario(actor);
    let animated_land = put(&mut game, actor, "Forest", Zone::Battlefield);
    let lesson = put(&mut game, actor, "Earthbending Lesson", Zone::Hand);
    cast(&mut game, lesson, Some(Target::Card(animated_land)));
    settle(&mut game);
    game.state.players[actor.index()].mana = [0; 6];
    let spell = put(&mut game, actor, "Fated Firepower", Zone::Hand);
    put(&mut game, actor, "Fated Firepower", Zone::Battlefield);
    let guide = put(&mut game, actor, "Great Divide Guide", Zone::Battlefield);
    let audience = put(&mut game, actor, "Raucous Audience", Zone::Battlefield);
    for i in 0..11 {
        put(
            &mut game,
            actor,
            if i < 3 { "Mountain" } else { "Forest" },
            Zone::Battlefield,
        );
    }
    act(&mut game, Action::AnnounceCast { card: spell });
    assert_eq!(
        game.x_choice().unwrap().intervals,
        [XInterval {
            minimum: 0,
            maximum: 12
        }]
    );
    reject(&mut game, Action::ChooseX { value: 13 });
    act(&mut game, Action::ChooseX { value: 12 });
    choose(
        &mut game,
        |action| matches!(action, Action::ActivateMana { card, production: [0, 0, 0, 0, 2, 0], .. } if *card == audience),
    );
    assert!(game.object_view(audience).tapped);
    let grantor = game.object_ref(guide);
    act(
        &mut game,
        Action::TapGrantedMana {
            card: guide,
            grantor,
            production: [0, 0, 0, 1, 0, 0],
        },
    );
    assert!(game.object_view(guide).tapped);
    assert!(game.object_view(animated_land).kinds & 1 != 0);
}

#[test]
fn ward_waterbend_is_paid_by_the_related_spell_controller() {
    let watcher = Card::from_name("Impact Tremors").unwrap();
    let _rules = fixture_rules::activate(&[(Card::GoblinPiker, &WARD), (watcher, &BEND_WATCHER)]);
    for payer in [Player::P0, Player::P1] {
        let mut game = scenario(payer);
        let target = put(&mut game, payer.other(), "Goblin Piker", Zone::Battlefield);
        spawn(&mut game, payer, watcher, Zone::Battlefield);
        spawn(&mut game, payer.other(), watcher, Zone::Battlefield);
        let spell = put(&mut game, payer, "Lightning Strike", Zone::Hand);
        let artifact = put(&mut game, payer, "Short Bow", Zone::Battlefield);
        let creature = put(&mut game, payer, "Bear Cub", Zone::Battlefield);
        let foreign = put(&mut game, payer.other(), "Bear Cub", Zone::Battlefield);
        game.state.players[payer.index()].mana = [0, 0, 0, 1, 0, 1];
        cast(&mut game, spell, Some(Target::Card(target)));
        for _ in 0..8 {
            if matches!(game.state.pending, Pending::ResolutionMana { .. }) {
                break;
            }
            act(&mut game, Action::Pass);
        }
        assert_eq!(game.actor(), Some(payer));
        game.add_quality_mana(
            payer,
            [0, 0, 0, 0, 0, 2],
            Some(Quality {
                restriction: ManaRestrictionView::ArtifactSpell,
                ..Quality::ORDINARY
            }),
        );
        reject(
            &mut game,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 0, 0, 2],
            },
        );
        act(&mut game, Action::BeginWaterbend);
        assert!(game.observe(payer.other()).waterbend_choice.is_none());
        let action = waterbender(&game, foreign);
        reject(&mut game, action);
        choose_waterbender(&mut game, artifact);
        let mut declined = game.clone();
        act(&mut declined, Action::DeclineResolutionMana);
        settle(&mut declined);
        assert_eq!(declined.object_view(target).zone, Zone::Battlefield);
        assert!(!declined.object_view(artifact).tapped);
        assert_eq!(
            declined.state.players[payer.index()].cards_drawn_this_turn,
            0
        );
        reject(&mut game, Action::PayResolutionMana { payment: [0; 6] });
        choose_waterbender(&mut game, creature);
        act(&mut game, Action::PayResolutionMana { payment: [0; 6] });
        settle(&mut game);
        assert_eq!(game.object_view(target).zone, Zone::Graveyard);
        assert!(game.object_view(artifact).tapped && game.object_view(creature).tapped);
        assert_eq!(game.state.players[payer.index()].cards_drawn_this_turn, 1);
        assert_eq!(
            game.state.players[payer.other().index()].cards_drawn_this_turn,
            0
        );
    }
}

#[test]
fn resolution_waterbend_chooses_after_drawing() {
    let spell = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(spell, &RESOLUTION)]);
    let actor = Player::P0;
    let mut game = scenario(actor);
    let spell = spawn(&mut game, actor, spell, Zone::Hand);
    let artifact = put(&mut game, actor, "Short Bow", Zone::Battlefield);
    let mountain = put(&mut game, actor, "Mountain", Zone::Battlefield);
    game.state.players[actor.index()].mana = [0, 1, 0, 0, 0, 0];
    cast(&mut game, spell, None);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.state.players[actor.index()].cards_drawn_this_turn, 3);
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineResolutionMana);
    assert!(
        declined
            .legal_actions()
            .iter()
            .any(|a| matches!(a, Action::Discard { .. }))
    );
    reject(&mut game, Action::BeginWaterbend);
    act(&mut game, Action::TapMana { card: mountain });
    act(&mut game, Action::BeginWaterbend);
    choose_waterbender(&mut game, artifact);
    act(
        &mut game,
        Action::PayResolutionMana {
            payment: [0, 0, 0, 1, 0, 0],
        },
    );
    settle(&mut game);
    assert_eq!(game.observe(actor).players[actor.index()].hand_count, 3);
}

#[test]
fn linked_waterbend_is_exclusive_and_combines_with_printed_additions() {
    let exiled_spell = Card::from_name("Quick Study").unwrap();
    let watcher = Card::from_name("Impact Tremors").unwrap();
    let _rules = fixture_rules::activate(&[
        (Card::GoblinPiker, &LINKED),
        (exiled_spell, &OPTIONAL),
        (watcher, &BEND_WATCHER),
    ]);
    let actor = Player::P0;
    let mut game = scenario(actor);
    spawn(&mut game, actor, watcher, Zone::Battlefield);
    let source = put(&mut game, actor, "Goblin Piker", Zone::Hand);
    let selected = spawn(&mut game, actor.other(), exiled_spell, Zone::Graveyard);
    let creature = put(&mut game, actor.other(), "Bear Cub", Zone::Graveyard);
    let foreign = spawn(&mut game, actor, exiled_spell, Zone::Graveyard);
    cast(&mut game, source, None);
    for _ in 0..6 {
        if matches!(game.state.pending, Pending::Targets { .. }) {
            break;
        }
        act(&mut game, Action::Pass);
    }
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(actor.other()),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    for illegal in [creature, foreign] {
        let object = game.object_ref(illegal);
        reject(&mut game, Action::SelectResolutionExile { object });
    }
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineEffectPayment);
    settle(&mut declined);
    assert_eq!(declined.object_view(selected).zone, Zone::Graveyard);
    let object = game.object_ref(selected);
    act(&mut game, Action::SelectResolutionExile { object });
    settle(&mut game);
    assert_eq!(game.object_view(selected).zone, Zone::Exile);
    let source = game.object_ref(source);
    let object = game.object_ref(selected);
    let announcement = Action::AnnounceLinkedWaterbend {
        object,
        source,
        face: CastFace::Normal,
        flash_surcharge: false,
    };
    assert!(game.legal_actions().contains(&announcement));
    reject(&mut game, Action::AnnounceExile { card: selected });
    let mut lost_control = game.clone();
    lost_control.change_controller(source, actor.other());
    lost_control.change_controller(source, actor);
    reject(&mut lost_control, announcement.clone());

    game.state.players[actor.index()].mana = [0, 0, 0, 0, 0, 5];
    act(&mut game, announcement);
    act(&mut game, Action::ChooseWaterbendCost { pay: true });
    pay(&mut game);
    let cast = game.state.stack.iter().find_map(|item| item.cast).unwrap();
    assert_eq!(cast.method, CastMethod::LinkedExileWaterbend);
    assert_eq!(cast.payment, [0, 0, 0, 0, 0, 5]);
    assert_eq!(game.spell_characteristics()[0].mana_value, 3);
    choose(&mut game, |action| {
        matches!(action, Action::PutTrigger { .. })
    });
    settle(&mut game);
    assert_eq!(game.state.players[actor.index()].cards_drawn_this_turn, 3);
    assert_eq!(game.object_view(selected).zone, Zone::Graveyard);
}

#[test]
fn zero_and_mana_only_waterbend_payments_trigger_once_and_copies_keep_the_receipt() {
    let variable = Card::from_name("Quick Study").unwrap();
    let optional = Card::from_name("Opt").unwrap();
    let watcher = Card::from_name("Impact Tremors").unwrap();
    let copier = Card::from_name("Lightning Strike").unwrap();
    let _rules = fixture_rules::activate(&[
        (variable, &X_CAST),
        (optional, &OPTIONAL),
        (watcher, &BEND_WATCHER),
        (copier, &COPY),
    ]);
    for actor in [Player::P0, Player::P1] {
        let mut game = scenario(actor);
        spawn(&mut game, actor, watcher, Zone::Battlefield);
        let spell = spawn(&mut game, actor, variable, Zone::Hand);
        game.state.players[actor.index()].mana = [0, 1, 0, 0, 0, 2];
        act(&mut game, Action::AnnounceCast { card: spell });
        act(&mut game, Action::ChooseX { value: 0 });
        if game.legal_actions().contains(&Action::FinishTargets) {
            act(&mut game, Action::FinishTargets);
        }
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.state.players[actor.index()].cards_drawn_this_turn, 1);

        let spell = spawn(&mut game, actor, optional, Zone::Hand);
        game.state.players[actor.index()].mana = [0, 1, 0, 0, 0, 2];
        act(&mut game, Action::AnnounceCast { card: spell });
        let mut declined = game.clone();
        act(&mut declined, Action::ChooseWaterbendCost { pay: false });
        pay(&mut declined);
        settle(&mut declined);
        assert_eq!(
            declined.state.players[actor.index()].cards_drawn_this_turn,
            1
        );

        act(&mut game, Action::ChooseWaterbendCost { pay: true });
        pay(&mut game);
        let copier = spawn(&mut game, actor, copier, Zone::Hand);
        game.state.players[actor.index()].mana = [0, 0, 0, 1, 0, 1];
        cast(&mut game, copier, Some(Target::Spell(spell)));
        settle(&mut game);
        assert_eq!(game.state.players[actor.index()].cards_drawn_this_turn, 4);
    }
}

#[test]
fn waterbend_x_sets_the_resolving_group_base_stats_and_expires_at_cleanup() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &X_STATS_ACTIVATION)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let recipient = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let opponent = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    game.add_counter(recipient, CounterKind::PlusOne, 1);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    reject(&mut game, Action::ChooseX { value: 0 });
    act(&mut game, Action::ChooseX { value: 4 });
    pay(&mut game);
    let before_resolution = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    settle(&mut game);
    assert_eq!(stats(&game, source), (4, 4));
    assert_eq!(stats(&game, recipient), (5, 5));
    assert_eq!(stats(&game, before_resolution), (4, 4));
    assert_eq!(stats(&game, opponent), (2, 2));
    let late = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    assert_eq!(stats(&game, late), (2, 2));
    game.finish_cleanup();
    assert_eq!(stats(&game, source), (3, 3));
    assert_eq!(stats(&game, recipient), (3, 3));
}
