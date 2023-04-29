

from langchain.document_loaders import PyPDFLoader # for loading the pdf
from langchain.embeddings import OpenAIEmbeddings # for creating embeddings
from langchain.vectorstores import Chroma # for the vectorization part
from langchain.chains import ChatVectorDBChain # for chatting with the pdf
from langchain.llms import OpenAI # the LLM model we'll use (CHatGPT)
from langchain.chat_models import ChatOpenAI

pdf_path = "/workspace/src/lib/etl_lib/tests/example/cameras.pdf"
loader = PyPDFLoader(pdf_path)
pages = loader.load_and_split()



# pages = [pages[0]]


for page in pages:
    print(page.page_content)


embeddings = OpenAIEmbeddings()
vectordb = Chroma.from_documents(pages, embedding=embeddings,
                                 persist_directory=".")
vectordb.persist()


pdf_qa = ChatVectorDBChain.from_llm(OpenAI(temperature=0.9, model_name="gpt-3.5-turbo"),
                                    vectordb, return_source_documents=True)

query = "Tell me about Zoom Lens available in 50mm"
result = pdf_qa({"question": query, "chat_history": ""})
print("Answer:")
print(result["answer"])