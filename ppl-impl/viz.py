import json
import os

import graphviz
import numpy as np
from matplotlib import pyplot as plt

def smallest_square_bigger_than(n):
    i = 0
    while i**2 < n:
        i += 1
    return i

def main():
    # render_graphs()
    render_charts()

def render_charts():
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
            has_weights = data["has_weights"]
            data = data["data"]
            if has_weights:
                weights = np.exp([d[1] for d in data])
                data = [d[0] for d in data]
            else:
                weights = None
                data = data
            
        print(f"{'Plotting': <10} {file_name}...")
        this_fig, this_ax = plt.subplots()
        for ax in [this_ax, axes[plt_x][plt_y]]:
            ax.set_title(file_stem)
            plot(ax, data, weights)
        this_fig.savefig(f"./charts/{file_stem}.png")
        plt.close(this_fig)
            
    fig.savefig(f"./charts/all.png")
    plt.draw()
    plt.pause(0.1)
    print("Press enter to close...")
    input()
    plt.close()

# def crop_to_greater_than_one(a):
#     less_than_one = a <= 0
#     almost_empty_cols = np.all(less_than_one, axis=0) 
#     almost_empty_rows = np.all(less_than_one, axis=1)
#     firstcol = almost_empty_cols.argmin() 
#     firstrow = almost_empty_rows.argmin()

#     lastcol = len(almost_empty_cols) - almost_empty_cols[::-1].argmin()
#     lastrow = len(almost_empty_rows) - almost_empty_rows[::-1].argmin()

#     return a[firstrow:lastrow,firstcol:lastcol]


def plot_hist(ax, data, weights):
    data = np.array(data)
    ax.set_xlabel("Value")
    ax.set_ylabel("Count")

    ax.hist(data, weights=weights, bins=45)

def plot_hist2d(ax, data, weights):
    print("hi")
    data = np.array(data)
    ax.set_xlabel("x1")
    ax.set_ylabel("x2")
    r = [[-5,5],[-5,5]]
    # r = None
    ax.hist2d(data[:, 0], data[:, 1], bins=50, range=r, weights=weights)
    # print(histogram)
    # histogram = crop_to_greater_than_one(histogram)
    # print(histogram.shape)
    # ax.imshow(histogram)
    # ax.hist2d(data[:, 0], data[:, 1], weights=weights, bins=100)

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

def plot(ax, data, weights):
    if type(data[0]) in (int, float):
        plot_hist(ax, data, weights)
    elif type(data[0]) is list and len(data[0]) == 2 and type(data[0][0]) is float:
        plot_hist2d(ax, data, weights)
    elif type(data[0]) is list and type(data[0][0]) is int:
        plot_hmm(ax, data)
    else:
        print("Data output type not supported yet.")

def render_graphs():
    pgm_files = sorted(list(os.listdir("./pgms-json")))
    print(f"Found {', '.join(pgm_files)}")
    for i, file_name in enumerate(pgm_files):
        file_stem, *_ = file_name.split(".")
        print(f"{'Reading': <10} {file_name}...")
        with open(f"./pgms-json/{file_name}") as data_file:
            pgm = json.load(data_file)
        functions = pgm[0]
        g = pgm[1]
        
        graph = graphviz.Digraph(format="png")
        for node in g["V"]:
            dist_type = g["P"][node][1][0]
            graph.node(node+"_factor", label=f"({dist_type} ...)", shape="box")
            observed = node in g["Y"]
            graph.node(node,  shape="circle", fillcolor = "gray" if observed else "white")
            graph.edge(node+"_factor", node)
        for a, bs in g["A"].items():
            for b in bs:
                graph.edge(a, b+"_factor")
        graph.render(f"./pgms-rendered/{file_stem}")
            

if __name__ == "__main__":
    main()
