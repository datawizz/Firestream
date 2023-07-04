import jax
import jax.numpy as np
from jax import grad, jit, vmap
from jax import random

# Initialize random key
key = random.PRNGKey(0)

# Generate some random data for a simple linear regression
n_samples = 100
X = random.normal(key, (n_samples, 1))
y = X * 3 + 2 + random.normal(key, (n_samples, 1))

# Define the model
def model(params, X):
    w, b = params
    return np.dot(X, w) + b

# Define the loss function
def loss(params, X, y):
    y_pred = model(params, X)
    return np.mean((y_pred - y) ** 2)

# Calculate gradient of the loss function with respect to the model parameters
grad_loss = jit(grad(loss, argnums=0))

# Initialize model parameters
params = (random.normal(key, (1,)), random.normal(key))

# Set learning rate
learning_rate = 0.01

# Train the model using gradient descent
for _ in range(1000):
    gradient = grad_loss(params, X, y)
    params = (params[0] - learning_rate * gradient[0], params[1] - learning_rate * gradient[1])

# Print out the final parameters
print(params)
