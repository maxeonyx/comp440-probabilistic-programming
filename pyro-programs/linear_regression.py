from pyro.primitives import sample
import torch
import pyro
from matplotlib import pyplot as plt

torch_true = torch.ones(())
torch_false = torch.zeros(())

def linear():
    x_data, y_data = [1, 2, 3, 4, 5, 6], torch.tensor([2.2, 4.2, 5.5, 8.3, 9.9, 12.1])
    slope = pyro.param('slope', pyro.distributions.Normal(0, 5))
    bias = pyro.param('bias', pyro.distributions.Normal(0, 5))

    for i in range(len(x_data)):
        x = x_data[i]
        mu = x * slope + bias
        y = pyro.sample(f"y_{i}", pyro.distributions.Normal(mu, 1), obs=y_data[i])

nuts_kernel = pyro.infer.NUTS(linear)

mcmc = pyro.infer.MCMC(nuts_kernel,
    num_samples=10000,
    warmup_steps=1000
)
mcmc.run()
mcmc.summary()
samples = mcmc.get_samples()

print(samples)
