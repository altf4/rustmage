//! A melded permanent is one game object represented by two physical cards.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const PAIR: MeldPair = MeldPair {
    source: Card::from_catalogue_id(4846),
    partner: Card::from_catalogue_id(1443),
    combined: Card::from_catalogue_id(3344),
};

const SOURCE: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Main1,
            relation: CastRelation::You,
            guard: StepGuard::OwnSourceAndNamedCreature(PAIR.partner),
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayMana {
                cost: ManaCost {
                    colored: [0, 0, 1, 0, 1, 0],
                    generic: 3,
                },
                skip: 1,
            },
            Instruction::Meld(PAIR),
        ],
    }],
    ..CardRules::EMPTY
};
const PARTNER: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    ..CardRules::EMPTY
};
const COMBINED: CardRules = CardRules {
    effect: Effect::Creature(7, 6),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    extras: &CardExtras {
        meld_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const fn one_creature(program: &'static [Instruction]) -> CardRules {
    CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program,
        ),
        ..CardRules::EMPTY
    }
}

const COPY: CardRules = one_creature(&[Instruction::CreateCopyToken {
    subject: Subject::Targets,
    amount: 1,
}]);
const DESTROY: CardRules = one_creature(&[Instruction::Destroy]);
const TOP: CardRules = one_creature(&[Instruction::TopLibrary]);

fn begin_first_main(game: &mut Game) {
    game.state.step = Step::Draw;
    act(game, Action::Pass);
    act(game, Action::Pass);
    assert_eq!(game.state.step, Step::Main1);
    assert_eq!(game.state.stack.len(), 1);
}

fn pay_meld(game: &mut Game) {
    let player = game.state.active;
    let lands = ["Swamp", "Forest", "Forest", "Forest", "Forest"]
        .map(|name| put(game, player, name, Zone::Battlefield));
    act(game, Action::Pass);
    act(game, Action::Pass);
    for card in lands {
        act(game, Action::TapMana { card });
    }
    choose(game, |action| {
        matches!(action, Action::PayResolutionMana { .. })
    });
}

fn setup_pair(player: Player) -> (Game, CardId, CardId) {
    let mut game = scenario(player);
    let source = spawn(&mut game, player, PAIR.source, Zone::Battlefield);
    let partner = spawn(&mut game, player, PAIR.partner, Zone::Battlefield);
    game.state.objects[partner as usize].tapped = true;
    game.state.objects[partner as usize].counters.plus_one = 1;
    begin_first_main(&mut game);
    pay_meld(&mut game);
    (game, source, partner)
}

fn choose_partner(game: &mut Game, partner: CardId) {
    let object = game.object_ref(partner);
    act(game, Action::ChooseEffectObject { object });
    settle(game);
}

fn resolve_targeting_spell(game: &mut Game, player: Player, name: &str, target: CardId) {
    game.state.players[player.index()].mana = [8; 6];
    let spell = put(game, player, name, Zone::Hand);
    cast(game, spell, Some(Target::Card(target)));
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn meld_enters_once_and_copies_use_the_combined_face_without_the_physical_pair() {
    let _rules = fixture_rules::activate(&[
        (PAIR.source, &SOURCE),
        (PAIR.partner, &PARTNER),
        (PAIR.combined, &COMBINED),
        (Card::from_name("Think Twice").unwrap(), &COPY),
    ]);
    let player = Player::P1;
    let (mut game, source, partner) = setup_pair(player);
    let selected = Action::ChooseEffectObject {
        object: game.object_ref(partner),
    };
    let before = game.snapshot();
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == selected)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, revision, index).unwrap();
    act(&mut game, selected);
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&before).unwrap();
    assert_eq!(
        indexed.apply_index(player, revision, index),
        Err(Error::StaleRevision)
    );
    choose_partner(&mut indexed, partner);
    assert_eq!(game.snapshot(), indexed.snapshot());

    let view = game.object_view(source);
    assert_eq!(view.card, PAIR.combined);
    assert_eq!(view.zone, Zone::Battlefield);
    assert_eq!((view.power, view.toughness), (7, 6));
    assert!(!view.tapped);
    assert_eq!(game.object_mana_value(source), 7);
    assert!(
        !game
            .observe(player)
            .objects
            .iter()
            .any(|object| object.id == partner)
    );
    assert_eq!(
        game.observe(player.other()).melded_permanents[0].front_cards,
        [PAIR.source, PAIR.partner]
    );

    act(&mut game, Action::Pass);
    resolve_targeting_spell(&mut game, player.other(), "Think Twice", source);
    settle(&mut game);
    let copy = game
        .observe(player.other())
        .objects
        .into_iter()
        .find(|object| {
            object.card == PAIR.combined && object.id != source && object.zone == Zone::Battlefield
        })
        .unwrap();
    assert!(copy.is_token());
    assert_eq!(game.object_mana_value(copy.id), 0);
    assert_eq!(game.observe(player).melded_permanents.len(), 1);
}

