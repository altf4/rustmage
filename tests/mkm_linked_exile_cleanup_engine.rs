//! Private linked exile, event-player permissions, and temporary token cleanup.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::engine::temporary_tokens::TokenCleanup;
use rustmage_core::{cards::*, engine::*};
use support::*;

const BLIND_EXILE: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledCreatureAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileControllerLibraryTopLinkedFaceDown(1)],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 1,
            colored: [0, 0, 0, 1, 0, 0],
            mana_component: true,
            sacrifice: true,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::DiscardHand,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 0,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
            }),
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ReturnLinkedExileToHand],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

const DAMAGE_PLAY: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureDamaged {
            relation: CastRelation::Any,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayForPlayer {
                player: PlayerSet::EventObjectController,
                skip: 1,
            },
            Instruction::ExileTopPlayForPlayer {
                player: PlayerSet::EventObjectController,
                amount: AmountValue::DamageReceivedAmount,
                next_turn: true,
            },
        ],
    }],
    ..CardRules::EMPTY
};

const THOPTER: Card = Card::from_catalogue_id(5266);
const TEMPORARY_TOKENS: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTemporaryTokensThenCleanup {
            token: THOPTER,
            amount: AmountValue::Fixed(1),
            keywords: Keywords::HASTE,
            cleanup: TokenCleanup::OwnEndStepSacrifice,
        }],
    }],
    ..CardRules::EMPTY
};
const END_TURN: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[Instruction::EndTurn]),
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn activate(game: &mut Game, source: CardId) {
    choose(game, |action| {
        matches!(
            action,
            Action::Activate { card, ability: 0, .. }
                | Action::AnnounceActivate { card, ability: 0, .. }
                if *card == source
        )
    });
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

fn advance_until(game: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..80 {
        if ready(game) {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass | Action::FinishAttackers | Action::FinishBlockers
            )
        });
    }
    panic!("scenario did not reach its stop condition");
}

