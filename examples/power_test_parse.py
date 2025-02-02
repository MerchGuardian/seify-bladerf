#!/usr/bin/env python3
import os
import argparse
import json
import base58
import pandas as pd
import numpy as np
import statsmodels.formula.api as smf
import matplotlib.pyplot as plt
import seaborn as sns

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

def combine_csv_files(directory):
    """
    Read all CSV files in the given directory, compute the average power from each,
    and combine the results into one DataFrame with parameters as columns.
    """
    records = []
    for fname in os.listdir(directory):
        if not fname.endswith(".csv"):
            continue
        filepath = os.path.join(directory, fname)
        try:
            df = pd.read_csv(filepath)
            avg_power = df['power'].mean()
        except Exception as e:
            print(f"Error processing {filepath}: {e}")
            continue

        params = parse_filename(fname)
        if params is None:
            continue

        record = {
            "frequency": params.get("frequency"),
            "channels_enabled": str(params.get("channels_enabled")),
            "rx_gain": params.get("rx_gain"),
            "tx_gain": params.get("tx_gain"),
            "external_bias_tee": params.get("external_bias_tee"),
            "external_lna": params.get("external_lna"),
            "average_power": avg_power
            # "csv_file": fname
        }
        records.append(record)
    if not records:
        raise ValueError("No valid records found in the directory.")
    df_combined = pd.DataFrame(records)
    # Create a multi-index based on the parameters for easier querying
    df_combined.set_index(
        ["frequency", "channels_enabled", "rx_gain", "tx_gain", "external_bias_tee", "external_lna"],
        inplace=True
    )
    # Sort by average_power (lowest at the top, highest at the bottom)
    df_combined.sort_values(by="average_power", ascending=True, inplace=True)
    return df_combined.reset_index()  # reset index for analysis

def add_derived_columns(df):
    """
    Add derived columns that help differentiate the effect of each parameter.
    For example, compute the total number of active channels from the channels_enabled string.
    """
    def count_channels(ch_str):
        # Expecting a string like "['Rx0', 'Rx1']"
        count = 0
        for token in ['Rx0', 'Rx1', 'Tx0', 'Tx1']:
            if token in ch_str:
                count += 1
        return count

    df['total_channels'] = df['channels_enabled'].apply(count_channels)
    return df

def analyze_effects(df):
    """
    Group the data by key parameters and print out mean power differences.
    """
    print("\nMean power by channels_enabled:")
    grp = df.groupby("channels_enabled")["average_power"].mean().reset_index()
    print(grp)

    print("\nMean power by total_channels (derived):")
    grp = df.groupby("total_channels")["average_power"].mean().reset_index()
    print(grp)

    print("\nMean power by rx_gain:")
    grp = df.groupby("rx_gain")["average_power"].mean().reset_index()
    print(grp)

    print("\nMean power by tx_gain:")
    grp = df.groupby("tx_gain")["average_power"].mean().reset_index()
    print(grp)

    print("\nMean power by external_bias_tee:")
    grp = df.groupby("external_bias_tee")["average_power"].mean().reset_index()
    print(grp)

    print("\nMean power by external_lna:")
    grp = df.groupby("external_lna")["average_power"].mean().reset_index()
    print(grp)

    print("\nMean power by frequency:")
    grp = df.groupby("frequency")["average_power"].mean().reset_index()
    print(grp)

def regression_analysis(df, output_dir):
    """
    Perform an OLS regression analysis to assess the effect of each parameter on average power.
    """
    # Convert booleans to integers for regression
    df['external_bias_tee'] = df['external_bias_tee'].astype(int)
    df['external_lna'] = df['external_lna'].astype(int)
    # For regression, we may want to include frequency, rx_gain, tx_gain, and total_channels.
    # We'll build a formula for OLS regression.
    formula = "average_power ~ frequency + rx_gain + tx_gain + external_bias_tee + external_lna + total_channels"
    model = smf.ols(formula, data=df).fit()
    print("\nRegression Analysis Summary:")
    print(model.summary())

def main():
    parser = argparse.ArgumentParser(
        description="Combine bladeRF power testing CSV files into one DataFrame and analyze which parameters affect power consumption."
    )
    parser.add_argument("-d", "--dir", required=True, help="Directory containing CSV files.")
    args = parser.parse_args()

    # Combine all CSV files into one DataFrame.
    try:
        df_combined = combine_csv_files(args.dir)
    except ValueError as e:
        print(e)
        return

    # Add derived columns.
    df_combined = add_derived_columns(df_combined)

    # Save the combined DataFrame for reference.
    combined_csv_path = os.path.join(args.dir, "combined_results.csv")
    df_combined.to_csv(combined_csv_path, index=False)
    print(f"Combined results saved to {combined_csv_path}\n")

    # Print the DataFrame sorted by average_power.
    print("Combined DataFrame (sorted by average_power):")
    print(df_combined)

    # Analyze effects by grouping by individual parameters.
    analyze_effects(df_combined)

    # Perform a regression analysis to determine which parameters have the most effect.
    regression_analysis(df_combined, args.dir)

if __name__ == "__main__":
    main()

