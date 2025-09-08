import os
import subprocess
import pandas as pd
import json


def strip_extension(filename):
    """Strip the extension from the filename."""
    return os.path.splitext(filename)[0]

def get_file_extension(filename):
    """Extract the file extension from the filename and strip the leading dot."""
    return os.path.splitext(filename)[1].lstrip('.')

def make_flags_combinations(flags):
    combinations = []
    for i in range(len(flags)):
        combinations.append([[flag for j, flag in enumerate(flags) if j != i], flags[i]])
    return combinations

def process_output(output):
    output = output.split()
    return int(output[1])


def collect_stats(base_directory, flag_sets):
    """Collect stats for all files in each catalog of the specified base directory."""
    stats_data = []
    catalogs = ["rbg", "hrg", "kif"]
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

                for flags in flag_sets:
                    try:
                        print(f"Running {file_path}...")
                        # Run the command and capture the output
                        result = subprocess.run(
                            ["cargo", "run", "--manifest-path", "../../interpreter_rust/Cargo.toml", "plays",  file_path, "10"] + flags[0],
                            capture_output=True,
                            text=True,
                            check=True,
                            timeout=180
                        )
                        print(result.stdout)
                        # Parse the command output
                        output = result.stdout
                        stats = {}
                        stats["plays"] = process_output(output)
                        stats['game'] = file_name_stripped
                        stats['language'] = file_extension
                        stats['flags'] = flags[1]
                        print(stats)
                        stats_data.append(stats)
                    except subprocess.CalledProcessError as e:
                        print(f"Error running{file_path}: {e}")
                    except subprocess.TimeoutExpired as e:
                        print(f"Timeout expired running {file_path}: {e}")
                        break
    return (pd.DataFrame(stats_data))

def main():
    base_directory = "../../games"
    if not os.path.isdir(base_directory):
        print("Invalid base directory.")
        return

    # Define the default flag sets
    flag_sets = [
        [["--enable-all-optimizations"], "--enable-all-optimizations"],
        [[], "none"]
    ]


    # Collect stats and save to a DataFrame
    stats_data = collect_stats(base_directory, flag_sets)

    # Save to CSV or display
    stats_output_csv = "results/runtime_stats.csv"
    stats_data.to_csv(stats_output_csv, index=False)
    print(f"Stats saved to {stats_output_csv}")

if __name__ == "__main__":
    main()
