//! Animation-granted static abilities follow the exact object and current controller.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const BLUE_RED: u8 = 0b01010;
const BLUE_BLACK: u8 = 0b00110;
const ANIMATION_COST: Mana = [0, 1, 0, 1, 0, 0];

const fn animation(program: &'static [Instruction]) -> ActivatedAbility {
    ActivatedAbility {
        cost: ActivationCost {
            colored: ANIMATION_COST,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        exhaust: false,
        cycling: false,
        sorcery: false,
        once_per_turn: false,
    }
}
const LAND: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[
        animation(&[Instruction::AnimateSourceColoredWithAbility {
            spec: &AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Named(&["Elemental"]),
                base: Some((2, 1)),
                keywords: Keywords::NONE,
                duration: AnimationDuration::EndOfTurn,
            },
            colors: BLUE_RED,
            kind: SelfGrantedAbilityKind::OwnTurnFirstStrike,
        }]),
        animation(&[Instruction::AnimateSourceColored(
            AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Named(&["Shark"]),
                base: Some((4, 4)),
                keywords: Keywords::DEATHTOUCH,
                duration: AnimationDuration::EndOfTurn,
            },
            BLUE_BLACK,
        )]),
    ],
    ..CardRules::EMPTY
};

const fn creature_spell(program: &'static [Instruction]) -> CardRules {
    CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program,
        ),
        ..CardRules::EMPTY
    }
}
const COPY: CardRules = creature_spell(&[Instruction::CreateCopyToken {
    subject: Subject::Targets,
    amount: 1,
}]);
const BLINK: CardRules = creature_spell(&[Instruction::BlinkImmediately {
    counter: None,
    tapped: false,
}]);
const BLANK: CardRules = creature_spell(&[Instruction::RemoveTargetAbilitiesUntilEndOfTurn]);

fn reef() -> Card {
    Card::from_name("Restless Reef").unwrap()
}

fn animate(game: &mut Game, land: CardId, ability: u16) {
    act(
        game,
        Action::Activate {
            card: land,
            ability,
            target: None,
            payment: ANIMATION_COST,
            counter_payment: Counters::default(),
        },
    );
}

fn assert_elemental(game: &Game, id: CardId, controller: Player, first_strike: bool) {
    let permanent = game.object_view(id);
    assert_eq!(permanent.controller, controller);
    assert_eq!(permanent.kinds & 3, 3);
    assert_eq!(permanent.colors, BLUE_RED);
    assert!(permanent.has_subtype("Elemental"));
    assert_eq!(stats(game, id), (2, 1));
    assert_eq!(
        permanent.keywords.contains(Keywords::FIRST_STRIKE),
        first_strike
    );
}

fn finish_turn(game: &mut Game) {
    let turn = game.state.turn;
    for _ in 0..48 {
        if game.state.turn != turn {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass | Action::FinishAttackers | Action::FinishBlockers
            )
        });
    }
    panic!("turn did not end");
}

#[test]
fn the_animation_uses_first_strike_in_its_controllers_combat() {
    let _rules = fixture_rules::activate(&[(reef(), &LAND)]);
    let owner = Player::P0;
    let mut game = scenario(owner);
    let land = put(&mut game, owner, "Restless Reef", Zone::Battlefield);
    let blocker = put(&mut game, owner.other(), "Bear Cub", Zone::Battlefield);
    animate(&mut game, land, 0);
    settle(&mut game);
    assert_elemental(&game, land, owner, true);

    for _ in 0..8 {
        if game.state.pending == Pending::Attackers {
            break;
        }
        act(&mut game, Action::Pass);
    }
    act(&mut game, Action::Attack { card: land });
    act(&mut game, Action::FinishAttackers);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    act(
        &mut game,
        Action::Block {
            blocker,
            attacker: land,
        },
    );
    act(&mut game, Action::FinishBlockers);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.state.step, Step::FirstStrikeDamage);
    assert_eq!(game.object_view(blocker).zone, Zone::Graveyard);
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
}

