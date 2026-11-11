//! Owned private entry groups and repeated blink choices.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const REPEAT_BLINK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::BlinkControlledCreaturesRepeated],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn resolve_top(g: &mut Game) {
    act(g, Action::Pass);
    act(g, Action::Pass);
}

#[test]
fn repeated_blink_selects_each_group_again_and_returns_under_owner_control() {
    let _rules = fixture_rules::activate(&[(card("Genesis Wave"), &REPEAT_BLINK)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let spell = put(&mut g, p, "Genesis Wave", Zone::Hand);
    let ours = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let other = put(&mut g, p.other(), "Healer's Hawk", Zone::Battlefield);
    let first = g.object_ref(ours);
    let stolen = g.object_ref(other);
    g.record_control(stolen, p, None);

    act(&mut g, Action::AnnounceCast { card: spell });
    act(&mut g, Action::ChooseX { value: 1 });
    pay(&mut g);
    resolve_top(&mut g);
    act(&mut g, Action::ChooseEffectObject { object: first });
    act(&mut g, Action::ChooseEffectObject { object: stolen });
    let saved = g.snapshot();
    let revision = g.revision();
    let index = g
        .legal_actions()
        .iter()
        .position(|a| *a == Action::FinishEffectChoice)
        .unwrap();
    let mut raw = g.clone();
    let mut indexed = g;
    act(&mut raw, Action::FinishEffectChoice);
    indexed.apply_index(p, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_ne!(raw.object_ref(ours), first);
    assert_eq!(raw.object_view(other).controller, p.other());
    let ineligible = raw.object_ref(other);
    reject(&mut raw, Action::ChooseEffectObject { object: ineligible });
    reject(&mut raw, Action::ChooseEffectObject { object: first });
    let current = raw.object_ref(ours);
    act(&mut raw, Action::ChooseEffectObject { object: current });
    act(&mut raw, Action::FinishEffectChoice);
    assert_ne!(raw.object_ref(ours), current);
    assert_eq!(raw.object_view(spell).zone, Zone::Graveyard);

    indexed.restore(&saved).unwrap();
    act(&mut indexed, Action::FinishEffectChoice);
    assert_eq!(indexed.object_ref(ours), current);
    act(&mut indexed, Action::FinishEffectChoice);
    assert_eq!(indexed.object_ref(ours), current);
    assert_ne!(indexed.snapshot(), raw.snapshot());
}

const DEPLOY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::PutHandCreatures { maximum: 2 },
            Instruction::GrantCreaturesWithPowerAtLeast {
                filter: CreatureFilter {
                    relation: 0,
                    other: false,
                    subtype: None,
                    keyword: Keywords::NONE,
                    attacking: false,
                    tokens: None,
                    artifact: false,
                },
                minimum: 4,
                keywords: Keywords::HEXPROOF,
            },
        ],
    ),
    ..CardRules::EMPTY
};
const WATCH_ENTRY: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreaturesEnterYouWithoutManaSpent,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

#[test]
fn hidden_hand_group_enters_simultaneously_then_current_power_is_sampled() {
    let _rules =
        fixture_rules::activate(&[(card("Opt"), &DEPLOY), (card("Hill Giant"), &WATCH_ENTRY)]);
    let p = Player::P1;
    let mut g = scenario(p);
    put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let large = put(&mut g, p, "Daring Thunder-Thief", Zone::Hand);
    let small = put(&mut g, p, "Healer's Hawk", Zone::Hand);
    let land = put(&mut g, p, "Forest", Zone::Hand);
    let spell = put(&mut g, p, "Opt", Zone::Hand);
    cast(&mut g, spell, None);
    resolve_top(&mut g);
    assert!(g.observe(p).effect_choice.is_some());
    assert!(g.observe(p.other()).effect_choice.is_none());
    let land = g.object_ref(land);
    reject(&mut g, Action::PutHandCreature { object: land });
    let large_ref = g.object_ref(large);
    let small_ref = g.object_ref(small);
    act(&mut g, Action::PutHandCreature { object: large_ref });
    assert_eq!(g.object_view(large).zone, Zone::Hand);
    assert!(g.observe(p.other()).effect_choice.is_none());
    act(&mut g, Action::PutHandCreature { object: small_ref });
    settle(&mut g);
    assert_eq!(g.object_view(large).zone, Zone::Battlefield);
    assert_eq!(g.object_view(small).zone, Zone::Battlefield);
    assert!(g.keywords(large).contains(Keywords::HEXPROOF));
    assert!(!g.keywords(small).contains(Keywords::HEXPROOF));
    assert_eq!(g.state.players[p.index()].life, 21);
}