#[test]
fn graveyard_order_and_library_privacy_follow_the_actual_two_fronts() {
    let _rules = fixture_rules::activate(&[
        (PAIR.source, &SOURCE),
        (PAIR.partner, &PARTNER),
        (PAIR.combined, &COMBINED),
        (Card::from_name("Opt").unwrap(), &DESTROY),
        (Card::from_name("Think Twice").unwrap(), &TOP),
    ]);
    let player = Player::P0;
    let (mut original, source, partner) = setup_pair(player);
    choose_partner(&mut original, partner);
    for (spell, zone) in [("Opt", Zone::Graveyard), ("Think Twice", Zone::Library)] {
        let mut game = original.clone();
        resolve_targeting_spell(&mut game, player, spell, source);
        assert_eq!(game.state.pending, Pending::MeldOrdering { player });
        assert_eq!(game.object_view(source).card, PAIR.combined);
        assert!(game.observe(player.other()).meld_ordering_choice.is_none());
        let order = game.observe(player).meld_ordering_choice.unwrap();
        assert_eq!(order.destination, zone);
        let selected = Action::ChooseMeldOrder {
            first: order.components[1],
        };
        let saved = game.snapshot();
        let mut alternative = game.clone();
        act(&mut game, selected);
        settle(&mut game);
        assert_eq!(alternative.snapshot(), saved);
        act(
            &mut alternative,
            Action::ChooseMeldOrder {
                first: order.components[0],
            },
        );
        settle(&mut alternative);
        for branch in [&game, &alternative] {
            assert_eq!(branch.object_view(source).card, PAIR.source);
            assert_eq!(branch.object_view(partner).card, PAIR.partner);
            assert_eq!(branch.object_view(source).zone, zone);
            assert_eq!(branch.object_view(partner).zone, zone);
            assert!(branch.observe(player).melded_permanents.is_empty());
        }
        if zone == Zone::Graveyard {
            assert_eq!(game.hand_count(player), 1);
            assert_eq!(
                game.observe(player.other()).meld_departure_orders[0].cards,
                [game.object_ref(partner), game.object_ref(source)]
            );
        } else {
            assert_eq!(game.top_library_card(player), Some(partner));
            assert_eq!(alternative.top_library_card(player), Some(source));
            assert!(game.observe(player.other()).known_library_top[player.index()].is_none());
            assert_eq!(
                game.observe(player.other()).meld_departure_orders,
                alternative.observe(player.other()).meld_departure_orders
            );
        }
    }
}

