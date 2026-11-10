//! Rule effects from paid spells, independent of the Spider-Man card definitions.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const GLOBAL_CHAPTER: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Chapter { number: 1 },
        targets: TargetPlan::NONE,
        program: &[Instruction::RequireAllAttacksUntilNextTurn],
    }],
    ..CardRules::EMPTY
};
const ATTACK_TAX: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AttackControllerTax {
            amount: 2,
            condition: StatValue::Fixed(1),
        }],
    }],
    ..CardRules::EMPTY
};
const BLOCK_LOCK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::GrantKeywords {
            subject: Subject::Targets,
            keywords: Keywords::CANT_BLOCK,
        }],
    ),
    ..CardRules::EMPTY
};
const BLINK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::BlinkImmediately {
            counter: None,
            tapped: false,
        }],
    ),
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap_or_else(|| panic!("unknown test card {name}"))
}

fn chapter(game: &mut Game, player: Player) {
    let saga = put(game, player, "The Last Ronin", Zone::Hand);
    cast(game, saga, None);
    settle(game);
    // The last-chapter Saga is gone. Its resolved combat rule still applies.
    assert_eq!(game.object_view(saga).zone, Zone::Graveyard);
}

fn walker(game: &mut Game, player: Player) -> ObjectRef {
    let id = put(
        game,
        player,
        "Liliana, Dreadhorde General",
        Zone::Battlefield,
    );
    game.add_counter(id, CounterKind::Loyalty, 6);
    game.object_ref(id)
}

fn advance_to(game: &mut Game, pending: Pending) {
    for _ in 0..72 {
        if game.state.pending == pending {
            return;
        }
        let action = match game.state.pending {
            Pending::Priority => Action::Pass,
            Pending::Blockers => Action::FinishBlockers,
            _ => panic!(
                "unexpected decision before combat: {:?}",
                game.state.pending
            ),
        };
        act(game, action);
    }
    panic!("combat decision did not arrive: {pending:?}");
}

fn finish_combat_choice(game: &mut Game) {
    act(game, Action::FinishAttackers);
    assert_eq!(game.state.pending, Pending::Priority);
}

