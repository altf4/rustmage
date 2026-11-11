//! Completed-cast subtype history and mana-value-capped next-spell copies.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const SHRINK_SPELL: &str = "Slice from the Shadows";
const COPY_SOURCE: &str = "Gilded Lotus";
const FAERIE_TOKEN: &str = "@token/81ae8bdd201bb6ba3b04171d";

const SHRINK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::GrowScaled {
                subject: Subject::Targets,
                amount: AmountValue::ChosenX,
                power: -1,
                toughness: -1,
            },
            Instruction::IfCastWithControlledSubtype { skip: 1 },
            Instruction::Grow {
                subject: Subject::Targets,
                power: -3,
                toughness: -3,
            },
        ],
    ),
    extras: &CardExtras {
        cast_controlled_subtype: Some("Faerie"),
        x_symbols: 1,
        bargain_reduction: Some(1),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const CAPPED_COPY: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyNextInstantSorcery { ability: 0 }],
        sorcery: false,
        once_per_turn: false,
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::DelayedInstantSorceryCastManaValueAtMost { maximum: 3 },
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyCapturedSpell {
            count: SpellCopyCount::One,
            retarget: true,
        }],
    }],
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn prepare_shrink(
    game: &mut Game,
    player: Player,
    target: CardId,
    value: u32,
    sacrifice: Option<ObjectRef>,
) -> CardId {
    let spell = put(game, player, SHRINK_SPELL, Zone::Hand);
    act(game, Action::AnnounceCast { card: spell });
    act(game, Action::ChooseX { value });
    act(game, Action::ChooseBargain { sacrifice });
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    assert_eq!(game.state.pending, Pending::PayCosts { player });
    spell
}

fn stack_spell(game: &Game, card: CardId) -> &SpellView {
    game.state
        .stack
        .iter()
        .find(|item| item.kind == StackKind::Spell && item.source.unwrap().id == card)
        .unwrap()
}

fn activate_copy_source(game: &mut Game, source: CardId) {
    choose(
        game,
        |action| matches!(action, Action::Activate { card, ability: 0, target: None, .. } if *card == source),
    );
}

fn register_copy(game: &mut Game, player: Player) {
    let source = put(game, player, COPY_SOURCE, Zone::Battlefield);
    activate_copy_source(game, source);
    pass_pair(game);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    assert_eq!(game.observe(player).delayed_spell_copies.len(), 1);
}

fn retarget_to(card: CardId) -> Action {
    Action::SetStackTarget {
        index: 0,
        target: Target::Card(card),
    }
}

#[test]
fn subtype_history_samples_postpayment_battlefield_and_survives_responses() {
    let _rules = fixture_rules::activate(&[(named(SHRINK_SPELL), &SHRINK)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        put(&mut game, opponent, "Barrow Naughty", Zone::Battlefield);
        let target = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);

        let mut departed = game.clone();
        let faerie = put(&mut departed, player, "Barrow Naughty", Zone::Battlefield);
        let spell = prepare_shrink(&mut departed, player, target, 1, None);
        assert!(
            !departed
                .observe(player)
                .announcing
                .unwrap()
                .cast
                .unwrap()
                .controlled_subtype_at_cast
        );
        pay(&mut departed);
        assert!(
            stack_spell(&departed, spell)
                .cast
                .unwrap()
                .controlled_subtype_at_cast
        );
        act(&mut departed, Action::Pass);
        let bounce = put(&mut departed, opponent, "Unsummon", Zone::Hand);
        cast(&mut departed, bounce, Some(Target::Card(faerie)));
        settle(&mut departed);
        assert_eq!(departed.object_view(faerie).zone, Zone::Hand);
        assert_eq!(departed.object_view(target).toughness, 6);

        let mut late = game.clone();
        let spell = prepare_shrink(&mut late, player, target, 1, None);
        pay(&mut late);
        assert!(
            !stack_spell(&late, spell)
                .cast
                .unwrap()
                .controlled_subtype_at_cast
        );
        let faerie = put(&mut late, player, "Stingblade Assassin", Zone::Hand);
        cast(&mut late, faerie, None);
        settle(&mut late);
        assert_eq!(late.object_view(faerie).zone, Zone::Battlefield);
        assert_eq!(late.object_view(target).toughness, 9);

        let token = game.state.objects.len() as CardId;
        game.create_tokens(player, named(FAERIE_TOKEN), 1, false);
        let faerie = game.object_view(token);
        assert!(faerie.is_token() && faerie.has_subtype("Faerie"));
        let offering = faerie.reference;
        let spell = prepare_shrink(&mut game, player, target, 1, Some(offering));
        assert_eq!(game.object_view(token).zone, Zone::Battlefield);
        pay(&mut game);
        assert_ne!(game.object_view(token).zone, Zone::Battlefield);
        let receipt = stack_spell(&game, spell).cast.unwrap();
        assert_eq!(receipt.bargain, Some(true));
        assert!(!receipt.controlled_subtype_at_cast);
        settle(&mut game);
        assert_eq!(game.object_view(target).toughness, 9);
    }
}