#[test]
fn projected_meld_departure_restores_two_exact_fronts_after_one_zone_plan() {
    use rustmage_core::engine::ability_world::CharacteristicWorld;
    use rustmage_core::engine::funding::{FundingGoal, FundingState};

    const DARKNESS: CardRules = CardRules {
        effect: Effect::Permanent,
        extras: &CardExtras {
            zone_replacement: Some(
                ZoneReplacementRule::ExileOpposingNontokenCreatureDeathLinked { life: 2 },
            ),
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let darkness = Card::from_name("Soul-Guide Lantern").unwrap();
    let _rules = fixture_rules::activate(&[
        (PAIR.source, &SOURCE),
        (PAIR.partner, &PARTNER),
        (PAIR.combined, &COMBINED),
        (darkness, &DARKNESS),
    ]);
    let owner = Player::P0;
    let (mut game, source, partner) = setup_pair(owner);
    choose_partner(&mut game, partner);
    let permanent = game.object_ref(source);
    let component = game.object_ref(partner);
    let goal = FundingGoal {
        actor: owner,
        required: [0; 6],
        generic: 0,
        reserve: Some((
            permanent,
            AdditionalCost {
                kind: EffectChoiceKind::Sacrifice,
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
            },
        )),
        source: None,
    };

    let before = game.snapshot();
    let world = FundingState::from_game(&game, &goal);
    assert_eq!(world.object(partner).zone, Zone::Outside);
    assert!(!world.cost_departures(&[component], Zone::Hand, &mut |_| {
        panic!("a parked meld component cannot be selected for a cost")
    }));
    for zone in [Zone::Hand, Zone::Library, Zone::Exile, Zone::Graveyard] {
        assert!(world.cost_departures(&[permanent], zone, &mut |next| {
            let key = next.key(&goal);
            for (old, front) in [(permanent, PAIR.source), (component, PAIR.partner)] {
                let object = next.object(old.card);
                assert_eq!(
                    (object.zone, object.card, object.controller),
                    (zone, front, owner)
                );
                assert_eq!(next.reference(old.card).generation, old.generation + 1);
                assert!(key.resources.iter().any(|resource| {
                    resource.identity == Some((old.card, old.generation + 1))
                        && resource.definition == front.index()
                }));
            }
            let died = u64::from(zone == Zone::Graveyard);
            assert_eq!(next.creatures_died(), world.creatures_died() + died);
            assert_eq!(next.controlled_creatures_died[owner.index()], died);
            true
        }));
    }
    assert_eq!(game.snapshot(), before);

    // The source sees the opponent's controller even though both cards
    // belong to the same player as the replacement source.
    game.record_control(permanent, owner.other(), None);
    spawn(&mut game, owner, darkness, Zone::Battlefield);
    put(&mut game, owner, "Rest in Peace", Zone::Battlefield);
    let before = game.snapshot();
    let goal = FundingGoal {
        actor: owner.other(),
        ..goal
    };
    let world = FundingState::from_game(&game, &goal);
    let mut gains = Vec::new();
    world.cost_departures(&[permanent], Zone::Graveyard, &mut |next| {
        for (id, front) in [(source, PAIR.source), (partner, PAIR.partner)] {
            let object = next.object(id);
            assert_eq!(
                (object.zone, object.card, object.controller),
                (Zone::Exile, front, owner)
            );
        }
        assert_eq!(next.creatures_died(), world.creatures_died());
        gains.push(next.life(owner) - world.life(owner));
        false
    });
    assert_eq!(gains, [0, 2]);
    assert_eq!(game.snapshot(), before);
}

#[test]
fn casting_can_be_funded_by_both_fronts_after_meld_sacrifice() {
    const BLUE: Mana = [0, 1, 0, 0, 0, 0];
    const COLORLESS: Mana = [0, 0, 0, 0, 0, 1];
    const GRAVE_MANA: ActivatedAbility = ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Graveyard,
        cost: ActivationCost {
            exile_self: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed(BLUE))],
        sorcery: false,
        once_per_turn: false,
    };
    const VANILLE: CardRules = CardRules {
        activated: &[GRAVE_MANA],
        ..SOURCE
    };
    const FANG: CardRules = CardRules {
        activated: &[GRAVE_MANA],
        ..PARTNER
    };
    const RAGNAROK: CardRules = CardRules {
        activated: &[ActivatedAbility {
            origin: ActivationOrigin::Battlefield,
            cost: ActivationCost {
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Fixed(COLORLESS))],
            ..GRAVE_MANA
        }],
        triggers: &[],
        ..COMBINED
    };
    let _rules = fixture_rules::activate(&[
        (PAIR.source, &VANILLE),
        (PAIR.partner, &FANG),
        (PAIR.combined, &RAGNAROK),
    ]);
    let player = Player::P1;
    let (mut game, source, partner) = setup_pair(player);
    choose_partner(&mut game, partner);
    game.state.players[player.index()].mana = [0; 6];
    let spell = put(&mut game, player, "Divination", Zone::Hand);
    let cast = Action::AnnounceCast { card: spell };
    let before = game.snapshot();
    assert!(game.legal_actions().contains(&cast));
    let mut replaced = game.clone();
    put(
        &mut replaced,
        player.other(),
        "Rest in Peace",
        Zone::Battlefield,
    );
    reject(&mut replaced, cast.clone());
    assert_eq!(game.snapshot(), before);

    act(&mut game, cast);
    let activate = |card, production| Action::ActivateMana {
        card,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    };
    act(&mut game, activate(source, COLORLESS));
    assert_eq!(game.state.pending, Pending::MeldOrdering { player });
    let choice = game.observe(player).meld_ordering_choice.unwrap();
    assert_eq!(choice.destination, Zone::Graveyard);
    let order = Action::ChooseMeldOrder {
        first: choice.components[1],
    };
    let saved = game.snapshot();
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == order)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, revision, index).unwrap();
    act(&mut game, order);
    assert_eq!(game.snapshot(), indexed.snapshot());

    act(&mut game, activate(source, BLUE));
    act(&mut game, activate(partner, BLUE));
    pay(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Stack);
    assert_eq!(game.object_view(source).zone, Zone::Exile);
    assert_eq!(game.object_view(partner).zone, Zone::Exile);
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(player, revision, index),
        Err(Error::StaleRevision)
    );
    assert_eq!(indexed.object_view(source).card, PAIR.combined);
    assert_eq!(indexed.object_view(partner).zone, Zone::Outside);
}

