//! Lifetime activation limits survive turn changes without behaving like exhaust.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const BASE_ABILITY: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        colored: [0, 1, 0, 0, 0, 0],
        generic: 1,
        mana_component: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[
        Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::PlusOne,
            amount: 1,
        },
        Instruction::Draw(PlayerSet::Controller, 1),
    ],
    sorcery: false,
    once_per_turn: false,
};
const CREATURE: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    activated: &[
        ActivatedAbility {
            guard: ActivationGuard::OncePerIncarnation(&ActivationGuard::GraveyardCardsAtLeast {
                kind: 0,
                minimum: 7,
            }),
            ..BASE_ABILITY
        },
        ActivatedAbility {
            once_per_turn: true,
            ..BASE_ABILITY
        },
    ],
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
const FULL: Mana = [0, 1, 0, 0, 0, 1];
const BLUE: Mana = [0, 1, 0, 0, 0, 0];

fn activation(card: CardId, ability: u16) -> Action {
    Action::Activate {
        card,
        ability,
        target: None,
        payment: FULL,
        counter_payment: Counters::default(),
    }
}

#[test]
fn an_ordinary_lifetime_limit_survives_new_turns_and_resets_on_blink() {
    let creature = Card::from_name("Grizzly Bears").unwrap();
    let blink = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(creature, &CREATURE), (blink, &BLINK)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let source = spawn(&mut game, player, creature, Zone::Battlefield);
        put(&mut game, player, "Boom Scholar", Zone::Battlefield);
        put(&mut game, player, "Elvish Refueler", Zone::Battlefield);
        for _ in 0..6 {
            put(&mut game, player, "Forest", Zone::Graveyard);
        }
        let original = game.object_ref(source);
        let first = activation(source, 0);
        game.state.players[player.index()].mana = FULL;
        reject(&mut game, first.clone());
        put(&mut game, player, "Forest", Zone::Graveyard);
        game.state.players[player.index()].mana = BLUE;
        reject(
            &mut game,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );

        game.state.players[player.index()].mana = FULL;
        let snapshot = game.snapshot();
        let mut indexed = game.clone();
        let index = game
            .legal_actions()
            .iter()
            .position(|a| a == &first)
            .unwrap();
        indexed.apply_index(player, game.revision(), index).unwrap();
        act(&mut game, first.clone());
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&snapshot).unwrap();
        assert_eq!(indexed.snapshot(), snapshot);
        settle(&mut game);
        assert_eq!(game.counter_count(source, CounterKind::PlusOne), 1);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
        assert!(
            !game.state.players[player.index()]
                .turn_facts
                .exhaust_activated_this_turn
        );
        assert_eq!(game.object_view(source).exhausted_abilities, 0);
        game.state.players[player.index()].mana = FULL;
        reject(&mut game, first.clone());
        act(&mut game, activation(source, 1));
        settle(&mut game);

        let turn = game.state.turn;
        game.finish_cleanup();
        game.finish_cleanup();
        assert_eq!(game.state.turn, turn + 2);
        assert_eq!(game.actor(), Some(player));
        assert_eq!(game.object_ref(source), original);
        game.state.players[player.index()].mana = FULL;
        reject(&mut game, first.clone());
        act(&mut game, activation(source, 1));
        settle(&mut game);

        game.state.players[player.index()].mana = BLUE;
        let spell = spawn(&mut game, player, blink, Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(source)));
        settle(&mut game);
        assert_ne!(game.object_ref(source), original);
        assert_eq!(game.counter_count(source, CounterKind::PlusOne), 0);
        game.state.players[player.index()].mana = FULL;
        act(&mut game, first);
        settle(&mut game);
        assert_eq!(game.counter_count(source, CounterKind::PlusOne), 1);
    }
}
