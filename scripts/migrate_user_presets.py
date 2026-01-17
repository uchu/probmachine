#!/usr/bin/env python3
"""
Migrate user presets to reset LFO destinations in bank A.
This handles the modulation destination index changes from the recent update.
"""

import json
import os
import shutil
from datetime import datetime

def get_user_presets_path():
    """Get the path to user presets file."""
    home = os.path.expanduser("~")
    return os.path.join(home, "Library", "Application Support", "Device", "user_presets.json")

def backup_presets(path):
    """Create a backup of the user presets file."""
    if os.path.exists(path):
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        backup_path = path.replace(".json", f"_backup_{timestamp}.json")
        shutil.copy2(path, backup_path)
        print(f"Backup created: {backup_path}")
        return backup_path
    return None

def reset_lfo_destinations_bank_a(data):
    """Reset LFO destinations to 0 for all presets in bank A (first bank)."""
    if "banks" not in data:
        print("No 'banks' key found in user presets")
        return False

    banks = data["banks"]
    if len(banks) == 0:
        print("No banks found")
        return False

    # Bank A is the first bank (index 0)
    bank_a = banks[0]
    presets = bank_a.get("presets", [])

    modified_count = 0
    for preset in presets:
        preset_data = preset.get("data", {})
        modified = False

        # Reset LFO1 destinations
        for key in ["lfo1_dest1", "lfo1_dest2", "lfo2_dest1", "lfo2_dest2", "lfo3_dest1", "lfo3_dest2"]:
            if key in preset_data and preset_data[key] != 0:
                preset_data[key] = 0
                modified = True

        if modified:
            modified_count += 1
            print(f"  Reset LFO destinations in: {preset.get('name', 'Unknown')}")

    print(f"Modified {modified_count} presets in Bank A")
    return True

def main():
    path = get_user_presets_path()

    if not os.path.exists(path):
        print(f"User presets file not found: {path}")
        return

    print(f"Loading user presets from: {path}")

    # Create backup
    backup_path = backup_presets(path)
    if not backup_path:
        print("Warning: Could not create backup")

    # Load presets
    with open(path, 'r') as f:
        data = json.load(f)

    # Migrate bank A
    if reset_lfo_destinations_bank_a(data):
        # Save updated presets
        with open(path, 'w') as f:
            json.dump(data, f, indent=2)
        print(f"User presets updated successfully")
    else:
        print("No migration needed or migration failed")

if __name__ == "__main__":
    main()
