import torch
from torch.autograd import Variable

# Check if a GPU is available and if not, use a CPU
device = torch.device("cuda" if torch.cuda.is_available() else "cpu")

print(f'Using device: {device}')

# Define a simple linear model with a single input and output
class LinearRegressionModel(torch.nn.Module):
    def __init__(self, input_dim, output_dim):
        super(LinearRegressionModel, self).__init__()
        self.linear = torch.nn.Linear(input_dim, output_dim)

    def forward(self, x):
        return self.linear(x)

# Define model
model = LinearRegressionModel(1, 1).to(device)

# Define loss criterion
criterion = torch.nn.MSELoss() 

# Define optimizer
optimizer = torch.optim.SGD(model.parameters(), lr=0.01)

# Define some data
x_values = [i for i in range(10)]
x_train = torch.tensor(x_values, dtype=torch.float).reshape(-1, 1).to(device)
y_train = torch.tensor(x_values, dtype=torch.float).reshape(-1, 1).to(device)

# Training loop
for epoch in range(100000):
    # Forward pass: Compute predicted y by passing x to the model
    y_pred = model(x_train)

    # Compute and print loss
    loss = criterion(y_pred, y_train)
    print(f'Epoch: {epoch}, Loss: {loss.item()}')

    # Zero gradients, perform a backward pass, and update the weights.
    optimizer.zero_grad()
    loss.backward()
    optimizer.step()

# Test the model
model.eval()
test = Variable(torch.Tensor([11.0]).to(device))
predicted = model(test).data[0]
print('Prediction after training ', 'input: ', test.item(), 'output: ', predicted.item())