#[test]
fn global_requirement_survives_its_source_applies_to_future_entrants_and_expires() {
    let _rules = fixture_rules::activate(&[(card("The Last Ronin"), &GLOBAL_CHAPTER)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let flyer = put(&mut game, player, "Spectral Sailor", Zone::Battlefield);
    let opposing = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let storm = put(&mut game, opponent, "Storm, Windrider", Zone::Battlefield);
    let own_walker = walker(&mut game, player);
    let enemy_walker = walker(&mut game, opponent);
    chapter(&mut game, player);

    let late_haste = put(&mut game, player, "Fanatical Firebrand", Zone::Hand);
    cast(&mut game, late_haste, None);
    settle(&mut game);
    let late_without_haste = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    cast(&mut game, late_without_haste, None);
    settle(&mut game);

    advance_to(&mut game, Pending::Attackers);
    assert_eq!(
        game.observe(opponent).require_all_attack_until_next_turn,
        Some([1, 0])
    );
    reject(&mut game, Action::FinishAttackers);
    reject(&mut game, Action::Attack { card: flyer });
    reject(
        &mut game,
        Action::Attack {
            card: late_without_haste,
        },
    );

    // Wrong defender is a reversible incremental choice, not a legal final
    // declaration, even if every eligible creature has been selected.
    let mut wrong_defender = game.clone();
    act(
        &mut wrong_defender,
        Action::AttackPlaneswalker {
            card: first,
            defender: enemy_walker,
        },
    );
    act(
        &mut wrong_defender,
        Action::AttackPlaneswalker {
            card: flyer,
            defender: enemy_walker,
        },
    );
    act(&mut wrong_defender, Action::Attack { card: late_haste });
    reject(&mut wrong_defender, Action::FinishAttackers);
    act(&mut wrong_defender, Action::CancelAttackDeclaration);
    assert_eq!(wrong_defender.snapshot(), game.snapshot());

    act(&mut game, Action::Attack { card: first });
    act(
        &mut game,
        Action::AttackPlaneswalker {
            card: flyer,
            defender: enemy_walker,
        },
    );
    reject(&mut game, Action::FinishAttackers);
    act(&mut game, Action::Attack { card: late_haste });
    finish_combat_choice(&mut game);

    advance_to(&mut game, Pending::Attackers);
    assert_eq!(game.state.active, opponent);
    reject(&mut game, Action::FinishAttackers);
    // The opponent can't attack another player besides the effect's
    // controller in a two-seat game, so an attack on a planeswalker is legal.
    act(
        &mut game,
        Action::AttackPlaneswalker {
            card: opposing,
            defender: own_walker,
        },
    );
    act(&mut game, Action::Attack { card: storm });
    finish_combat_choice(&mut game);

    advance_to(&mut game, Pending::Attackers);
    assert_eq!(game.state.active, player);
    assert_eq!(
        game.observe(player).require_all_attack_until_next_turn,
        None
    );
    finish_combat_choice(&mut game);
}

#[test]
fn attacking_a_player_with_an_optional_tax_is_never_required() {
    let _rules = fixture_rules::activate(&[
        (card("The Last Ronin"), &GLOBAL_CHAPTER),
        (card("Swiftfoot Boots"), &ATTACK_TAX),
    ]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    put(&mut game, opponent, "Swiftfoot Boots", Zone::Battlefield);
    let defender = walker(&mut game, opponent);
    chapter(&mut game, player);

    advance_to(&mut game, Pending::Attackers);
    reject(&mut game, Action::FinishAttackers);
    act(
        &mut game,
        Action::AttackPlaneswalker {
            card: creature,
            defender,
        },
    );
    finish_combat_choice(&mut game);
    assert_eq!(
        game.state.attackers[0].defender,
        Some(CombatDefender::Planeswalker(defender))
    );
}

fn block_locks(game: &Game, viewer: Player) -> Vec<ObjectRef> {
    game.observe(viewer)
        .keyword_effects
        .into_iter()
        .filter(|effect| effect.rules.contains(Keywords::CANT_BLOCK))
        .map(|effect| effect.recipient)
        .collect()
}

fn advance_to_blocks(game: &mut Game, attacker: CardId) {
    advance_to(game, Pending::Attackers);
    act(game, Action::Attack { card: attacker });
    finish_combat_choice(game);
    advance_to(game, Pending::Blockers);
}

#[test]
fn cannot_block_effect_is_owned_ends_at_cleanup_and_does_not_follow_a_blink() {
    let _rules =
        fixture_rules::activate(&[(card("Shock"), &BLOCK_LOCK), (card("Unsummon"), &BLINK)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut parent = scenario(player);
    let attacker = put(&mut parent, player, "Grizzly Bears", Zone::Battlefield);
    let victim = put(&mut parent, opponent, "Llanowar Elves", Zone::Battlefield);
    let other = put(&mut parent, opponent, "Grizzly Bears", Zone::Battlefield);
    let blink = put(&mut parent, opponent, "Unsummon", Zone::Hand);
    let lock = put(&mut parent, player, "Shock", Zone::Hand);
    let blank = put(&mut parent, player, "Witness Protection", Zone::Hand);
    act(&mut parent, Action::AnnounceCast { card: lock });
    let saved = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut restored = parent.clone();
    drop(parent);

    let choose_victim = Action::ChooseTarget {
        target: Target::Card(victim),
    };
    let choose_other = Action::ChooseTarget {
        target: Target::Card(other),
    };
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == choose_other)
        .unwrap();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    act(&mut raw, choose_victim.clone());
    restored.restore(&saved).unwrap();
    act(&mut restored, choose_victim);
    for branch in [&mut raw, &mut indexed, &mut restored] {
        pay(branch);
        settle(branch);
    }
    assert_eq!(raw.snapshot(), restored.snapshot());
    let old_victim = raw.object_ref(victim);
    assert_eq!(block_locks(&raw, opponent), vec![old_victim]);
    assert_eq!(
        block_locks(&indexed, player),
        vec![indexed.object_ref(other)]
    );

    cast(&mut raw, blank, Some(Target::Card(victim)));
    settle(&mut raw);
    assert!(!raw.printed_abilities_available(victim));
    assert!(raw.keywords(victim).contains(Keywords::CANT_BLOCK));

    act(&mut restored, Action::Pass);
    assert_eq!(restored.actor(), Some(opponent));
    cast(&mut restored, blink, Some(Target::Card(victim)));
    settle(&mut restored);
    assert_ne!(restored.object_ref(victim), old_victim);
    assert!(block_locks(&restored, opponent).is_empty());

    for branch in [&mut raw, &mut indexed, &mut restored] {
        advance_to_blocks(branch, attacker);
    }
    let block_victim = Action::Block {
        blocker: victim,
        attacker,
    };
    let block_other = Action::Block {
        blocker: other,
        attacker,
    };
    reject(&mut raw, block_victim.clone());
    assert!(raw.legal_actions().contains(&block_other));
    reject(&mut indexed, block_other);
    assert!(indexed.legal_actions().contains(&block_victim));
    assert!(restored.legal_actions().contains(&block_victim));
    advance_to(&mut raw, Pending::Attackers);
    assert_eq!(raw.state.active, opponent);
    assert!(block_locks(&raw, opponent).is_empty());
}
