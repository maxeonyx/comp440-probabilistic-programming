from pyro.primitives import sample
import torch
import numpy as np
import pyro
from pyro import sample
from pyro.infer import NUTS, MCMC
from pyro.distributions import Normal
from matplotlib import pyplot as plt

def linear():
    x_data, y_data = [1, 2, 3, 4, 5, 6], torch.tensor([2.2, 4.2, 5.5, 8.3, 9.9, 12.1])
    k = sample('k', pyro.distributions.Normal(0, 1))
    if k < 0:
        slope = sample('slope', Normal(0, 5))
    else:
        slope = sample('slope', pyro.distributions.Bernoulli(0.5))

    bias = sample('bias', Normal(0, 5))

    for i in range(len(x_data)):
        x = x_data[i]
        mu = x * slope + bias
        y = sample(f"y_{i}", Normal(mu, 1), obs=y_data[i])

nuts_kernel = NUTS(linear)

mcmc = MCMC(nuts_kernel,
    num_samples=1000,
    warmup_steps=10
)
mcmc.run()
mcmc.summary()
samples = mcmc.get_samples()

print(samples)

fig, ax = plt.subplots()
ax.hist2d(np.array(samples["slope"]), np.array(samples["bias"]), bins=30)
plt.show()