#[test]
fn face_down_library_exiles_are_private_and_linked_to_exact_source_incarnations() {
    let _rules = fixture_rules::activate(&[(named("Soul-Guide Lantern"), &BLIND_EXILE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let first = put(&mut game, player, "Soul-Guide Lantern", Zone::Battlefield);
    let second = put(&mut game, player, "Soul-Guide Lantern", Zone::Battlefield);
    let attackers = [
        put(&mut game, player, "Bear Cub", Zone::Battlefield),
        put(&mut game, player, "Grizzly Bears", Zone::Battlefield),
    ];
    let first_cards = [
        put(&mut game, player, "Swamp", Zone::Library),
        put(&mut game, player, "Mountain", Zone::Library),
    ];
    let second_cards = [
        put(&mut game, player, "Think Twice", Zone::Library),
        put(&mut game, player, "Island", Zone::Library),
    ];

    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    for card in attackers {
        act(&mut game, Action::Attack { card });
    }
    act(&mut game, Action::FinishAttackers);
    for _ in 0..2 {
        let id = game
            .state
            .ordering_triggers
            .iter()
            .find(|item| item.source.is_some_and(|source| source.id == first))
            .unwrap()
            .id;
        act(&mut game, Action::PutTrigger { id });
    }
    choose(&mut game, |action| {
        matches!(action, Action::PutTrigger { .. })
    });
    assert_eq!(game.state.stack.len(), 4);
    settle(&mut game);

    for id in first_cards.into_iter().chain(second_cards) {
        assert_eq!(game.object_view(id).zone, Zone::Exile);
        let object = game.object_ref(id);
        for viewer in [player, player.other()] {
            let observation = game.observe(viewer);
            assert!(observation.objects.iter().all(|row| row.id != id));
            assert!(
                observation
                    .linked_exile
                    .iter()
                    .all(|link| link.exiled != object)
            );
            assert!(
                observation
                    .exile_permissions
                    .iter()
                    .all(|row| row.object != object)
            );
        }
    }

    let mut empty_hand = game.clone();
    activate(&mut empty_hand, first);
    settle(&mut empty_hand);
    for id in first_cards {
        assert_eq!(empty_hand.object_view(id).zone, Zone::Hand);
    }

    let discard = put(&mut game, player, "Forest", Zone::Hand);
    activate(&mut game, first);
    settle(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Graveyard);
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    for id in first_cards {
        assert_eq!(game.object_view(id).zone, Zone::Hand);
        assert!(
            game.observe(player.other())
                .objects
                .iter()
                .all(|row| row.id != id)
        );
    }

    game.move_object(second, Zone::Exile);
    game.move_object(second, Zone::Battlefield);
    activate(&mut game, second);
    settle(&mut game);
    for id in second_cards {
        assert_eq!(game.object_view(id).zone, Zone::Exile);
        assert!(game.exiled_face_down(game.object_ref(id)));
    }
}

#[test]
fn actual_damage_gives_its_recipient_controller_normal_plays_until_their_next_cleanup() {
    let _rules = fixture_rules::activate(&[(named("Soul-Guide Lantern"), &DAMAGE_PLAY)]);
    let player = Player::P0;
    let beneficiary = player.other();
    let mut game = scenario(player);
    put(&mut game, player, "Soul-Guide Lantern", Zone::Battlefield);
    let recipient = put(&mut game, beneficiary, "Bear Cub", Zone::Battlefield);
    let spell = put(&mut game, beneficiary, "Think Twice", Zone::Library);
    let land = put(&mut game, beneficiary, "Island", Zone::Library);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Card(recipient)));
    pass_pair(&mut game);
    assert_eq!(game.object_view(recipient).zone, Zone::Graveyard);
    assert_eq!(game.state.stack.len(), 1);
    assert_eq!(
        game.state.stack[0].damage_received.as_ref().unwrap().amount,
        2
    );
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(beneficiary));

    let checkpoint = game.snapshot();
    let mut decline = game.clone();
    act(&mut decline, Action::ChooseOptional { accept: false });
    for id in [land, spell] {
        assert_eq!(decline.object_view(id).zone, Zone::Library);
    }
    let accept = Action::ChooseOptional { accept: true };
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == accept)
        .unwrap();
    let mut indexed = game.clone();
    indexed
        .apply_index(beneficiary, game.revision(), index)
        .unwrap();
    act(&mut game, accept);
    assert_eq!(indexed.snapshot(), game.snapshot());
    indexed.restore(&checkpoint).unwrap();
    assert_eq!(indexed.actor(), Some(beneficiary));
    act(&mut indexed, Action::ChooseOptional { accept: false });
    assert_eq!(indexed.snapshot(), decline.snapshot());

    for id in [land, spell] {
        let object = game.object_ref(id);
        assert_eq!(game.object_view(id).zone, Zone::Exile);
        assert!(game.observe(player).exile_permissions.iter().any(|grant| {
            grant.object == object
                && grant.player == beneficiary
                && grant.expiry == ExilePlayExpiry::EndOfNextTurn
                && !grant.cast_only
                && !grant.spend_any_mana
        }));
    }
    reject(&mut game, Action::PlayLand { card: land });
    act(&mut game, Action::Pass);
    reject(&mut game, Action::PlayLand { card: land });
    game.state.players[beneficiary.index()].mana = [0, 0, 0, 4, 0, 0];
    reject(&mut game, Action::AnnounceExile { card: spell });

    let first_turn = game.state.turn;
    advance_until(&mut game, |g| {
        g.state.turn == first_turn + 1 && g.state.step == Step::Main1
    });
    assert_eq!(game.actor(), Some(beneficiary));
    let mut plays = game.clone();
    act(&mut plays, Action::PlayLand { card: land });
    assert_eq!(plays.object_view(land).zone, Zone::Battlefield);
    plays.state.players[beneficiary.index()].mana = [0, 2, 0, 0, 0, 0];
    act(&mut plays, Action::AnnounceExile { card: spell });
    pay(&mut plays);
    let cast = plays.state.stack.last().unwrap().cast.unwrap();
    assert_eq!(cast.origin, Zone::Exile);
    assert_eq!(cast.caster, beneficiary);
    assert_eq!(cast.method, CastMethod::Normal);
    assert_eq!(cast.payment, [0, 2, 0, 0, 0, 0]);
    settle(&mut plays);
    assert_eq!(plays.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(plays.state.players[beneficiary.index()].mana[1], 0);

    let exact = game.object_ref(spell);
    game.move_object(spell, Zone::Graveyard);
    game.move_object(spell, Zone::Exile);
    assert_ne!(game.object_ref(spell), exact);
    game.state.players[beneficiary.index()].mana = [0, 3, 0, 0, 0, 0];
    reject(&mut game, Action::AnnounceExile { card: spell });
    advance_until(&mut game, |g| g.state.step == Step::End);
    let exact_land = game.object_ref(land);
    assert!(
        game.observe(beneficiary)
            .exile_permissions
            .iter()
            .any(|grant| {
                grant.object == exact_land && grant.expiry == ExilePlayExpiry::Turn(first_turn + 1)
            })
    );
    advance_until(&mut game, |g| g.state.turn == first_turn + 2);
    assert_eq!(game.object_view(land).zone, Zone::Exile);
    assert!(
        game.observe(beneficiary)
            .exile_permissions
            .iter()
            .all(|grant| grant.object.card != land)
    );
}

