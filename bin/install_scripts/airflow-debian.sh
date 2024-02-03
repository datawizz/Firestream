
set -e

mkdir -p submodules/apache/airflow

cd submodules/apache/airflow

if [ -d .git ]; then
    echo "Airflow already cloned"
else
    git clone https://github.com/apache/airflow.git
fi

git checkout c0ffa9c5d96625c68ded9562632674ed366b5eb3

python -m pip install .