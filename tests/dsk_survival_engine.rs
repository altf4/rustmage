//! Second-main timing and exact triggered ability usage.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const SURVIVOR: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SURVIVAL,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
    }],
    ..CardRules::EMPTY
};

const LIMITED_SURVIVOR: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OncePerIncarnation(&TriggerCondition::SURVIVAL),
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::Flying,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};

const EXTRA_PHASES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CombatAndMainAfterCurrentMain(1)],
    ),
    ..CardRules::EMPTY
};

const UNTAP: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::Untap],
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

const COUNTER_ABILITY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
        &[Instruction::Counter],
    ),
    ..CardRules::EMPTY
};

const TWO_ENTRY_TRIGGERS: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::OncePerIncarnation(&TriggerCondition::SelfEnters),
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
    ],
    ..CardRules::EMPTY
};

fn advance_until(game: &mut Game, stop: impl Fn(&Observation) -> bool) {
    for _ in 0..96 {
        if stop(&game.observe(game.actor().unwrap())) {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::FinishAttackers
                    | Action::FinishBlockers
                    | Action::PutTrigger { .. }
            )
        });
    }
    panic!("phase did not advance within 96 actions");
}

fn attack_to_end_combat(game: &mut Game, player: Player, source: CardId) {
    advance_until(game, |view| {
        view.step == Step::Attackers && view.actor == Some(player)
    });
    act(game, Action::Attack { card: source });
    advance_until(game, |view| {
        view.step == Step::EndCombat && view.actor == Some(player.other())
    });
}

fn cast_funded(game: &mut Game, player: Player, card: CardId, target: Option<Target>) {
    assert_eq!(game.actor(), Some(player));
    game.state.players[player.index()].mana = [8; 6];
    cast(game, card, target);
}

fn next_main(game: &mut Game, player: Player) {
    let turn = game.observe(player).turn;
    advance_until(game, |view| {
        view.turn > turn && view.active == player && view.step == Step::Main1
    });
}

#[test]
fn survival_checks_the_second_main_and_last_tapped_status() {
    let full_throttle = Card::from_name("Full Throttle").unwrap();
    let opt = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &SURVIVOR),
        (full_throttle, &EXTRA_PHASES),
        (opt, &UNTAP),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let extra = spawn(&mut game, player, full_throttle, Zone::Hand);
        let untap = spawn(&mut game, player, opt, Zone::Hand);
        let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
        game.begin_step(Step::Main1);
        cast_funded(&mut game, player, extra, None);
        settle(&mut game);
        attack_to_end_combat(&mut game, player, source);
        act(&mut game, Action::Pass);
        let at_second_main = game.observe(player);
        assert_eq!(at_second_main.step, Step::Main2);
        assert_eq!(
            at_second_main.turn_facts.as_ref().unwrap()[player.index()].main_phases_started,
            2
        );
        assert_eq!(at_second_main.stack.len(), 1);

        let mut left_tapped = game.clone();
        cast_funded(&mut left_tapped, player, bounce, Some(Target::Card(source)));
        settle(&mut left_tapped);
        assert_eq!(left_tapped.observe(player).players[player.index()].life, 22);

        let mut left_untapped = game.clone();
        cast_funded(
            &mut left_untapped,
            player,
            untap,
            Some(Target::Card(source)),
        );
        act(&mut left_untapped, Action::Pass);
        act(&mut left_untapped, Action::Pass);
        assert_eq!(left_untapped.observe(player).stack.len(), 1);
        assert!(!left_untapped.object_view(source).tapped);
        cast_funded(
            &mut left_untapped,
            player,
            bounce,
            Some(Target::Card(source)),
        );
        settle(&mut left_untapped);
        assert_eq!(
            left_untapped.observe(player).players[player.index()].life,
            20
        );

        settle(&mut game);
        advance_until(&mut game, |view| {
            view.step == Step::Main2
                && view.turn_facts.as_ref().unwrap()[player.index()].main_phases_started == 3
        });
        assert!(game.object_view(source).tapped);
        assert!(game.observe(player).stack.is_empty());
        assert_eq!(game.observe(player).players[player.index()].life, 22);
    }
}

