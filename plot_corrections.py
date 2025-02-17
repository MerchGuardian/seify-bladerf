#!/usr/bin/env python

import pandas as pd
import matplotlib.pyplot as plt
import numpy as np

# Load the CSV file
file_path = "correction_data.csv"  # Change this to your actual file path
df = pd.read_csv(file_path)

# Calculate the deviation (difference) between requested and applied corrections
for col in ["I correction", "Q correction", "phase correction", "gain correction"]:
    df[f"{col} error"] = df[col] - df["v"]

# Setup the figure with a larger size
fig, axes = plt.subplots(2, 2, figsize=(14, 10))

correction_types = ["I correction", "Q correction", "phase correction", "gain correction"]
colors = ["tab:blue", "tab:orange", "tab:green", "tab:red"]

for ax, corr_type, color in zip(axes.flatten(), correction_types, colors):
    ax.scatter(df["v"], df[corr_type], color=color, s=30, label=corr_type, alpha=0.7)
    ax.plot(df["v"], df["v"], linestyle="dashed", color="black", label="Expected (y=x)")  # Diagonal reference line
    ax.set_xlabel("Requested Correction (v)")
    ax.set_ylabel(f"Applied {corr_type}")
    ax.set_title(f"{corr_type} Trend")
    ax.legend()
    ax.grid(True)

# Adjust layout and show the plots
plt.savefig(f"corrections.png", dpi=300)

# Now, plot the error (deviation from expected values)
fig, ax = plt.subplots(figsize=(10, 5))

for corr_type, color in zip(correction_types, colors):
    ax.plot(df["v"], df[f"{corr_type} error"], label=f"{corr_type} error", color=color)

ax.axhline(0, color="black", linestyle="dashed", linewidth=1)  # Reference line
ax.set_xlabel("Requested Correction (v)")
ax.set_ylabel("Error (Applied - Requested)")
ax.set_title("Deviation from Expected Correction")
ax.legend()
ax.grid(True)

plt.savefig(f"correction_error.png", dpi=300)