#[test]
fn capped_callback_uses_committed_mana_value_and_the_adventure_face() {
    let _rules = fixture_rules::activate(&[
        (named(SHRINK_SPELL), &SHRINK),
        (named(COPY_SOURCE), &CAPPED_COPY),
    ]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        register_copy(&mut game, player);

        let land = put(&mut game, player, "Forest", Zone::Hand);
        act(&mut game, Action::PlayLand { card: land });
        assert_eq!(game.observe(player).delayed_spell_copies.len(), 1);

        act(&mut game, Action::Pass);
        let shock = put(&mut game, opponent, "Shock", Zone::Hand);
        cast(&mut game, shock, Some(Target::Player(player)));
        settle(&mut game);
        assert_eq!(game.observe(player).delayed_spell_copies.len(), 1);

        let first = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
        let offering = put(&mut game, player, "Short Bow", Zone::Battlefield);
        let offering = game.object_ref(offering);
        let over_cap = prepare_shrink(&mut game, player, first, 3, Some(offering));
        reject(&mut game, Action::PayCosts { payment: [0; 6] });
        assert_eq!(game.observe(player).delayed_spell_copies.len(), 1);
        pay(&mut game);
        let spell = stack_spell(&game, over_cap);
        assert_eq!(spell.mana_value(), 4);
        assert_eq!(spell.cast.unwrap().mana_spent(), 3);
        settle(&mut game);
        assert_eq!(game.object_view(first).toughness, 7);
        assert_eq!(game.observe(player).delayed_spell_copies.len(), 1);

        let creature = put(&mut game, player, "Grizzly Bears", Zone::Hand);
        cast(&mut game, creature, None);
        settle(&mut game);
        assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
        assert_eq!(game.observe(player).delayed_spell_copies.len(), 1);

        let second = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
        game.state.objects[first as usize].tapped = true;
        game.state.objects[second as usize].tapped = true;
        let adventure = put(&mut game, player, "Threadbind Clique", Zone::Hand);
        act(&mut game, Action::AnnounceAdventure { card: adventure });
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(first),
            },
        );
        pay(&mut game);
        assert!(game.observe(player).delayed_spell_copies.is_empty());
        assert_eq!(stack_spell(&game, adventure).mana_value(), 3);
        pass_pair(&mut game);
        let copy = game.observe(player).retarget.unwrap().item;
        assert!(copy.is_copy);
        assert_eq!(copy.cast.unwrap().face, CastFace::Adventure);
        assert_eq!(copy.mana_value(), 3);
        act(&mut game, retarget_to(second));
        act(&mut game, Action::FinishStackTargets);
        settle(&mut game);
        assert_eq!(game.object_view(adventure).zone, Zone::Exile);
        assert_eq!(game.object_view(first).zone, Zone::Graveyard);
        assert_eq!(game.object_view(second).zone, Zone::Graveyard);
    }
}

