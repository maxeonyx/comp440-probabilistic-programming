from pyro.primitives import sample
import torch
import numpy as np
import pyro
from pyro import sample
from pyro.infer import NUTS, MCMC
from pyro.distributions import Normal
from matplotlib import pyplot as plt

def bad():
    x = sample('x', pyro.distributions.Normal(0, 1))
    for i in range(10):
        x = sample('x', pyro.distributions.Normal(x, 3))
    x

nuts_kernel = NUTS(bad)

mcmc = MCMC(nuts_kernel,
    num_samples=10,
    warmup_steps=10
)
mcmc.run()
mcmc.summary()
samples = mcmc.get_samples()

print(samples.keys())

fig, ax = plt.subplots()
ax.hist(np.array(samples["x"]), bins=50)
plt.show()
