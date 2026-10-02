#!/usr/bin/env python3
"""Static release assertions; no generated-file mutation or live infrastructure."""
import json
import pathlib
import re
import tomllib

ROOT = pathlib.Path(__file__).resolve().parents[1]
CONTRACTS = [
    "docs/contracts/storyscript_localization_v1.md",
    "docs/contracts/storybundle_v1.md",
    "docs/contracts/storyplayer_save_v1.md",
    "docs/feature/storyscript_localization.md",
    "docs/onboarding/localized_story_project_checklist.md",
    "docs/playbook/localization_release_recovery.md",
    "docs/qa-docs/storyscript_localization_v1_qa.md",
]


def main():
    for relative in CONTRACTS:
        assert (ROOT / relative).is_file(), relative
    combined = "\n".join((ROOT / p).read_text() for p in CONTRACTS)
    for phrase in ["Fluent", "message ID", "default locale", "supported locale",
                   "catalog", "locale-neutral", "no migration", "not encrypted"]:
        assert phrase.lower() in combined.lower(), f"missing normative phrase: {phrase}"
    stale = re.compile(r"archive format remains unchanged|resolved strings only|"
                       r"locale selection is outside|translations belong to Flutter|"
                       r"v1 schema remains unchanged")
    for doc in (ROOT / "docs/feature").glob("*.md"):
        assert not stale.search(doc.read_text()), f"stale localization claim: {doc.name}"
    for relative in ["bundle/rust/Cargo.toml", "player/Cargo.toml"]:
        dependencies = tomllib.loads((ROOT / relative).read_text())["dependencies"]
        for crate, version in {"fluent": "=0.17.0", "fluent-syntax": "=0.12.0"}.items():
            value = dependencies[crate]
            assert (value if isinstance(value, str) else value["version"]) == version, (relative, crate)
    for owner in ["bundle/rust", "bundle/rust/fuzz", "player", "storyscript_player_core/rust", "storyscript_bundle/rust"]:
        packages = tomllib.loads((ROOT / owner / "Cargo.lock").read_text())["package"]
        for crate, version in {"fluent": "0.17.0", "fluent-syntax": "0.12.0", "unic-langid": "0.9.6", "icu_locale_core": "2.1.1"}.items():
            assert {p["version"] for p in packages if p["name"] == crate} == {version}, (owner, crate)
        assert not any(p["name"] == "rkyv" and p["version"].startswith("0.7.") for p in packages), f"obsolete vulnerable optional rkyv lock entry: {owner}"
    package = json.loads((ROOT / "tool/vscode-storyscript/package.json").read_text())
    lock = json.loads((ROOT / "tool/vscode-storyscript/package-lock.json").read_text())
    assert package["dependencies"]["@fluent/syntax"] == "0.19.0"
    assert lock["packages"]["node_modules/@fluent/syntax"]["version"] == "0.19.0"
    for relative in ["bundle/proto/storybundle/v1/schema.sha256", "player/proto/storyplayer/v1/schema.sha256"]:
        assert re.fullmatch(r"[0-9a-f]{64}\s*", (ROOT / relative).read_text()), relative
    en = json.loads((ROOT / "storyscript_bundle/example/lib/l10n/app_en.arb").read_text())
    translated = json.loads((ROOT / "storyscript_bundle/example/lib/l10n/app_id.arb").read_text())
    keys = lambda catalog: {k for k in catalog if not k.startswith("@")}
    assert keys(en) == keys(translated), "incomplete Flutter shell translation"
    for key in keys(en):
        assert set(re.findall(r"\{(\w+)\}", en[key])) == set(re.findall(r"\{(\w+)\}", translated[key])), key
    game_key = (ROOT / "storyscript_bundle/example/assets/station_nine_public_key.txt").read_text().strip()
    demo_key = (ROOT / "storyscript_bundle/example/assets/demo_public_key.txt").read_text().strip()
    assert game_key != demo_key and all(re.fullmatch(r"[0-9a-f]{64}", k) for k in [game_key, demo_key])
    assert not (ROOT / "docs/contracts/api.yaml").exists(), "no HTTP API introduced"
    print("Localization documentation, dependency, descriptor and shell-resource gates passed")


if __name__ == "__main__":
    main()
