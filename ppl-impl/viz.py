import json
import os

import matplotlib

import numpy as np
from matplotlib import pyplot as plt

def smallest_square_bigger_than(n):
    i = 0
    while i**2 < n:
        i += 1
    return i

def main():
    data_files = sorted(list(os.listdir("./data")))
    n_plots = smallest_square_bigger_than(len(data_files))
    print(f"{len(data_files)} plot(s) on a {n_plots}x{n_plots} grid.")
    print(f"Found {', '.join(data_files)}")
    # big plot with a lot of axes
    fig, axes = plt.subplots(n_plots, n_plots, squeeze=False, figsize=(4*n_plots, 4*n_plots))

    for i, file_name in enumerate(data_files):
        plt_x, plt_y = i // n_plots, i % n_plots
        file_stem, *_ = file_name.split(".")
        print(f"{'Reading': <10} {file_name}...")
        with open(f"./data/{file_name}") as data_file:
            data = json.load(data_file)
            print(f"{'Plotting': <10} {file_name}...")
            this_fig, this_ax = plt.subplots()
            for ax in [this_ax, axes[plt_x][plt_y]]:
                ax.set_title(file_stem)
                plot(ax, data)
            this_fig.savefig(f"./charts/{file_stem}.png")
            plt.close(this_fig)
            
    fig.savefig(f"./charts/all.png")
    plt.draw()
    plt.pause(0.1)
    print("Press enter to close...")
    input()
    plt.close()

def plot_hist(ax, data):
    ax.set_xlabel("Value")
    ax.set_ylabel("Count")
    ax.hist(data, bins=45)

def plot_hist2d(ax, data):
    data = np.array(data)
    ax.set_xlabel("x1")
    ax.set_ylabel("x2")
    ax.hist2d(data[:, 0], data[:, 1], bins=20)

def better_bincount(data, n):
    m = data.shape[1]   
    A1 = data + (n*np.arange(m))
    return np.bincount(A1.ravel(),minlength=n*m).reshape(m,-1).T

def plot_hmm(ax: plt.Axes, data):
    data = np.array(data)
    n = data.max() + 1
    counts = better_bincount(data, n)
    dist = counts / data.shape[0]
    ax.set_xlabel("iteration")
    ax.set_ylabel("state")
    ax.imshow(dist, vmin=0, vmax=1)

def plot(ax, data):
    if type(data[0]) in (int, float):
        plot_hist(ax, data)
    elif type(data[0]) is list and len(data[0]) == 2 and type(data[0][0]) is float:
        plot_hist2d(ax, data)
    elif type(data[0]) is list and type(data[0][0]) is int:
        plot_hmm(ax, data)
    else:
        print("Data output type not supported yet.")

if __name__ == "__main__":
    main()
