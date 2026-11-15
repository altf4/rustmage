//! Species 8472's restriction concerns abilities from opposing artifact sources.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const PROTECTED: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::CannotBeTargetedByOpposingArtifactAbilities,
            StaticAbilityFragment::CannotBeBlockedByArtifactCreatures,
        ],
    }],
    ..CardRules::EMPTY
};
const TARGET: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Creature));
const ABILITY: ActivatedAbility = ActivatedAbility {
    cost: ActivationCost::ZERO,
    guard: ActivationGuard::Always,
    targets: TARGET,
    program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    origin: ActivationOrigin::Battlefield,
    sorcery: false,
    cycling: false,
    exhaust: false,
    once_per_turn: false,
};
const ABILITY_SOURCE: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ABILITY],
    ..CardRules::EMPTY
};
const SPELL: CardRules = CardRules {
    effect: Effect::Spell(TARGET, &[Instruction::Draw(PlayerSet::Controller, 1)]),
    ..CardRules::EMPTY
};
const BYPASS_HEXPROOF: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::OpposingCreatureHexproofAndWardDisabled],
    }],
    ..CardRules::EMPTY
};

fn activation(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

#[test]
fn opposing_artifact_abilities_cannot_target_but_other_abilities_and_artifact_spells_can() {
    let artifact = Card::from_name("Candy Trail").unwrap();
    let nonartifact = Card::from_name("Banishing Light").unwrap();
    let artifact_spell = Card::from_name("Short Bow").unwrap();
    let protected_card = Card::from_name("Gigantosaurus").unwrap();
    let _rules = fixture_rules::activate(&[
        (artifact, &ABILITY_SOURCE),
        (nonartifact, &ABILITY_SOURCE),
        (artifact_spell, &SPELL),
        (protected_card, &PROTECTED),
        (Card::LlanowarElves, &BYPASS_HEXPROOF),
    ]);
    let player = Player::P0;
    let mut only_invalid = scenario(player);
    spawn(&mut only_invalid, player.other(), protected_card, Zone::Battlefield);
    let invalid_artifact = spawn(&mut only_invalid, player, artifact, Zone::Battlefield);
    assert!(!only_invalid.legal_actions().iter().any(|action| {
        matches!(action, Action::AnnounceActivate { card, .. } | Action::Activate { card, .. } if *card == invalid_artifact)
    }));
    reject(&mut only_invalid, activation(invalid_artifact));

    let mut game = scenario(player);
    let protected = spawn(&mut game, player.other(), protected_card, Zone::Battlefield);
    let artifact_id = spawn(&mut game, player, artifact, Zone::Battlefield);
    let nonartifact_id = spawn(&mut game, player, nonartifact, Zone::Battlefield);
    let artifact_spell_id = spawn(&mut game, player, artifact_spell, Zone::Hand);
    let friendly = spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
    spawn(&mut game, player, Card::LlanowarElves, Zone::Battlefield);
    let target = Action::ChooseTarget {
        target: Target::Card(protected),
    };

    act(&mut game, activation(artifact_id));
    reject(&mut game, target.clone());
    act(&mut game, Action::ChooseTarget { target: Target::Card(friendly) });
    pay(&mut game);
    settle(&mut game);

    act(&mut game, activation(nonartifact_id));
    assert!(game.legal_actions().contains(&target));
    let checkpoint = game.snapshot();
    let revision = game.revision();
    let index = game.legal_actions().iter().position(|action| *action == target).unwrap();
    let mut indexed = game.clone();
    act(&mut game, target.clone());
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&checkpoint).unwrap();
    assert_eq!(indexed.apply_index(player, revision, index), Err(Error::StaleRevision));
    pay(&mut game);
    settle(&mut game);

    let mut branch = game.clone();
    act(&mut branch, Action::AnnounceCast { card: artifact_spell_id });
    assert!(branch.legal_actions().contains(&target));
    act(&mut branch, target);
    pay(&mut branch);
    settle(&mut branch);
    assert_eq!(branch.state.players[player.index()].cards_drawn_this_turn, 3);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
}

