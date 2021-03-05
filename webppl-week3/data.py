import numpy as np

slope = 3
intercept = 6
x = np.linspace(0, 10, 11) + np.random.normal(0, 0.3, 11)
y = ( x * slope + intercept ) + np.random.normal(0, 4, 11)

print(x)
print(y)