#[test]
fn retargeted_copy_retains_x_and_paid_bargain_without_cast_subtype_history() {
    let _rules = fixture_rules::activate(&[
        (named(SHRINK_SPELL), &SHRINK),
        (named(COPY_SOURCE), &CAPPED_COPY),
    ]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        register_copy(&mut game, player);
        put(&mut game, player, "Barrow Naughty", Zone::Battlefield);
        let first = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
        let second = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
        let offering = put(&mut game, player, "Short Bow", Zone::Battlefield);
        let offering = game.object_ref(offering);
        let spell = prepare_shrink(&mut game, player, first, 2, Some(offering));
        assert_eq!(game.observe(player).delayed_spell_copies.len(), 1);
        pay(&mut game);
        assert_eq!(stack_spell(&game, spell).mana_value(), 3);
        assert!(
            stack_spell(&game, spell)
                .cast
                .unwrap()
                .controlled_subtype_at_cast
        );
        assert!(game.observe(player).delayed_spell_copies.is_empty());
        pass_pair(&mut game);
        let copy = game.observe(player).retarget.unwrap().item;
        assert!(copy.is_copy);
        assert_eq!(copy.x, Some(2));
        assert_eq!(copy.cast.unwrap().bargain, Some(true));
        assert!(!copy.cast.unwrap().controlled_subtype_at_cast);

        let invalid = Action::SetStackTarget {
            index: 0,
            target: Target::Player(opponent),
        };
        reject(&mut game, invalid);
        let action = retarget_to(second);
        let checkpoint = game.snapshot();
        let revision = game.revision();
        let index = game
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        let mut indexed = game.clone();
        assert_eq!(
            indexed.apply(opponent, action.clone()),
            Err(Error::WrongActor)
        );
        assert_eq!(indexed.snapshot(), checkpoint);
        act(&mut game, action);
        indexed.apply_index(player, revision, index).unwrap();
        for branch in [&mut game, &mut indexed] {
            act(branch, Action::FinishStackTargets);
            settle(branch);
            assert_eq!(branch.object_view(first).toughness, 5);
            assert_eq!(branch.object_view(second).toughness, 8);
            assert_eq!(
                branch.state.players[player.index()].spells_cast_this_turn,
                1
            );
        }
        assert_eq!(indexed.snapshot(), game.snapshot());

        indexed.restore(&checkpoint).unwrap();
        act(&mut indexed, Action::FinishStackTargets);
        settle(&mut indexed);
        assert_eq!(indexed.object_view(first).toughness, 3);
        assert_eq!(indexed.object_view(second).toughness, 10);
    }
}

#[test]
fn eligible_uncast_copy_does_not_consume_registration_and_cleanup_expires_it() {
    let _rules = fixture_rules::activate(&[(named(COPY_SOURCE), &CAPPED_COPY)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let provider = put(&mut game, player, COPY_SOURCE, Zone::Battlefield);
        let teach = put(&mut game, player, "Teach by Example", Zone::Hand);
        cast(&mut game, teach, None);
        settle(&mut game);
        let shock = put(&mut game, player, "Shock", Zone::Hand);
        cast(&mut game, shock, Some(Target::Player(opponent)));
        assert_eq!(game.state.stack.len(), 2);
        activate_copy_source(&mut game, provider);
        pass_pair(&mut game);
        assert_eq!(game.observe(player).delayed_spell_copies.len(), 1);
        pass_pair(&mut game);
        act(&mut game, Action::FinishStackTargets);
        settle(&mut game);
        assert_eq!(game.state.players[opponent.index()].life, 16);
        assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 2);
        assert_eq!(game.observe(player).delayed_spell_copies.len(), 1);

        game.state.step = Step::End;
        game.give_priority();
        pass_pair(&mut game);
        assert_eq!(game.state.active, opponent);
        assert!(game.observe(player).delayed_spell_copies.is_empty());
    }
}
