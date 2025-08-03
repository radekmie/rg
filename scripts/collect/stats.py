import os
import subprocess
import pandas as pd
import json 
   
def makeGamesList():
    games = []
    games.append('alquerque.py')
    games.append('alquerque_lud.py')
    games.append('amazons.hrg')
    games.append('amazons_split2.hrg')
    games.append('ataxx.hrg')
    games.append('backgammon.hrg')
    #games.append('battleships.hrg')
    games.append('bombardment.hrg')
    games.append('breakthrough.hrg')
    games.append('chess.hrg')
    games.append('chess_kingCapture.hrg')
    games.append('clobber.hrg')
    games.append('connect4.hrg')
    games.append('dashGuti.py')
    games.append('dashGuti_lud.py')
    games.append('dotsAndBoxes.hrg')
    games.append('englishDraughts.hrg')
    games.append('foxAndGeese.hrg')
    games.append('foxAndGeese_lud.hrg')
    games.append('golSkuish.py')
    games.append('golEkuish_lud.py')
    games.append('gomoku_standard.hrg')
    games.append('knightthrough.hrg')
    games.append('lauKataKati.py')
    games.append('lauKataKati_lud.py')
    games.append('oware.hrg')
    games.append('pentago.hrg')
    games.append('pentago_split.hrg')
    games.append('pretwa.py')
    games.append('pretwa_lud.py')
    games.append('ticTacDie.hrg')
    #games.append('twentyOne.hrg')
    games.append('ultimateTicTacToe.hrg')
    
    games.append('alquerque.rbg')
    games.append('alquerque_lud.rbg')
    games.append('amazons.rbg')
    games.append('amazons_split2.rbg')
    games.append('breakthrough.rbg')
    # games.append('chessGardner5x5_kingCapture.rbg')
    # games.append('chessLosAlamos6x6_kingCapture.rbg')
    # games.append('chessQuick5x6_kingCapture.rbg')
    # games.append('chessSilverman4x5_kingCapture.rbg')
    games.append('chess.rbg')
    games.append('chess_kingCapture.rbg')
    games.append('connect4.rbg')
    games.append('dashGuti.rbg')
    games.append('englishDraughts.rbg')
    games.append('foxAndHounds.rbg')
    games.append('golSkuish.rbg')
    games.append('gomoku_standard.rbg')
    games.append('hex.rbg')
    games.append('knightthrough.rbg')
    games.append('lauKataKati.rbg')
    games.append('pentago.rbg')
    games.append('pentago_split.rbg')
    games.append('pretwa.rbg')
    games.append('reversi.rbg')
    # games.append('skirmish.rbg')
    games.append('surakarta.rbg')
    games.append('theMillGame.rbg')
    games.append('theMillGame_lud.rbg')
    games.append('yavalath.rbg')
    return games

def split_output(output):
    # find line starting with STATS
    stats_start = output.find("STATS")
    transforms = output[:stats_start].strip().split("\n")
    stats_output = output[(stats_start + 5):].strip()
    return transforms, stats_output

def parse_transforms(transforms_output):
    """Each transform line is in the format: '<name> <time> <did_change>'"""
    parsed_transforms = []
    start = transforms_output.find("add_builtins")
    transforms = transforms_output[start:].strip().split("\n")
    for transform in transforms:
        parts = transform.split()
        if len(parts) < 3:
            continue
        name = parts[0]
        time = float(parts[1])
        did_change = parts[2] == "true"
        parsed_transforms.append({
            'name': name,
            'time': time,
            'did_change': did_change
        })
    return parsed_transforms

def process_transforms(transforms):
    """Count how many times each transform was applied, how many times it changed the game, average time taken, and total time taken."""
    transform_stats = {}
    for transform in transforms:
        name = transform['name']
        if name not in transform_stats:
            transform_stats[name] = {
                'count': 0,
                'changed': 0,
                'total_time': 0.0,
                'changed_time': 0.0
            }
        transform_stats[name]['count'] += 1
        if transform['did_change']:
            transform_stats[name]['changed'] += 1
            transform_stats[name]['changed_time'] += transform['time']
        transform_stats[name]['total_time'] += transform['time']
    
    # Convert to a DataFrame
    return pd.DataFrame.from_dict(transform_stats, orient='index').reset_index().rename(columns={'index': 'transform'})

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
        combinations.append([[flag for j, flag in enumerate(flags) if j != i], flags[i]])
    return combinations

