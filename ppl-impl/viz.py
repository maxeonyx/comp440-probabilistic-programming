import json
import os

from matplotlib import pyplot as plt

data_files = list(os.listdir("./data"))

def smallest_square_bigger_than(n):
    i = 0
    while i**2 < n:
        i += 1
    return i

n_plots = smallest_square_bigger_than(len(data_files))
print(n_plots)
print(data_files)
# big plot with a lot of axes
fig, axes = plt.subplots(n_plots, n_plots, squeeze=False)

for i, file_name in enumerate(data_files):
    plt_x, plt_y = i % n_plots, i // n_plots
    file_stem, *_ = file_name.split(".")
    with open(f"./data/{file_name}") as data_file:
        data = json.load(data_file)
        ax = axes[plt_x][plt_y]
        ax.set_title(file_stem)
        ax.set_xlabel("Value")
        ax.set_ylabel("Count")
        ax.hist(data, bins=45)
fig.savefig(f"./charts/{file_stem}.png", )
plt.show()
