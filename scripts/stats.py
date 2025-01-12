import os
import subprocess
import pandas as pd
import json

def parse_cargo_output(output):
    """Parse the JSON output of the `cargo run stats <file>` command into a dictionary."""
    try:
        stats = json.loads(output)
        return stats
    except json.JSONDecodeError as e:
        print(f"Error decoding JSON: {e}")
        return {}

def strip_extension(filename):
    """Strip the extension from the filename."""
    return os.path.splitext(filename)[0]

def get_file_extension(filename):
    """Extract the file extension from the filename and strip the leading dot."""
    return os.path.splitext(filename)[1].lstrip('.')

def make_flags_combinations(flags):
    combinations = []
    for i in range(len(flags)):
        combinations.append([flag for j, flag in enumerate(flags) if j != i])
    return combinations

def collect_stats(base_directory, flag_sets):
    """Collect stats for all files in each catalog of the specified base directory."""
    data = []
    catalogs = ["rbg", "rg", "hrg", "kif"]
    for catalog in catalogs:
        catalog_path = os.path.join(base_directory, catalog)
        if not os.path.isdir(catalog_path):
            print(f"Catalog {catalog} not found in {base_directory}.")
            continue

        for filename in os.listdir(catalog_path):
            file_path = os.path.join(catalog_path, filename)
            if os.path.isfile(file_path):
                file_name_stripped = strip_extension(filename)
                file_extension = get_file_extension(filename)

                # file_name_stripped cant contain word Test or test
                if "test" in file_name_stripped.lower():
                    continue

                # Adjust flag sets for .hrg files
                current_flag_sets = []
                for flags in flag_sets:
                    if file_extension == "hrg":
                        current_flag_sets.append(flags + ["--reuse-functions"])
                    else:
                        current_flag_sets.append(flags)

                for flags in current_flag_sets:
                    try:
                        print(f"Running {file_path}...")
                        # Run the command and capture the output
                        result = subprocess.run(
                            ["cargo", "run", "--manifest-path", "interpreter_rust/Cargo.toml", "stats", "--json", file_path] + flags,
                            capture_output=True,
                            text=True,
                            check=True,
                            timeout=60
                        )
                        # Parse the command output
                        stats = parse_cargo_output(result.stdout)
                        stats['game'] = file_name_stripped
                        stats['language'] = file_extension
                        stats['flags'] = " ".join(flags) if flags else "none"
                        data.append(stats)
                    except subprocess.CalledProcessError as e:
                        print(f"Error running{file_path}: {e}")
                    except subprocess.TimeoutExpired as e:
                        print(f"Timeout expired running {file_path}: {e}")
                        break
    return pd.DataFrame(data)

def main():
    base_directory = "../games"
    if not os.path.isdir(base_directory):
        print("Invalid base directory.")
        return

    flags = [
        "--compact-comparisons",
        "--compact-skip-edges",
        "--inline-assignment",
        "--inline-reachability",
        "--join-exclusive-edges",
        "--join-fork-prefixes",
        "--join-fork-suffixes",
        "--join-generators",
        "--merge-accesses",
        "--normalize-constants",
        "--propagate-constants",
        "--prune-singleton-types",
        "--prune-unreachable-nodes",
        "--prune-unused-bindings",
        "--prune-unused-constants",
        "--prune-unused-variables",
        "--skip-generator-comparisons",
        "--skip-self-assignments",
        "--skip-self-comparisons",
        "--skip-unused-tags"
    ]
    
    # Define the default flag sets
    flag_sets = [
        ["--enable-all-optimizations"],
        []
    ]

    # All flags but one combinations
    flag_sets.extend(make_flags_combinations(flags))


    # Collect stats and save to a DataFrame
    df = collect_stats(base_directory, flag_sets)

    # Save to CSV or display
    output_csv = "stats_output.csv"
    df.to_csv(output_csv, index=False)
    print(f"Stats saved to {output_csv}")

if __name__ == "__main__":
    main()