#[test]
fn live_control_turns_the_keyword_on_or_off_and_cleanup_removes_the_grant() {
    let _rules = fixture_rules::activate(&[(reef(), &LAND)]);
    let owner = Player::P1;
    let borrower = owner.other();
    for active in [owner, borrower] {
        let mut game = scenario(active);
        let land = put(&mut game, owner, "Restless Reef", Zone::Battlefield);
        let theft = put(&mut game, borrower, "Goatnap", Zone::Hand);
        if active == borrower {
            act(&mut game, Action::Pass);
        } else {
            put(&mut game, borrower, "High Fae Trickster", Zone::Battlefield);
        }
        assert_eq!(game.actor(), Some(owner));
        animate(&mut game, land, 0);
        settle(&mut game);
        assert_elemental(&game, land, owner, active == owner);

        if active == owner {
            act(&mut game, Action::Pass);
        }
        assert_eq!(game.actor(), Some(borrower));
        cast(&mut game, theft, Some(Target::Card(land)));
        settle(&mut game);
        assert_eq!(game.object_view(land).owner, owner);
        assert_elemental(&game, land, borrower, active == borrower);

        finish_turn(&mut game);
        let permanent = game.object_view(land);
        assert_eq!(permanent.controller, owner);
        assert_eq!(permanent.kinds, 2);
        assert!(!permanent.keywords.contains(Keywords::FIRST_STRIKE));
        assert!(game.observe(owner).self_granted_abilities.is_none());
    }
}

#[test]
fn copies_and_blinks_do_not_inherit_the_grant_and_layer_six_can_remove_it() {
    let _rules = fixture_rules::activate(&[
        (reef(), &LAND),
        (Card::from_name("Opt").unwrap(), &COPY),
        (Card::from_name("Think Twice").unwrap(), &BLINK),
        (Card::from_name("Unsummon").unwrap(), &BLANK),
    ]);
    let owner = Player::P0;
    let mut game = scenario(owner);
    let land = put(&mut game, owner, "Restless Reef", Zone::Battlefield);
    let copy_spell = put(&mut game, owner, "Opt", Zone::Hand);
    let blink = put(&mut game, owner, "Think Twice", Zone::Hand);
    let blank = put(&mut game, owner, "Unsummon", Zone::Hand);
    animate(&mut game, land, 0);
    settle(&mut game);
    assert_elemental(&game, land, owner, true);

    let mut blanked = game.clone();
    cast(&mut blanked, blank, Some(Target::Card(land)));
    settle(&mut blanked);
    assert_elemental(&blanked, land, owner, false);

    cast(&mut game, copy_spell, Some(Target::Card(land)));
    settle(&mut game);
    let copy = game
        .state
        .objects
        .iter()
        .position(|object| object.zone == Zone::Battlefield && object.is_token())
        .unwrap() as CardId;
    assert_eq!(game.object_view(copy).kinds, 2);
    assert!(!game.keywords(copy).contains(Keywords::FIRST_STRIKE));

    animate(&mut game, copy, 1);
    settle(&mut game);
    assert_eq!(stats(&game, copy), (4, 4));
    assert_eq!(game.object_view(copy).colors, BLUE_BLACK);
    assert!(game.keywords(copy).contains(Keywords::DEATHTOUCH));
    assert!(!game.keywords(copy).contains(Keywords::FIRST_STRIKE));
    assert_elemental(&game, land, owner, true);

    // Blanking the copy in response clears its old abilities. The resolving
    // animation can still grant a new ability with the later timestamp.
    let mut late = game.clone();
    animate(&mut late, copy, 0);
    cast(&mut late, blank, Some(Target::Card(copy)));
    settle(&mut late);
    assert_elemental(&late, copy, owner, true);
    assert!(!late.keywords(copy).contains(Keywords::DEATHTOUCH));

    // Another pending activation cannot follow the land across an immediate blink.
    let prior = game.object_ref(land);
    animate(&mut game, land, 0);
    cast(&mut game, blink, Some(Target::Card(land)));
    settle(&mut game);
    assert_ne!(game.object_ref(land), prior);
    assert_eq!(game.object_view(land).kinds, 2);
    assert!(!game.keywords(land).contains(Keywords::FIRST_STRIKE));

    animate(&mut game, land, 1);
    settle(&mut game);
    assert!(game.keywords(land).contains(Keywords::DEATHTOUCH));
    assert!(!game.keywords(land).contains(Keywords::FIRST_STRIKE));
}
