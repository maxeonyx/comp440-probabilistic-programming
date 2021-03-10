import numpy as np
n_data = 100
slope = 3
intercept = 6
x = np.linspace(0, 10, 100)
y = ( x * slope + intercept ) + np.random.normal(0, 4, 100)

print(repr(x))
print(repr(y))
