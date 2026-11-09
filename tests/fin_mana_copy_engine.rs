//! Immediate mana abilities that register an owned next-spell copy callback.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const SOURCE: &str = "Gilded Lotus";
const ZERO: Mana = [0; 6];
const BLUE: Mana = [0, 1, 0, 0, 0, 0];
const MANA_COPY: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            exile_self: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::AddMana(ManaProduction::Fixed(BLUE)),
            Instruction::CopyNextInstantSorcery { ability: 0 },
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyCapturedSpell {
            count: SpellCopyCount::One,
            retarget: true,
        }],
    }],
    ..CardRules::EMPTY
};

fn mana(card: CardId) -> Action {
    Action::ActivateMana {
        card,
        ability: 0,
        payment: ZERO,
        counter_payment: Counters::default(),
        production: BLUE,
    }
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn mana_payment_registers_before_the_cast_and_copies_the_countered_spell_once() {
    let _rules = fixture_rules::activate(&[(Card::from_name(SOURCE).unwrap(), &MANA_COPY)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut parent = scenario(player);
    parent.state.players[player.index()].mana = ZERO;
    let source = put(&mut parent, player, SOURCE, Zone::Battlefield);
    let reference = parent.object_ref(source);
    let first = put(&mut parent, opponent, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut parent, opponent, "Hill Giant", Zone::Battlefield);
    let original = put(&mut parent, player, "Unsummon", Zone::Hand);
    let announce = Action::AnnounceCast { card: original };

    let mut tapped = parent.clone();
    tapped.state.objects[source as usize].tapped = true;
    reject(&mut tapped, announce.clone());
    reject(&mut tapped, mana(source));
    assert!(tapped.observe(player).delayed_spell_copies.is_empty());

    act(&mut parent, announce);
    act(
        &mut parent,
        Action::ChooseTarget {
            target: Target::Card(first),
        },
    );
    if parent.legal_actions().contains(&Action::FinishTargets) {
        act(&mut parent, Action::FinishTargets);
    }
    assert_eq!(parent.state.pending, Pending::PayCosts { player });
    let stack_before = parent.state.stack.clone();
    act(&mut parent, mana(source));
    assert_eq!(parent.state.pending, Pending::PayCosts { player });
    assert_eq!(parent.state.stack, stack_before);
    assert_eq!(parent.state.players[player.index()].mana, BLUE);
    assert_eq!(parent.object_view(source).zone, Zone::Exile);
    assert_ne!(parent.object_ref(source), reference);
    let waiting = parent.observe(player).delayed_spell_copies;
    assert_eq!(waiting.len(), 1);
    assert_eq!(waiting[0].source.unwrap().object, reference);
    assert_eq!(waiting[0].controller, player);
    reject(&mut parent, mana(source));

    pay(&mut parent);
    assert_eq!(parent.state.stack.len(), 2);
    assert!(parent.observe(player).delayed_spell_copies.is_empty());
    assert_eq!(parent.state.players[player.index()].mana, ZERO);
    act(&mut parent, Action::Pass);
    assert_eq!(parent.actor(), Some(opponent));
    let negate = put(&mut parent, opponent, "Negate", Zone::Hand);
    cast(&mut parent, negate, Some(Target::Spell(original)));
    pass_pair(&mut parent);
    assert_eq!(parent.object_view(original).zone, Zone::Graveyard);
    assert_eq!(parent.state.stack.len(), 1);
    pass_pair(&mut parent);
    assert_eq!(parent.state.pending, Pending::Retarget { player });
    let retarget = parent.observe(player).retarget.unwrap();
    assert!(retarget.item.is_copy);
    assert_eq!(retarget.original[0].target, Target::Card(first));

    let choose_new = Action::SetStackTarget {
        index: 0,
        target: Target::Card(second),
    };
    let revision = parent.revision();
    let index = parent
        .legal_actions()
        .iter()
        .position(|action| *action == choose_new)
        .unwrap();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    drop(parent);
    act(&mut raw, choose_new);
    indexed.apply_index(player, revision, index).unwrap();
    for branch in [&mut raw, &mut indexed] {
        act(branch, Action::FinishStackTargets);
        settle(branch);
        assert_eq!(branch.object_view(first).zone, Zone::Battlefield);
        assert_eq!(branch.object_view(second).zone, Zone::Hand);
        assert_eq!(
            branch.state.players[player.index()].spells_cast_this_turn,
            1
        );
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());

    raw.state.players[player.index()].mana = BLUE;
    let later = put(&mut raw, player, "Unsummon", Zone::Hand);
    cast(&mut raw, later, Some(Target::Card(first)));
    assert_eq!(raw.state.stack.len(), 1);
    settle(&mut raw);
    assert_eq!(raw.object_view(first).zone, Zone::Hand);
}

#[test]
fn the_callback_follows_the_activator_after_its_mana_funds_a_creature() {
    let _rules = fixture_rules::activate(&[(Card::from_name(SOURCE).unwrap(), &MANA_COPY)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, opponent, SOURCE, Zone::Battlefield);
    game.record_control(game.object_ref(source), player, None);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 0];
    act(&mut game, mana(source));
    assert!(game.state.stack.is_empty());
    assert_eq!(game.actor(), Some(player));
    assert_eq!(game.object_view(source).owner, opponent);
    assert_eq!(game.object_view(source).zone, Zone::Exile);

    let bear = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    cast(&mut game, bear, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].mana, ZERO);
    assert_eq!(game.observe(player).delayed_spell_copies.len(), 1);

    act(&mut game, Action::Pass);
    let shock = put(&mut game, opponent, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(player)));
    settle(&mut game);
    assert_eq!(game.observe(player).delayed_spell_copies.len(), 1);

    let mut expired = game.clone();
    expired.state.step = Step::End;
    expired.give_priority();
    pass_pair(&mut expired);
    assert_eq!(expired.state.active, opponent);
    assert!(expired.observe(player).delayed_spell_copies.is_empty());

    game.state.players[player.index()].mana = [0, 1, 0, 0, 2, 0];
    let study = put(&mut game, player, "Quick Study", Zone::Hand);
    cast(&mut game, study, None);
    assert_eq!(game.state.stack.len(), 2);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 4);
    assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 2);
    assert!(game.observe(player).delayed_spell_copies.is_empty());
}