#[test]
fn projected_meld_entry_parks_the_exact_exiled_companion() {
    use rustmage_core::engine::ability_world::CharacteristicWorld;
    use rustmage_core::engine::funding::{FundingGoal, FundingState};
    use rustmage_core::engine::meld::MeldEntry;

    let _rules = fixture_rules::activate(&[
        (PAIR.source, &SOURCE),
        (PAIR.partner, &PARTNER),
        (PAIR.combined, &COMBINED),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = spawn(&mut game, player, PAIR.source, Zone::Exile);
    let partner = spawn(&mut game, player, PAIR.partner, Zone::Exile);
    let component = game.object_ref(partner);
    let mut movement = game.entry_move(source, Zone::Battlefield, false);
    movement.project_printed_face(PAIR.combined);
    movement.meld = Some(Box::new(MeldEntry {
        companion: component,
        front_cards: [PAIR.source, PAIR.partner],
        combined: PAIR.combined,
    }));
    let goal = FundingGoal {
        actor: player,
        required: [0; 6],
        generic: 0,
        reserve: None,
        source: None,
    };
    let before = game.snapshot();
    let world = FundingState::from_game(&game, &goal);
    assert!(
        world.commit_projected_entries(vec![movement], &[], &mut |melded| {
            assert_eq!(melded.object(source).card, PAIR.combined);
            assert_eq!(melded.mana_value(source), 7);
            assert_eq!(melded.object(partner).zone, Zone::Outside);
            assert_eq!(
                melded.reference(partner).generation,
                component.generation + 1
            );
            melded.cost_departures(&[melded.reference(source)], Zone::Graveyard, &mut |next| {
                assert_eq!(next.object(partner).zone, Zone::Graveyard);
                assert_eq!(next.object(partner).card, PAIR.partner);
                assert_eq!(next.reference(partner).generation, component.generation + 2);
                true
            })
        })
    );
    assert_eq!(game.snapshot(), before);
}

#[test]
fn stolen_partners_fail_the_intervening_condition_and_a_named_copy_cannot_meld() {
    let _rules = fixture_rules::activate(&[
        (PAIR.source, &SOURCE),
        (PAIR.partner, &PARTNER),
        (PAIR.combined, &COMBINED),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = spawn(&mut game, player, PAIR.source, Zone::Battlefield);
    let partner = spawn(&mut game, player, PAIR.partner, Zone::Battlefield);
    begin_first_main(&mut game);
    game.record_control(game.object_ref(partner), player.other(), None);
    settle(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Battlefield);
    assert!(game.observe(player).melded_permanents.is_empty());

    let mut game = scenario(player);
    let source = spawn(&mut game, player, PAIR.source, Zone::Battlefield);
    let copy = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    game.apply_temporary_copy(
        game.object_ref(copy),
        CopyRecipe::new(PAIR.partner),
        CopyDeadline::EndOfTurn,
    );
    begin_first_main(&mut game);
    pay_meld(&mut game);
    choose_partner(&mut game, copy);
    assert_eq!(game.object_view(source).zone, Zone::Exile);
    assert_eq!(game.object_view(copy).zone, Zone::Exile);
    assert_eq!(game.object_view(copy).card, Card::GrizzlyBears);
    assert!(game.observe(player).melded_permanents.is_empty());
}

const BLINK: CardRules =
    one_creature(&[Instruction::BlinkFollowingPublicMove { all_groups: false }]);
const COMBINED_RETURNS: CardRules = CardRules {
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::ReturnEventCardToHand],
    }],
    ..COMBINED
};

#[test]
fn one_blink_returns_both_fronts_and_owned_meld_order_replays() {
    let _rules = fixture_rules::activate(&[
        (PAIR.source, &SOURCE),
        (PAIR.partner, &PARTNER),
        (PAIR.combined, &COMBINED),
        (Card::from_name("Opt").unwrap(), &BLINK),
    ]);
    let player = Player::P0;
    let (mut game, source, partner) = setup_pair(player);
    choose_partner(&mut game, partner);
    let permanent = game.object_ref(source);
    resolve_targeting_spell(&mut game, player, "Opt", source);
    let order = game.observe(player).meld_ordering_choice.unwrap();
    let action = Action::ChooseMeldOrder {
        first: order.components[1],
    };
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|candidate| *candidate == action)
        .unwrap();
    let saved = game.snapshot();
    let mut indexed = game.clone();
    act(&mut game, action);
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(player, revision, index),
        Err(Error::StaleRevision)
    );
    settle(&mut game);
    for (card, face) in [(source, PAIR.source), (partner, PAIR.partner)] {
        assert_eq!(game.object_view(card).zone, Zone::Battlefield);
        assert_eq!(game.object_view(card).card, face);
    }
    assert_ne!(game.object_ref(source), permanent);
    assert!(game.observe(player).melded_permanents.is_empty());
}

