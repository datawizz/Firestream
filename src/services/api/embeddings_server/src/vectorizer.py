# This takes a Avro or JSON record and finds the vectorized representation of it.
# Different interfaces are supported, including:
# - Avro
# - JSON
# - Spark SQL


# This is the main entry point for the vectorizer.
import torch
from transformers import AutoModel, AutoTokenizer
from nltk.tokenize import sent_tokenize
import weaviate

torch.set_grad_enabled(False)

# udpate to use different model if desired
MODEL_NAME = "distilbert-base-uncased"
model = AutoModel.from_pretrained(MODEL_NAME)
model.to('cuda') # remove if working without GPUs
tokenizer = AutoTokenizer.from_pretrained(MODEL_NAME)

# initialize nltk (for tokenizing sentences)
import nltk
nltk.download('punkt')

# initialize weaviate client for importing and searching
client = weaviate.Client("http://localhost:8080")