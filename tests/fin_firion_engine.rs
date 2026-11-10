//! Copiable equip reductions and one-shot exact upkeep sacrifices.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const FIRION: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FilteredEntersYou(EntryPredicate::NontokenEquipment),
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyEnteringWithEquipDiscount(2)],
    }],
    ..CardRules::EMPTY
};

const EQUIP: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    cycling: false,
    origin: ActivationOrigin::Battlefield,
    cost: ActivationCost {
        colored: [0, 0, 0, 1, 0, 0],
        generic: 3,
        mana_component: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
    program: &[Instruction::AttachSource],
    sorcery: true,
    once_per_turn: false,
};
const EQUIPMENT: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[
        EQUIP,
        ActivatedAbility {
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
            ..EQUIP
        },
    ],
    ..CardRules::EMPTY
};

const COPY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 16,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[Instruction::CreateCopyToken {
            subject: Subject::Targets,
            amount: 1,
        }],
    ),
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn copies(game: &Game, card: Card) -> Vec<ObjectRef> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, object)| {
            object.zone == Zone::Battlefield && object.card == card && object.is_token()
        })
        .map(|(id, _)| game.object_ref(id as CardId))
        .collect()
}

fn settle_entries(game: &mut Game) {
    for _ in 0..32 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::ChooseEntryTimestamp { .. }
                    | Action::PutTrigger { .. }
                    | Action::ChooseCreatedToken { .. }
            )
        });
    }
    panic!("copy and Job select must finish within the fixture budget");
}

fn next_upkeep(game: &mut Game, player: Player) {
    game.state.step = Step::Main2;
    act(game, Action::Pass);
    act(game, Action::Pass);
    assert_eq!(game.state.step, Step::End);
    assert_eq!(game.observe(player).delayed_sacrifices.len(), 1);
    assert!(game.state.stack.is_empty());
    act(game, Action::Pass);
    act(game, Action::Pass);
    assert_eq!(game.state.step, Step::Upkeep);
    assert_eq!(game.state.active, player.other());
    let trigger = game.state.stack.last().unwrap();
    assert_eq!(trigger.kind, StackKind::DelayedSacrifice);
    assert_eq!(trigger.controller, player);
    assert!(trigger.targets.is_empty());
}

#[test]
fn discount_copies_and_applies_only_to_the_equips_generic_component() {
    let bow = named("Short Bow");
    let _rules = fixture_rules::activate(&[
        (Card::HillGiant, &FIRION),
        (bow, &EQUIPMENT),
        (named("Opt"), &COPY),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let firion = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let recipient = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let original = put(&mut game, player, "Short Bow", Zone::Hand);
    let foreign = put(&mut game, player.other(), "Short Bow", Zone::Hand);
    game.move_object(foreign, Zone::Battlefield);
    assert!(game.state.pending_triggers.is_empty());
    game.move_object(original, Zone::Battlefield);
    game.prepare_triggers();
    assert_eq!(game.state.stack.len(), 1);
    assert_eq!(
        game.state.stack[0].entering_object,
        Some(game.object_ref(original))
    );

    settle(&mut game);
    let [temporary]: [ObjectRef; 1] = copies(&game, bow).try_into().expect("one nontoken entry");
    let delay = &game.observe(player.other()).delayed_sacrifices[0];
    assert!(delay.next_upkeep);
    assert_eq!(delay.tokens, [temporary]);

    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(temporary.card)));
    settle(&mut game);
    assert_eq!(game.observe(player).delayed_sacrifices.len(), 1);
    let [first, successor]: [ObjectRef; 2] = copies(&game, bow).try_into().expect("two copies");
    assert_eq!(first, temporary);
    assert_eq!(
        game.copiable_recipe(successor)
            .exceptions
            .equip_generic_reduction,
        2
    );

    let payment = [0, 0, 0, 1, 0, 1];
    let target = Target::Card(recipient);
    let announce = |card, ability| Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    };
    let direct = |object: ObjectRef| Action::Activate {
        card: object.card,
        ability: 0,
        target: Some(target),
        payment,
        counter_payment: Counters::default(),
    };
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 8];
    assert!(!game.legal_actions().contains(&announce(successor.card, 0)));
    game.state.players[player.index()].mana = payment;
    let legal = game.legal_actions();
    assert!(!legal.contains(&announce(original, 0)));
    assert!(!legal.contains(&announce(successor.card, 1)));
    assert!(legal.contains(&direct(temporary)));
    assert!(legal.contains(&direct(successor)));

    let mut indexed = game.clone();
    let mut immediate = game.clone();
    let selection = announce(successor.card, 0);
    let index = legal
        .iter()
        .position(|action| *action == selection)
        .unwrap();
    let revision = game.revision();
    let before = game.snapshot();
    indexed.apply_index(player, revision, index).unwrap();
    act(&mut game, selection);
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&before).unwrap();
    assert_eq!(
        indexed.apply_index(player, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut game, Action::ChooseTarget { target });
    act(&mut game, Action::PayCosts { payment });
    settle(&mut game);

    act(&mut immediate, direct(successor));
    settle(&mut immediate);
    for branch in [&game, &immediate] {
        assert_eq!(
            branch.object_view(successor.card).attached_to,
            Some(branch.object_ref(recipient))
        );
        assert_eq!(branch.state.players[player.index()].mana, [0; 6]);
    }

    game.move_object(firion, Zone::Graveyard);
    next_upkeep(&mut game, player);
    settle(&mut game);
    assert_eq!(copies(&game, bow), [successor]);
    assert_eq!(game.object_view(original).zone, Zone::Battlefield);
}