#[test]
fn artifact_creature_cannot_block_but_ordinary_creature_can() {
    let protected_card = Card::from_name("Gigantosaurus").unwrap();
    let _rules = fixture_rules::activate(&[(protected_card, &PROTECTED)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let attacker = spawn(&mut game, player, protected_card, Zone::Battlefield);
    let artifact_blocker = put(&mut game, player.other(), "Solemn Simulacrum", Zone::Battlefield);
    let ordinary_blocker = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    game.state.step = Step::Blockers;
    game.state.pending = Pending::Blockers;
    game.state.attackers.push(AttackDeclaration {
        card: attacker,
        defender: Some(CombatDefender::Player(player.other())),
        blocked: false,
    });
    let artifact_block = Action::Block { blocker: artifact_blocker, attacker };
    let ordinary_block = Action::Block { blocker: ordinary_blocker, attacker };
    assert!(!game.legal_actions().contains(&artifact_block));
    reject(&mut game, artifact_block);
    assert!(game.legal_actions().contains(&ordinary_block));
    act(&mut game, ordinary_block);
}

#[test]
fn artifact_ability_uses_source_controller_after_activation() {
    let artifact = Card::from_name("Candy Trail").unwrap();
    let protected_card = Card::from_name("Gigantosaurus").unwrap();
    let _rules = fixture_rules::activate(&[
        (artifact, &ABILITY_SOURCE),
        (protected_card, &PROTECTED),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let protected = spawn(&mut game, player, protected_card, Zone::Battlefield);
    let source = spawn(&mut game, player, artifact, Zone::Battlefield);

    act(&mut game, activation(source));
    let target = Action::ChooseTarget { target: Target::Card(protected) };
    assert!(game.legal_actions().contains(&target));
    act(&mut game, target);
    pay(&mut game);

    // The stack ability remains controlled by the activating player, while
    // its artifact source becomes opposing to the protected creature.
    game.state.objects[source as usize].controller = player.other();
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
    assert_eq!(game.object_view(protected).zone, Zone::Battlefield);
}

#[test]
fn departed_ability_source_uses_last_controller_and_artifact_type() {
    let artifact = Card::from_name("Candy Trail").unwrap();
    let nonartifact = Card::from_name("Banishing Light").unwrap();
    let protected_card = Card::from_name("Gigantosaurus").unwrap();
    let _rules = fixture_rules::activate(&[
        (artifact, &ABILITY_SOURCE),
        (protected_card, &PROTECTED),
    ]);
    let player = Player::P0;
    for copied_nonartifact in [false, true] {
        let mut game = scenario(player);
        let protected = spawn(&mut game, player, protected_card, Zone::Battlefield);
        let source = spawn(&mut game, player, artifact, Zone::Battlefield);
        act(&mut game, activation(source));
        act(&mut game, Action::ChooseTarget { target: Target::Card(protected) });
        pay(&mut game);
        game.state.objects[source as usize].controller = player.other();
        if copied_nonartifact {
            game.apply_temporary_copy(
                game.object_ref(source),
                CopyRecipe::new(nonartifact),
                CopyDeadline::EndOfTurn,
            );
            assert_eq!(game.object_view(source).kinds & 16, 0);
        }
        game.move_object(source, Zone::Graveyard);
        let checkpoint = game.snapshot();
        let actor = game.actor().unwrap();
        let revision = game.revision();
        let index = game.legal_actions().iter().position(|a| *a == Action::Pass).unwrap();
        let mut indexed = game.clone();
        act(&mut game, Action::Pass);
        indexed.apply_index(actor, revision, index).unwrap();
        assert_eq!(game.snapshot(), indexed.snapshot());
        settle(&mut game);
        settle(&mut indexed);
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&checkpoint).unwrap();
        settle(&mut indexed);
        assert_eq!(game.snapshot(), indexed.snapshot());
        assert_eq!(
            game.state.players[player.index()].cards_drawn_this_turn,
            u64::from(copied_nonartifact),
            "departure LKI must govern artifact ability target legality"
        );
    }
}
