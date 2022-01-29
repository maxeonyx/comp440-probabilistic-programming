import torch
import pyro
from matplotlib import pyplot as plt

torch_true = torch.ones(())
torch_false = torch.zeros(())

def weather():
    # It's cloudy half the time.
    cloudy = pyro.sample('cloudy', pyro.distributions.Bernoulli(0.5))
    
    if cloudy:
        # If it's cloudy it's usually raining.
        raining = pyro.sample('raining', pyro.distributions.Normal(0.8, 0.1))
    else:
        # Sometimes it rains when it's not cloudy.
        raining = pyro.sample('raining', pyro.distributions.Bernoulli(0.2))

    if cloudy:
        # We don't usually turn the sprinkler on when it's cloudy
        sprinkler_on = pyro.sample('sprinkler_on', pyro.distributions.Bernoulli(0.1))
    else:
        # We turn it on half of the time when it's fine.
        sprinkler_on = pyro.sample('sprinkler_on', pyro.distributions.Bernoulli(0.5))

    if sprinkler_on and raining:
        # The grass is almost certainly wet if it's raining and the sprinkler is on.
        grass_wet_theta = 0.99
    elif sprinkler_on or raining:
        # the sprinkler and the rain each have a 90% chance of making the grass wet
        grass_wet_theta = 0.9
    else:
        # if neither the sprinkler nor the rain are going then the grass is almost certainly dry.
        grass_wet_theta = 0.01
    grass_wet = pyro.sample('grass_wet', pyro.distributions.Bernoulli(grass_wet_theta))
    
    return torch.tensor([cloudy, raining, sprinkler_on, grass_wet])

# def conditioned_weather():
#     pyro.condition(weather, data={"grass_wet": torch_true})

nuts_kernel = pyro.infer.NUTS(weather)

mcmc = pyro.infer.MCMC(nuts_kernel,
    num_samples=10,
)
mcmc.run()
samples = mcmc.get_samples()

print(samples)
