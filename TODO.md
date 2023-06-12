




1. Setup Kyuubi to build a Spark Driver/Worker container with all dependencies (from the build.sh)

2. Setup Superset to have a default Database in Hive / Kyuubi

3. Setup Spark Catalog to load the SPIDER dataset and BIRD dataset

4. THE MVP - Develop CTEs for Abstract Tools over the database.
    1. Review the database table by table.
        Use the "Contraint" to find the forgien keys
        Map this into DataModel definition?


Benchmark it!
https://bird-bench.github.io
The SOTA is 43% accuracy on text to SQL commands???
     



Run these SQL commands one at a time through GPT4
/workspace/tmp/data_plugins/bird/train/train_gold.sql