#[test]
fn lifetime_limit_is_owned_and_counts_a_countered_occurrence_until_blink() {
    let shock = Card::from_name("Shock").unwrap();
    let growth = Card::from_name("Giant Growth").unwrap();
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &LIMITED_SURVIVOR),
        (shock, &COUNTER_ABILITY),
        (growth, &BLINK),
    ]);
    let player = Player::P0;
    let mut parent = scenario(player);
    let source = put(&mut parent, player, "Grizzly Bears", Zone::Battlefield);
    let counter = spawn(&mut parent, player, shock, Zone::Hand);
    let blink = spawn(&mut parent, player, growth, Zone::Hand);
    let original = parent.object_ref(source);
    parent.begin_step(Step::Main1);
    attack_to_end_combat(&mut parent, player, source);
    let revision = parent.revision();
    let pass_index = parent
        .legal_actions()
        .iter()
        .position(|action| *action == Action::Pass)
        .unwrap();
    let checkpoint = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    drop(parent);
    act(&mut raw, Action::Pass);
    indexed
        .apply_index(player.other(), revision, pass_index)
        .unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    indexed.restore(&checkpoint).unwrap();
    assert!(
        indexed
            .observe(player)
            .triggered_this_incarnation
            .is_empty()
    );
    indexed
        .apply_index(player.other(), indexed.revision(), pass_index)
        .unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());

    let used = IncarnationTriggerUse {
        source: original,
        definition: Card::GrizzlyBears,
        ability: 0,
        origin: None,
        acquired_link_cohort: None,
    };
    for viewer in [player, player.other()] {
        assert_eq!(raw.observe(viewer).triggered_this_incarnation, vec![used]);
    }
    settle(&mut indexed);
    assert_eq!(indexed.counter_count(source, CounterKind::Flying), 1);

    let trigger = raw.observe(player).stack[0].id;
    cast_funded(&mut raw, player, counter, Some(Target::Stack(trigger)));
    settle(&mut raw);
    assert_eq!(raw.counter_count(source, CounterKind::Flying), 0);
    next_main(&mut raw, player);
    attack_to_end_combat(&mut raw, player, source);
    act(&mut raw, Action::Pass);
    assert!(raw.observe(player).stack.is_empty());
    assert_eq!(raw.observe(player).triggered_this_incarnation, vec![used]);

    cast_funded(&mut raw, player, blink, Some(Target::Card(source)));
    settle(&mut raw);
    assert_ne!(raw.object_ref(source), original);
    assert!(raw.observe(player).triggered_this_incarnation.is_empty());
    next_main(&mut raw, player);
    attack_to_end_combat(&mut raw, player, source);
    act(&mut raw, Action::Pass);
    settle(&mut raw);
    assert_eq!(raw.counter_count(source, CounterKind::Flying), 1);
}

#[test]
fn extra_entry_triggers_respect_the_lifetime_limit_of_each_ability() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &TWO_ENTRY_TRIGGERS)]);
    let player = Player::P1;
    let mut game = scenario(player);
    put(&mut game, player, "Starfield Vocalist", Zone::Battlefield);
    let source = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    cast(&mut game, source, None);
    advance_until(&mut game, |view| {
        view.stack.is_empty() && view.decision == "priority"
    });
    let player_view = &game.observe(player).players[player.index()];
    assert_eq!(player_view.cards_drawn_this_turn, 2);
    assert_eq!(player_view.life, 21);
    let uses = game.observe(player).triggered_this_incarnation;
    assert_eq!(uses.len(), 1);
    assert_eq!(uses[0].source, game.object_ref(source));
    assert_eq!(uses[0].ability, 0);
}
