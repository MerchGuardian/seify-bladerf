#!/usr/bin/env python3
import os
import argparse
import json
import base58
import pandas as pd

def parse_filename(filename):
    """
    Given a filename (without path) like "<base58_encoded_string>.csv",
    remove the extension, decode the Base58 string, and return the JSON-parsed parameters.
    """
    base58_str = os.path.splitext(filename)[0]
    try:
        decoded = base58.b58decode(base58_str)
        params = json.loads(decoded)
        return params
    except Exception as e:
        print(f"Error decoding filename {filename}: {e}")
        return None

def main():
    parser = argparse.ArgumentParser(
        description="Combine CSV files from bladeRF power testing into one DataFrame indexed by Parameters."
    )
    parser.add_argument(
        "-d", "--dir", required=True, help="Directory containing CSV files."
    )
    args = parser.parse_args()

    # List all CSV files in the specified directory.
    csv_files = [f for f in os.listdir(args.dir) if f.endswith(".csv")]
    if not csv_files:
        print("No CSV files found in the specified directory.")
        return

    records = []
    for fname in csv_files:
        params = parse_filename(fname)
        if params is None:
            continue

        filepath = os.path.join(args.dir, fname)
        try:
            df = pd.read_csv(filepath)
            avg_power = df['power'].mean()
        except Exception as e:
            print(f"Error processing {filepath}: {e}")
            continue

        # Extract relevant parameters.
        record = {
            "frequency": params.get("frequency"),
            "channel_set": str(params.get("channel_set")),
            "rx_gain": params.get("rx_gain"),
            "tx_gain": params.get("tx_gain"),
            "external_bias_tee": params.get("external_bias_tee"),
            "external_lna": params.get("external_lna"),
            "average_power": avg_power
        }
        records.append(record)

    if not records:
        print("No valid data records found.")
        return

    # Create a combined DataFrame.
    combined_df = pd.DataFrame(records)
    # Optionally, set a MultiIndex with the parameters for easier querying.
    combined_df.set_index(
        ["frequency", "channel_set", "rx_gain", "tx_gain", "external_bias_tee", "external_lna"],
        inplace=True
    )

    combined_df.sort_values(by="average_power", ascending=True, inplace=True)

    # Display the combined DataFrame.
    print("Combined DataFrame (indexed by Parameters):")
    print(combined_df)

    # Optionally, save the combined DataFrame to a CSV file.
    # output_file = os.path.join(args.dir, "combined_results.csv")
    # combined_df.to_csv(output_file)
    # print(f"Combined results saved to {output_file}")

if __name__ == "__main__":
    main()