#[test]
fn departed_entrant_lki_copies_job_select_and_all_exact_replacement_tokens() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &FIRION)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let firion = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let old = put(&mut game, player, "Short Bow", Zone::Hand);
    let copy_of = named("Monk's Fist");
    put(&mut game, player, "Doubling Season", Zone::Battlefield);
    game.move_object(old, Zone::Battlefield);
    game.prepare_triggers();
    let entrant = game.object_ref(old);
    game.apply_temporary_copy(entrant, CopyRecipe::new(copy_of), CopyDeadline::EndOfTurn);
    game.state.objects[old as usize].tapped = true;
    game.add_counter(old, CounterKind::Charge, 4);
    game.move_object(firion, Zone::Graveyard);
    game.move_object(old, Zone::Exile);
    game.move_object(old, Zone::Battlefield);
    assert_ne!(game.object_ref(old), entrant);
    assert_eq!(game.object_view(old).card, named("Short Bow"));
    settle_entries(&mut game);

    let created = copies(&game, copy_of);
    let &[first, second] = created.as_slice() else {
        panic!("replacement creates two Equipment copies");
    };
    let observed = game.observe(player.other());
    let [delay] = observed.delayed_sacrifices.as_slice() else {
        panic!("one delayed ability for the replaced creation batch");
    };
    assert_eq!(delay.tokens, created);
    assert!(delay.next_upkeep);
    let mut recipients = Vec::new();
    for copy in [first, second] {
        let view = game.object_view(copy.card);
        assert!(!view.tapped);
        assert_eq!(view.counters.get(CounterKind::Charge), 0);
        assert_eq!(
            game.copiable_recipe(copy)
                .exceptions
                .equip_generic_reduction,
            2
        );
        let attached = view.attached_to.expect("each copied Job select attaches");
        let bearer = game.object_view(attached.card);
        assert!(bearer.is_token() && bearer.has_subtype("Hero") && bearer.has_subtype("Monk"));
        recipients.push(attached);
    }
    assert_ne!(recipients[0], recipients[1]);

    let mut changed = game.clone();
    changed.move_object(first.card, Zone::Exile);
    changed.state_based_actions();
    let [reused]: [ObjectRef; 1] = changed
        .create_token_batch_capture(player, Card::DrakeToken, 1, false, None, true)
        .try_into()
        .expect("one replacement incarnation");
    assert_eq!(reused.card, first.card);
    assert_ne!(reused, first);
    changed.record_control(second, player.other(), None);

    next_upkeep(&mut game, player);
    next_upkeep(&mut changed, player);
    settle(&mut game);
    settle(&mut changed);
    assert!(copies(&game, copy_of).is_empty());
    assert_eq!(changed.object_view(second.card).controller, player.other());
    assert_eq!(changed.object_view(second.card).zone, Zone::Battlefield);
    assert_eq!(changed.object_ref(reused.card), reused);
    assert_eq!(changed.object_view(reused.card).zone, Zone::Battlefield);
}