#[test]
fn death_returns_each_exact_card_and_one_replacement_redirects_both() {
    let _rules = fixture_rules::activate(&[
        (PAIR.source, &SOURCE),
        (PAIR.partner, &PARTNER),
        (PAIR.combined, &COMBINED_RETURNS),
        (Card::from_name("Opt").unwrap(), &DESTROY),
    ]);
    let player = Player::P0;
    let (mut game, source, partner) = setup_pair(player);
    choose_partner(&mut game, partner);
    for (stale, replaced) in [(false, false), (true, false), (false, true)] {
        let mut branch = game.clone();
        if replaced {
            put(
                &mut branch,
                player.other(),
                "Rest in Peace",
                Zone::Battlefield,
            );
        }
        resolve_targeting_spell(&mut branch, player, "Opt", source);
        let order = branch.observe(player).meld_ordering_choice.unwrap();
        assert_eq!(
            order.destination,
            if replaced {
                Zone::Exile
            } else {
                Zone::Graveyard
            }
        );
        act(
            &mut branch,
            Action::ChooseMeldOrder {
                first: order.components[0],
            },
        );
        if stale {
            branch.move_object(partner, Zone::Exile);
            branch.move_object(partner, Zone::Graveyard);
        }
        settle(&mut branch);
        assert_eq!(
            branch.object_view(source).zone,
            if replaced { Zone::Exile } else { Zone::Hand }
        );
        let partner_zone = match (stale, replaced) {
            (_, true) => Zone::Exile,
            (true, false) => Zone::Graveyard,
            (false, false) => Zone::Hand,
        };
        assert_eq!(branch.object_view(partner).zone, partner_zone);
    }
}
