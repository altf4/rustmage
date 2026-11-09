//! Ordinary library-origin casting and land play use live, exact permissions.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const LOOK: StaticAbilityGroup = StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::LookAtLibraryTop],
};

const BIRD_GROUPS: &[StaticAbilityGroup] = &[
    LOOK,
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[
            StaticAbilityFragment::PlayLandFromLibraryTop,
            StaticAbilityFragment::CastFromLibraryTop(CastFilter::Subtype("Bird")),
        ],
    },
];
const BIRD: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: BIRD_GROUPS,
    extras: &CardExtras {
        static_ability_groups: BIRD_GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const AFTER_ATTACK_GROUPS: &[StaticAbilityGroup] = &[
    LOOK,
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::PlayFromLibraryTopAfterAttacking],
    },
];
const AFTER_ATTACK: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: AFTER_ATTACK_GROUPS,
    extras: &CardExtras {
        static_ability_groups: AFTER_ATTACK_GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const ARTIFACT_GROUPS: &[StaticAbilityGroup] = &[
    LOOK,
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CastFromLibraryTop(
            CastFilter::Artifact,
        )],
    },
];
const ARTIFACT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: ARTIFACT_GROUPS,
    extras: &CardExtras {
        static_ability_groups: ARTIFACT_GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn can_cast(g: &Game, id: CardId) -> bool {
    g.legal_actions().iter().any(|action| {
        matches!(action, Action::Cast { card, .. } | Action::AnnounceCast { card } if *card == id)
    })
}

#[test]
fn bird_permission_filters_the_chosen_face_and_uses_the_actual_or_projected_top() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &BIRD)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let provider = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let bird = put(&mut g, p, "Most Decrepit Old Bird", Zone::Library);
        let cover = put(&mut g, p, "Bear Cub", Zone::Library);
        let cover_cast = g.cast_departure(g.object_ref(cover), p).unwrap();
        assert!(!can_cast(&g, bird));
        assert!(!can_cast(&g, cover));
        assert!(cast_origins::library_spell_origin_available(
            &cover_cast,
            p,
            bird,
            g.object_view(bird).card
        ));
        assert_eq!(
            g.observe(p).known_library_top[p.index()].unwrap().object,
            g.object_ref(cover)
        );
        assert!(g.observe(p.other()).known_library_top[p.index()].is_none());

        g.move_object(cover, Zone::Exile);
        reject(&mut g, Action::AnnounceAdventure { card: bird });
        assert!(can_cast(&g, bird));
        let mana = g.state.players[p.index()].mana[1];
        cast(&mut g, bird, None);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana[1], mana - 1);
        assert_eq!(g.object_view(bird).zone, Zone::Battlefield);
        assert_eq!(g.object_view(bird).counters.plus_one, 0);

        let land = put(&mut g, p, "Forest", Zone::Library);
        act(&mut g, Action::PlayLand { card: land });
        let second = put(&mut g, p, "Forest", Zone::Library);
        reject(&mut g, Action::PlayLand { card: second });

        // Changeling applies even on a kindred noncreature spell.
        let changeling = put(&mut g, p, "Crib Swap", Zone::Library);
        assert!(can_cast(&g, changeling));
        g.move_object(provider, Zone::Graveyard);
        assert!(!can_cast(&g, changeling));
        assert!(g.observe(p).known_library_top[p.index()].is_none());
    }
}

#[test]
fn normal_bird_permission_remains_distinct_from_the_entry_counter_route() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &BIRD)]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    put(&mut g, p, "Mikey & Don, Party Planners", Zone::Battlefield);
    let duck = put(&mut g, p, "Turtle-Duck", Zone::Library);
    act(&mut g, Action::AnnounceCast { card: duck });
    let receipt = g.snapshot();

    for with_counter in [false, true] {
        g.restore(&receipt).unwrap();
        act(&mut g, Action::ChooseLibraryCastPermission { with_counter });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(duck).zone, Zone::Battlefield);
        assert_eq!(
            g.object_view(duck).counters.plus_one,
            u64::from(with_counter)
        );
    }
}

#[test]
fn a_matching_modal_back_can_be_cast_without_permission_for_its_front() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &ARTIFACT)]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let modal = put(
        &mut g,
        p,
        "Tony Stark // The Invincible Iron Man",
        Zone::Library,
    );
    assert!(!can_cast(&g, modal));
    act(&mut g, Action::AnnounceBack { card: modal });
    let cast = g.state.announcing.as_ref().unwrap().item.cast.unwrap();
    assert_eq!(cast.face, CastFace::Back);
    assert_eq!(cast.origin, Zone::Library);
}

#[test]
fn declaring_the_exact_attacker_enables_paid_library_spells_and_one_normal_land_play() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &AFTER_ATTACK)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let provider = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let top = put(&mut g, p, "Ishgard, the Holy See", Zone::Library);
        let land = Action::PlayLand { card: top };
        let adventure = Action::AnnounceAdventure { card: top };
        reject(&mut g, land.clone());
        reject(&mut g, adventure.clone());

        // Being in combat is not a declaration, for example after entering attacking.
        g.state.attackers.push(AttackDeclaration {
            card: provider,
            defender: Some(CombatDefender::Player(p.other())),
            blocked: false,
        });
        reject(&mut g, land.clone());
        g.state.attackers.clear();
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: provider });
        act(&mut g, Action::FinishAttackers);
        reject(&mut g, land.clone());
        reject(&mut g, adventure.clone());

        // The relevant attack was committed through normal actions; skip the
        // remainder of combat to sample the same turn's ordinary main-phase play.
        g.state.attackers.clear();
        g.state.step = Step::Main2;
        g.state.priority = p;
        assert!(g.legal_actions().contains(&adventure));
        let receipt = g.snapshot();
        let revision = g.revision();
        let index = g.legal_actions().iter().position(|a| a == &land).unwrap();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        let mut returned = g.clone();
        drop(g);

        raw.apply(p, land.clone()).unwrap();
        indexed.apply_index(p, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.object_view(top).zone, Zone::Battlefield);
        assert!(raw.object_view(top).tapped);
        assert!(raw.observe(p.other()).known_library_top[p.index()].is_none());
        let next = raw.top_library_card(p).unwrap();
        reject(&mut raw, Action::PlayLand { card: next });

        indexed.restore(&receipt).unwrap();
        assert_eq!(
            indexed.apply_index(p, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, adventure.clone());
        let cast = indexed
            .state
            .announcing
            .as_ref()
            .unwrap()
            .item
            .cast
            .unwrap();
        assert_eq!(cast.origin, Zone::Library);
        assert_eq!(cast.face, CastFace::Adventure);
        assert!(indexed.observe(p).known_library_top[p.index()].is_none());
        assert!(indexed.observe(p.other()).known_library_top[p.index()].is_none());
        if indexed.legal_actions().contains(&Action::FinishTargets) {
            act(&mut indexed, Action::FinishTargets);
        }
        pay(&mut indexed);
        assert!(indexed.observe(p).known_library_top[p.index()].is_some());
        settle(&mut indexed);
        assert_eq!(indexed.object_view(top).zone, Zone::Exile);
        assert!(indexed.legal_actions().contains(&land));

        let old_provider = returned.object_ref(provider);
        returned.move_object(provider, Zone::Exile);
        returned.move_object(provider, Zone::Battlefield);
        assert_ne!(returned.object_ref(provider), old_provider);
        reject(&mut returned, land);
        reject(&mut returned, adventure);
        assert_eq!(
            returned.observe(p).known_library_top[p.index()]
                .unwrap()
                .object,
            returned.object_ref(top)
        );
    }
}
