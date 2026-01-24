#!/usr/bin/env python3
import json
import sys
from pathlib import Path

def main():
    user_presets_path = Path.home() / "Library" / "Application Support" / "Device" / "user_presets.json"
    factory_bank_a_path = Path(__file__).parent.parent / "assets" / "presets" / "factory_bank_a.json"

    if not user_presets_path.exists():
        print(f"Error: User presets file not found at {user_presets_path}")
        sys.exit(1)

    with open(user_presets_path, 'r') as f:
        user_data = json.load(f)

    if not factory_bank_a_path.exists():
        print(f"Error: Factory bank A file not found at {factory_bank_a_path}")
        sys.exit(1)

    with open(factory_bank_a_path, 'r') as f:
        factory_bank_a = json.load(f)

    user_bank_a = user_data['banks'][0]
    user_presets_1_12 = user_bank_a['presets'][:12]

    print(f"Copying {len(user_presets_1_12)} presets from user bank A to factory bank A")

    for i, preset in enumerate(user_presets_1_12, 1):
        print(f"  {i}. {preset['name']}")
        preset['author'] = "Device"

    factory_bank_a['presets'][:12] = user_presets_1_12

    with open(factory_bank_a_path, 'w') as f:
        json.dump(factory_bank_a, f, indent=2)

    print(f"\nSuccessfully updated {factory_bank_a_path}")
    print("Factory bank A presets 1-12 have been replaced with user bank A presets 1-12")

if __name__ == '__main__':
    main()
