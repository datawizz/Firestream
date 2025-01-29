import os

thelibFolder = os.path.dirname(os.path.realpath(__file__))
requirementPath = thelibFolder + "/requirements.txt"
install_requires = []  # Here we'll get: ["gunicorn", "docutils>=0.3", "lxml==0.5a7"]
if os.path.isfile(requirementPath):
    with open(requirementPath, "r") as f:
        install_requires = f.read().splitlines()


from setuptools import setup, find_packages

if __name__ == "__main__":
    setup(
        name="etl_lib",
        version="0.0.1",
        packages=find_packages(),
        install_requires=install_requires,
    )