const RETURN_LAND: CardRules = CardRules {
    effect: Effect::Land,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::ReturnToHandCountingOtherSubtype("Desert"),
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    kinds: 2,
                    source_only: false,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 1,
                optional_skip: None,
            },
            Instruction::DrawMoved,
        ],
    }],
    ..CardRules::EMPTY
};

#[test]
fn counted_untargeted_land_return_checks_actual_returned_permanents() {
    let _rules = fixture_rules::activate(&[(card("Abraded Bluffs"), &RETURN_LAND)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Abraded Bluffs", Zone::Hand);
    let desert = put(&mut g, p, "Bristling Backwoods", Zone::Battlefield);
    let forest = put(&mut g, p, "Forest", Zone::Battlefield);
    let opponent_land = put(&mut g, p.other(), "Creosote Heath", Zone::Battlefield);
    act(&mut g, Action::PlayLand { card: source });
    resolve_top(&mut g);
    let opposing = g.object_ref(opponent_land);
    reject(&mut g, Action::ChooseEffectObject { object: opposing });
    for (id, draws) in [(source, 0), (forest, 0), (desert, 1)] {
        let mut branch = g.clone();
        let object = branch.object_ref(id);
        act(&mut branch, Action::ChooseEffectObject { object });
        assert_eq!(branch.object_view(id).zone, Zone::Hand);
        assert_eq!(branch.state.players[p.index()].cards_drawn_this_turn, draws);
    }
}

const PLAINS: SearchSpec = SearchSpec {
    filter: SearchFilter::Subtype("Plains"),
    maximum: 1,
    destination: SearchDestination::Battlefield,
    tapped: true,
    reveal: false,
};
const SEARCH_TWICE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::SearchWithoutShuffle(PLAINS),
            Instruction::SearchWithoutShuffle(PLAINS),
            Instruction::ShuffleAfterSearches,
        ],
    ),
    ..CardRules::EMPTY
};

#[test]
fn deferred_searches_enter_each_find_before_one_final_shuffle() {
    let _rules = fixture_rules::activate(&[(card("Opt"), &SEARCH_TWICE)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let first = put(&mut g, p, "Plains", Zone::Library);
    let second = put(&mut g, p, "Plains", Zone::Library);
    let spell = put(&mut g, p, "Opt", Zone::Hand);
    let mut expected_rng = g.state.rng;
    expected_rng.shuffle(&mut [0; 8]);
    cast(&mut g, spell, None);
    resolve_top(&mut g);
    act(&mut g, Action::ChooseSearch { card: first });
    act(&mut g, Action::FinishSearch);
    assert_eq!(g.object_view(first).zone, Zone::Battlefield);
    assert!(g.object_view(first).tapped);
    assert_eq!(g.object_view(second).zone, Zone::Library);
    assert_ne!(g.state.rng, expected_rng);
    act(&mut g, Action::ChooseSearch { card: second });
    act(&mut g, Action::FinishSearch);
    assert_eq!(g.object_view(second).zone, Zone::Battlefield);
    assert!(g.object_view(second).tapped);
    assert_eq!(g.state.rng, expected_rng);
}

const LOOK_FOR_MOUNT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 1,
                count: LibrarySelectionCount::Fixed(1),
                filter: LibrarySelectionFilter::Subtype("Mount"),
                optional: true,
                reveal: true,
                remainder: LibraryRemainder::OptionalBottom,
            },
            look: LibraryLook::Fixed(1),
            destination: LibraryPickDestination::Hand,
            public: false,
            maximum_mana_value: None,
        })],
    ),
    ..CardRules::EMPTY
};

#[test]
fn declining_a_private_top_pick_allows_bottom_or_keep() {
    let _rules = fixture_rules::activate(&[(card("Opt"), &LOOK_FOR_MOUNT)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let mount = put(&mut g, p, "Bridled Bighorn", Zone::Library);
    let spell = put(&mut g, p, "Opt", Zone::Hand);
    cast(&mut g, spell, None);
    resolve_top(&mut g);
    assert!(g.observe(p).library_selection.is_some());
    assert!(g.observe(p.other()).library_selection.is_none());

    let mut take = g.clone();
    act(&mut take, Action::SelectLibrary { card: mount });
    act(&mut take, Action::FinishLibrarySelection);
    assert_eq!(take.object_view(mount).zone, Zone::Hand);

    act(&mut g, Action::FinishLibrarySelection);
    let mut keep = g.clone();
    act(&mut keep, Action::FinishLibrarySelection);
    assert_eq!(keep.state.players[p.index()].library.last(), Some(&mount));
    act(&mut g, Action::OrderLibraryBottom { card: mount });
    assert_eq!(g.state.players[p.index()].library.first(), Some(&mount));
}
