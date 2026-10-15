#!/usr/bin/env python3
"""Freeze Scryfall's current Standard identities and rules metadata, never images.

The output is input data, not evidence of implemented card semantics. Run an
explicit reviewed update when the format changes; normal engine runs are offline.
"""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import time
import urllib.parse
import urllib.request

API = "https://api.scryfall.com/cards/search"
FIELDS = ("oracle_id", "name", "layout", "mana_cost", "cmc", "type_line", "oracle_text",
          "power", "toughness", "loyalty", "defense", "colors", "color_identity",
          "keywords", "produced_mana", "card_faces", "all_parts")
FACE_FIELDS = ("name", "mana_cost", "type_line", "oracle_text", "power", "toughness",
               "loyalty", "defense", "colors", "color_indicator")


def validate_rules_text(row):
    if row.get('layout') == 'normal' and not isinstance(row.get('oracle_text'), str):
        raise ValueError(f"normal card is missing explicit rules text: {row.get('name')}")
    for face in row.get('card_faces', []):
        if not isinstance(face.get('oracle_text'), str):
            raise ValueError(f"card face is missing explicit rules text: {face.get('name')}")


def fetch(as_of):
    # Date-filter printings as well: unique=cards can otherwise choose an
    # unreleased reprint of a card already legal in Standard.
    source = API + "?" + urllib.parse.urlencode({
        "q": f"legal:standard date<={as_of}", "unique": "cards", "order": "name",
    })
    url = source
    cards = []
    expected_total = None
    pages = 0
    while url:
        parsed = urllib.parse.urlparse(url)
        if parsed.scheme != "https" or parsed.netloc != "api.scryfall.com":
            raise ValueError("unexpected pagination host")
        request = urllib.request.Request(url, headers={
            "User-Agent": "rustmage/0.1 (https://github.com/altf4/rustmage; Standard migration)",
            "Accept": "application/json",
        })
        with urllib.request.urlopen(request, timeout=60) as response:
            page = json.load(response)
        if page.get("object") != "list" or page.get("warnings"):
            raise ValueError(f"unexpected API response: {page.get('warnings')}")
        if expected_total is None:
            expected_total = page["total_cards"]
        elif page["total_cards"] != expected_total:
            raise ValueError("format changed while paging; retry as a fresh snapshot")
        for row in page["data"]:
            validate_rules_text(row)
            if row["legalities"]["standard"] != "legal" or row.get("digital", False):
                raise ValueError(f"non-tabletop Standard card in response: {row['name']}")
            if row["released_at"] > as_of:
                raise ValueError(f"API includes cards after requested snapshot date: {row['name']}")
            card = {key: row[key] for key in FIELDS if key in row}
            if "card_faces" in card:
                card["card_faces"] = [{key: face[key] for key in FACE_FIELDS if key in face}
                                      for face in card["card_faces"]]
            if "all_parts" in card:
                card["all_parts"] = [{key: part[key] for key in ("id", "component", "name", "type_line") if key in part}
                                     for part in card["all_parts"]]
            card["reference_printing"] = {key: row[key] for key in ("id", "set", "collector_number", "released_at")}
            cards.append(card)
        pages += 1
        url = page.get("next_page") if page["has_more"] else None
        if page["has_more"] and not url:
            raise ValueError("missing pagination link")
        if url:
            time.sleep(0.12)
    if len(cards) != expected_total:
        raise ValueError("pagination count mismatch")
    cards.sort(key=lambda card: card["name"])
    if len({card["oracle_id"] for card in cards}) != len(cards):
        raise ValueError("duplicate oracle identity")
    if len({card["name"] for card in cards}) != len(cards):
        raise ValueError("duplicate name")
    payload = json.dumps(cards, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()
    return {
        "schema": 1, "as_of": as_of, "source": source, "pages": pages,
        "card_count": len(cards), "cards_sha256": hashlib.sha256(payload).hexdigest(),
        "status": "legal-card data snapshot; engine implementation coverage is separate",
        "cards": cards,
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--as-of", required=True, type=lambda value: datetime.date.fromisoformat(value).isoformat())
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    result = fetch(args.as_of)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({key: value for key, value in result.items() if key != "cards"}))
