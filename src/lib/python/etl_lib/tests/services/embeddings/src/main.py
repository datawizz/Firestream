

from langchain.vectorstores import AtlasDB
from langchain.embeddings.openai import OpenAIEmbeddings

embeddings = OpenAIEmbeddings()
vectorstore = AtlasDB("my_project", embeddings.embed_query)