const ATTACK_TAX: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AttackControllerTax {
            amount: 1,
            condition: StatValue::Fixed(1),
        }],
    }],
    ..CardRules::EMPTY
};

#[test]
fn retained_mana_keeps_only_its_exact_waiting_callback() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name(SOURCE).unwrap(), &MANA_COPY),
        (Card::from_name("Swiftfoot Boots").unwrap(), &ATTACK_TAX),
    ]);
    let player = Player::P0;
    let mut parent = scenario(player);
    parent.state.players[player.index()].mana = ZERO;
    put(
        &mut parent,
        player.other(),
        "Swiftfoot Boots",
        Zone::Battlefield,
    );
    let earlier = put(&mut parent, player, SOURCE, Zone::Battlefield);
    let source = put(&mut parent, player, SOURCE, Zone::Battlefield);
    let reference = parent.object_ref(source);
    let attacker = put(&mut parent, player, "Grizzly Bears", Zone::Battlefield);
    act(&mut parent, mana(earlier));
    pass_pair(&mut parent);
    pass_pair(&mut parent);
    assert_eq!(parent.state.pending, Pending::Attackers);
    assert_eq!(parent.state.players[player.index()].mana, ZERO);
    act(&mut parent, Action::Attack { card: attacker });
    act(&mut parent, Action::FinishAttackers);
    assert_eq!(parent.state.pending, Pending::AttackPayment { player });

    act(&mut parent, mana(source));
    assert_eq!(parent.state.pending, Pending::AttackPayment { player });
    let waiting = parent.observe(player).delayed_spell_copies;
    let exiled = parent.object_ref(source);
    assert_eq!(waiting.len(), 2);
    assert_eq!(waiting[1].source.unwrap().object, reference);
    act(&mut parent, Action::AbortAttackPayment);
    let receipt = parent.observe(player).mana_rollback.unwrap();
    assert_eq!(receipt.source, reference);
    assert!(receipt.can_keep && receipt.can_reverse);

    let action = Action::ChooseManaRollback { keep: true };
    let revision = parent.revision();
    let index = parent
        .legal_actions()
        .iter()
        .position(|offered| *offered == action)
        .unwrap();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut reversed = parent.clone();
    drop(parent);
    act(&mut raw, action);
    indexed.apply_index(player, revision, index).unwrap();
    act(&mut reversed, Action::ChooseManaRollback { keep: false });
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.state.players[player.index()].mana, BLUE);
    assert_eq!(raw.object_view(source).zone, Zone::Exile);
    assert_eq!(raw.object_ref(source), exiled);
    assert_eq!(raw.observe(player).delayed_spell_copies, waiting);

    assert_eq!(reversed.state.players[player.index()].mana, ZERO);
    assert_eq!(reversed.object_view(source).zone, Zone::Battlefield);
    assert!(!reversed.object_view(source).tapped);
    assert_eq!(reversed.object_ref(source), reference);
    assert_eq!(reversed.observe(player).delayed_spell_copies, waiting[..1]);
}
