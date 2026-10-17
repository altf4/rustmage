# Reminder grammar audit

Issue #261 tracks whole-body reminder closure. The inventory in `reminder-audit-inventory.json` records 198 distinct parenthetical forms and their actual instruction contexts in the 1,811 supported Standard bodies at the PR #278 baseline. It is evidence for review and test selection. The compiler never reads this inventory, its observed strings, or its card names to authorize a card.

A reminder must belong to a parsed semantic production. Its location, subject, number, repeated cost, permanent type and token recipe must agree with that production. Recognizing familiar prose anywhere in a body is insufficient. Unknown or malformed residuals must reject the entire body; stripping them cannot authenticate their absence.

## Incremental implementation in issue #280

The first grammar phase protects complete fixed-stat intrinsic-keyword-only creature bodies and intrinsic-mana-only normal lands. A keyword reminder must follow its own keyword on the same line. Its grammar describes that keyword; putting the reach explanation after flying, moving it to a separate paragraph, nesting it, or splitting a keyword with parentheses rejects the whole card. Multiple keywords still compile with their actual combined keyword mask.

A land's complete intrinsic mana reminder must agree with its actual basic land subtypes. A Forest cannot claim blue intrinsic production, and a two-type Island Forest must produce the corresponding alternatives rather than both mana simultaneously. The grammar also rejects extra, unknown and nested parentheticals. Names do not affect these decisions.

Parenthetical erasure is used only to detect whether a complete body belongs to this semantic route. The original raw body must then pass authentication. A final rejection guard prevents later generic templates or legacy fixtures from reviving the failed body. This protects the scoped routes without replacing unknown text with executable normalized text.

The entire generated catalogue, colors and helper/recipe tables remain unchanged apart from the fingerprint. More complicated bodies that contain additional costs, triggers, activations, scaling rules or keyword grants remain in the broader audit below. A creature-template provenance label alone does not imply that its original whole body contains only keywords: extractors may have consumed additional mechanics first.

## Admission-site audit

The baseline AST inventory has 26 parenthesis-related regex sites. Twenty use the broad innermost-parenthetical pattern; the table distinguishes their roles rather than claiming every occurrence is an unsafe admission route.

| Functions | Current guard and remaining work |
| --- | --- |
| `creature_template` | Whole-card intrinsic route now authenticated. Extracted/proxy bodies still depend on their caller authenticating consumed mechanics and reminders. |
| `land_template` | Leading intrinsic mana already checks actual basic subtypes. The new guard closes complete intrinsic-only bodies; lands with entry rules or activated abilities still require full reminder authentication. |
| `extract_uncounterable` | Retains unconsumed original lines, but erases parentheses on a recognized prohibition line. Authenticate its optional Ward explanation before consuming that line. |
| `extract_creature_entry_tapped` | Retains unconsumed lines; a recognized entry clause can currently lose unknown parentheticals. Bind any stun explanation to the same exact entry/counter semantics. |
| `extract_stat_rules` | Consumed scaling/characteristic clauses need their own grammar; raw reminders must not disappear while extracting the recognized clause. |
| `static_permanent_template` | Whole-body normalization needs authenticated keyword grants, predicates, type references and semantic action reminders. |
| `evidence_text` | The fixed-evidence body matcher normalizes parentheticals. Bind repeated thresholds, collection costs, subjects and optional branches before recognizing the body. |
| `scalar_x_body` | Generic scalar bodies normalize reminders; the strict newer selected-face route has additional raw checks, but those do not authenticate all inherited scalar bodies. |
| `trigger_creature_template` | Authenticate keyword, action, counter, token and trigger explanations in their exact instruction context. |
| `spell_program_template`, `modal_spell_template` | Every mode/instruction must compile fully, but reminder normalization can still discard unknown clauses. Bind the reminder to the instruction it explains. |
| `aura_template`, `equipment_template` | Attachment Ward routes have explicit raw reminder checks. Other attachment bodies, keyword grants, equip costs and token/job mechanics need their own complete grammar audit. |
| `permanent_program_template` | Activated/triggered/intrinsic residual bodies depend on generic normalization. Some specialized selectors later enforce stricter raw syntax; check every path rather than assuming the wrapper applies universally. |
| `cycling_abilities` | Parse the printed cost, search subtype and reminder together; an unrelated or changed payment/search reminder cannot be silently removed. |
| `endure_body`, `mobilize_body`, `saddle_body` | Match repeated counts and affected subjects/token recipes against the mechanic declaration before accepting the whole body. |
| `attachment_ward_route` | Detection only: erased text locates a possible route; `attachment_ward_reminders_valid` authenticates original syntax. Preserve this distinction. |
| `compile_catalogue` timing scan | Detection only, including intermediate nested-erasure stages. Raw timing grammar remains authoritative; do not replace it with the detected text. |

The remaining six AST sites include authenticated Adventure/Omen wrappers and generated-Rust token-factory rewrites. The latter manipulate compiler output, not Oracle reminder input. Saga wrappers and several additional-cost mechanics already parse exact repeated parameters without broad erasure; preserve those checks during composition.

## Semantic families and next grammar work

The inventory assigns every observed form to one semantic family. Related families can share typed grammar while retaining their distinct instruction contexts.

| Family group | Required semantic binding |
| --- | --- |
| Intrinsic keywords and grants | Keyword, source/recipient, intrinsic versus granted ability, reminder position and allowed subject pronoun. |
| Named tokens and investigate | Created token type, count/recipient, exact activation costs, sacrifice/tap requirements and effect recipe. A Food reminder cannot describe a Treasure or change its life gain. |
| Scry/surveil, mill, explore, connive, fight | Action kind, count, optionality, target/controller, zone/destination and repeated instruction. `Scry 2` cannot carry a top-one-card explanation. |
| Stun and poison counters; modified/outlaw predicates | Counter kind, threshold/removal event, actual type-set predicate and referenced subject. |
| Evidence, forage, bargain, convoke, affinity | Paid/consumed resources, repeated threshold or discount, legal resource filter and mandatory/optional casting context. |
| Cycling, flashback, warp, kicker, offspring, plot | Repeated mana cost and additional resources, actual search subtype, token-copy recipe, cast origin, timing and eventual destination. Existing exact grammar remains authoritative. |
| Equip, job select, crew, saddle | Repeated cost/power threshold, source/recipient type, tap/sacrifice requirements and timing restrictions. |
| Ward, prowess, increment, firebending, mobilize, exhaust | Trigger event, relevant player/subject, repeated amount or cost, once restriction, mana lifetime and token/delayed-sacrifice recipe. |
| Saga, Adventure and Omen | Whole physical-card type/face/chapter structure, exact owner/caster destination and timing. Preserve their existing strict wrappers. |

## Required positive and negative controls

For each future production, use genuine published contexts plus independently renamed semantic equivalents. Check its emitted program and parameters, not just an `implemented` flag. Negative controls must include unknown parenthetical sentences, altered known explanations, nested and unbalanced groups, an insertion inside a keyword/action, familiar prose stolen from another mechanic, moved reminders, wrong recipient/owner, wrong repeated counts or costs, and wrong token/type context.

Every phase needs canonical preservation of all admitted definitions, helper arms, colors, recipes and emblems, or a specific report of a prior incomplete admission being corrected or gated. Keep the full compiler suite and relevant native/public rules checks scoped to the exact source and immutable library fingerprints. Issue #261 stays open until every actual admission path has closed grammar.