def collect_stats(base_directory, flag_sets, games):
    """Collect stats for all files in each catalog of the specified base directory."""
    stats_data = []
    transforms_data = pd.DataFrame()
    catalogs = ["rbg", "hrg", "kif", "rg"]
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
                if filename not in games:
                    print(f"Skipping {filename} as it is not in the predefined games list.")
                    continue
                # file_name_stripped cant contain word Test or test
                if "test" in file_name_stripped.lower():
                    continue

                file_stats = []
                file_transforms = pd.DataFrame()
                all_flags_succeeded = True

                for flags in flag_sets:
                    try:
                        print(f"Running {file_path}...")
                        # Run the command and capture the output
                        result = subprocess.run(
                            ["cargo", "run", "--manifest-path", "../../interpreter_rust/Cargo.toml", "stats", "--json", file_path] + flags[0] + ["--verbose"],
                            capture_output=True,
                            text=True,
                            check=True,
                            timeout=600
                        )
                        # Parse the command output
                        stats_output = result.stdout
                        transforms_output = result.stderr
                        stats = parse_cargo_output(stats_output)
                        stats['game'] = file_name_stripped
                        stats['language'] = file_extension
                        stats['flags'] = flags[1]
                        file_stats.append(stats)

                        parsed_transforms = parse_transforms(transforms_output)
                        transforms_df = process_transforms(parsed_transforms)
                        transforms_df['game'] = file_name_stripped
                        transforms_df['language'] = file_extension
                        transforms_df['flags'] = flags[1]
                        file_transforms = pd.concat([file_transforms, transforms_df], ignore_index=True)
                    except subprocess.CalledProcessError as e:
                        print(f"Error running {file_path}: {e}")
                        all_flags_succeeded = False
                        break
                    except subprocess.TimeoutExpired as e:
                        print(f"Timeout expired running {file_path}: {e}")
                        all_flags_succeeded = False
                        break

                if all_flags_succeeded:
                    stats_data.extend(file_stats)
                    transforms_data = pd.concat([transforms_data, file_transforms], ignore_index=True)
    return (pd.DataFrame(stats_data), transforms_data)

def main():
    base_directory = "../../games"
    if not os.path.isdir(base_directory):
        print("Invalid base directory.")
        return

    flags = [
        "--compact-comparisons",
        "--compact-reachability",
        "--compact-skip-edges",
        "--inline-assignment",
        "--inline-reachability",
        "--join-exclusive-edges",
        "--join-fork-prefixes",
        "--join-fork-suffixes",
        "--merge-accesses",
        "--propagate-constants",
        "--prune-self-loops",
        "--prune-singleton-types",
        "--prune-unreachable-nodes",
        "--prune-unused-constants",
        "--prune-unused-variables",
        "--reorder-conditions",
        "--skip-artificial-tags",
        "--skip-redundant-tags",
        "--skip-self-assignments",
        "--skip-self-comparisons",
        "--skip-unused-tags"
    ]
    
    # Define the default flag sets
    flag_sets = [
        [["--enable-all-optimizations"], "--enable-all-optimizations"],
        [[], "none"]
    ]

    # All flags but one combinations
    flag_sets.extend(make_flags_combinations(flags))


    # Collect stats and save to a DataFrame
    (stats_data, transform_data) = collect_stats(base_directory, flag_sets, makeGamesList())

    # Save to CSV or display
    stats_output_csv = "results/stats.csv"
    stats_data.to_csv(stats_output_csv, index=False)
    print(f"Stats saved to {stats_output_csv}")

    transforms_output_csv = "results/transforms.csv"
    transform_data.to_csv(transforms_output_csv, index=False)
    print(f"Transforms saved to {transforms_output_csv}")

if __name__ == "__main__":
    main()