#[test]
fn damage_offer_uses_current_or_last_known_controller_before_resolution() {
    let _rules = fixture_rules::activate(&[(named("Soul-Guide Lantern"), &DAMAGE_PLAY)]);
    let player = Player::P1;
    let original_controller = player.other();
    for leaves in [false, true] {
        let mut game = scenario(player);
        put(&mut game, player, "Soul-Guide Lantern", Zone::Battlefield);
        let recipient = put(
            &mut game,
            original_controller,
            "Hill Giant",
            Zone::Battlefield,
        );
        let shock = put(&mut game, player, "Shock", Zone::Hand);
        cast(&mut game, shock, Some(Target::Card(recipient)));
        pass_pair(&mut game);
        assert_eq!(game.state.stack.len(), 1);
        assert!(game.on_battlefield(recipient));
        let exact = game.object_ref(recipient);
        game.record_control(exact, player, None);
        if leaves {
            game.move_object(recipient, Zone::Graveyard);
            assert_ne!(game.object_ref(recipient), exact);
        }
        pass_pair(&mut game);
        assert_eq!(game.actor(), Some(player));
        act(&mut game, Action::ChooseOptional { accept: false });
        assert_eq!(game.state.pending, Pending::Priority);
    }
}

#[test]
fn haste_expires_at_cleanup_and_delayed_sacrifice_waits_for_the_creators_end_step() {
    let _rules = fixture_rules::activate(&[
        (named("Hill Giant"), &TEMPORARY_TOKENS),
        (named("Opt"), &END_TURN),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let source_ref = game.object_ref(source);
    put(&mut game, player, "Doubling Season", Zone::Battlefield);
    game.begin_step(Step::BeginCombat);
    game.prepare_triggers();
    settle(&mut game);
    let tokens: Vec<_> = game
        .controlled_token_ids(player)
        .filter(|&id| game.object_view(id).card == THOPTER)
        .collect();
    assert_eq!(tokens.len(), 2);
    let token_refs: Vec<_> = tokens.iter().map(|&id| game.object_ref(id)).collect();
    let delays = game.observe(player).delayed_sacrifices;
    assert_eq!(delays.len(), 1);
    assert_eq!(delays[0].tokens, token_refs);
    assert_eq!(delays[0].source, source_ref);
    assert!(delays[0].own_end_step);
    for &id in &tokens {
        assert!(
            game.keywords(id)
                .contains(Keywords::FLYING.union(Keywords::HASTE))
        );
    }

    let first_turn = game.state.turn;
    let mut normal = game.clone();
    advance_until(&mut normal, |g| g.state.step == Step::End);
    assert_eq!(normal.state.stack.len(), 1);
    settle(&mut normal);
    assert!(tokens.iter().all(|&id| !normal.on_battlefield(id)));

    game.add_characteristics_to(
        &[source_ref],
        CharacteristicAddition {
            card_types: 16,
            creature_types: &[],
            colors: 0,
            duration: AnimationDuration::Permanent,
        },
    );
    game.move_object(source, Zone::Exile);
    assert_ne!(game.object_ref(source), source_ref);
    let finish = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, finish, None);
    pass_pair(&mut game);
    assert_eq!(game.state.turn, first_turn + 1);
    assert_eq!(game.state.active, opponent);
    for &id in &tokens {
        assert!(game.on_battlefield(id));
        assert!(!game.keywords(id).contains(Keywords::HASTE));
    }
    game.record_control(game.object_ref(tokens[1]), opponent, None);
    advance_until(&mut game, |g| g.state.step == Step::End);
    settle(&mut game);
    assert!(tokens.iter().all(|&id| game.on_battlefield(id)));

    advance_until(&mut game, |g| {
        g.state.turn == first_turn + 2 && g.state.step == Step::End
    });
    assert_eq!(game.state.stack.len(), 1);
    let delayed = &game.state.stack[0];
    let departed = delayed.source.unwrap();
    assert_eq!(departed.object, source_ref);
    assert_eq!(departed.card, named("Hill Giant"));
    assert_eq!(departed.facts.kinds & 17, 17);
    assert_eq!(
        delayed.delayed_sacrifice.as_ref().unwrap().tokens,
        token_refs
    );
    settle(&mut game);
    assert!(!game.on_battlefield(tokens[0]));
    assert!(game.on_battlefield(tokens[1]));
    assert_eq!(game.object_view(tokens[1]).controller, opponent);
}
