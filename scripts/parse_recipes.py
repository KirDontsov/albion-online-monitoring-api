#!/usr/bin/env python3
"""Parse ao-bin-dumps items.json to extract artefact crafting recipes."""

import json
import sys
from pathlib import Path


def normalize_cr(cr):
    """craftingrequirements can be dict or list"""
    if isinstance(cr, dict):
        return [cr]
    if isinstance(cr, list):
        return cr
    return []


def normalize_resources(resources):
    """craftresource can be dict or list"""
    if isinstance(resources, dict):
        return [resources]
    if isinstance(resources, list):
        return resources
    return []


ENCHANT_SUFFIX = {0: "", 1: "@1", 2: "@2", 3: "@3", 4: "@4"}


def parse_recipes(items_data):
    recipes = []
    resource_ids = set()

    for category in ["weapon", "equipmentitem"]:
        items = items_data.get(category, [])
        if isinstance(items, dict):
            items = [items]

        for item in items:
            item_id = item.get("@uniquename", "")
            if not item_id:
                continue

            # Base crafting requirements
            base_cr = normalize_cr(item.get("craftingrequirements", []))

            # Enchantment crafting requirements
            ench_data = item.get("enchantments", {})
            ench_list = []
            if isinstance(ench_data, dict):
                ench_list = ench_data.get("enchantment", [])
                if isinstance(ench_list, dict):
                    ench_list = [ench_list]

            # Process base recipe
            for cr in base_cr:
                resources = normalize_resources(cr.get("craftresource", []))
                artefact_id = None
                other_resources = []

                for res in resources:
                    uid = res.get("@uniquename", "")
                    count = res.get("@count", "1")

                    if "ARTEFACT" in uid and "TOKEN" not in uid:
                        artefact_id = uid
                    elif "ARTEFACT" not in uid and "TOKEN" not in uid:
                        other_resources.append({"item_id": uid, "count": count})
                        resource_ids.add(uid)

                if artefact_id:
                    recipes.append({
                        "item_id": item_id,
                        "artefact_id": artefact_id,
                        "resources": other_resources,
                    })
                    break  # Take first recipe with artefact

            # Process enchantment recipes
            for ench_idx, ench in enumerate(ench_list):
                ench_level = ench_idx + 1  # 1-based
                suffix = ENCHANT_SUFFIX.get(ench_level, f"@{ench_level}")
                ench_item_id = f"{item_id}{suffix}"

                ench_cr_list = normalize_cr(ench.get("craftingrequirements", []))
                for cr in ench_cr_list:
                    resources = normalize_resources(cr.get("craftresource", []))
                    artefact_id = None
                    other_resources = []

                    for res in resources:
                        uid = res.get("@uniquename", "")
                        count = res.get("@count", "1")

                        if "ARTEFACT" in uid and "TOKEN" not in uid:
                            artefact_id = uid
                        elif "ARTEFACT" not in uid and "TOKEN" not in uid:
                            other_resources.append({"item_id": uid, "count": count})
                            resource_ids.add(uid)

                    if artefact_id:
                        recipes.append({
                            "item_id": ench_item_id,
                            "artefact_id": artefact_id,
                            "resources": other_resources,
                        })
                        break  # Take first recipe with artefact

    return recipes, sorted(resource_ids)


def main():
    data_dir = Path(__file__).parent.parent / "data"
    input_file = data_dir / "items.json"
    output_recipes = data_dir / "item_recipes.json"
    output_resources = data_dir / "resource_ids.json"

    if not input_file.exists():
        print(f"Error: {input_file} not found")
        sys.exit(1)

    with open(input_file) as f:
        data = json.load(f)

    items_data = data.get("items", {})
    recipes, resource_ids = parse_recipes(items_data)

    with open(output_recipes, "w") as f:
        json.dump(recipes, f, indent=2)

    with open(output_resources, "w") as f:
        json.dump(resource_ids, f, indent=2)

    print(f"Parsed {len(recipes)} recipes")
    print(f"Found {len(resource_ids)} unique resource IDs")
    print(f"Saved to {output_recipes}")
    print(f"Resource IDs saved to {output_resources}")


if __name__ == "__main__":
    main()
