

###############################################################################
### Run Spark Applications                                                  ###
###############################################################################

# spark-submit --master spark://spark-master-svc.default.svc.cluster.local:7077 /workspace/src/spark_applications/metronome/src/main.py 

# echo "Run the Metronome Spark Application"
# nohup python /workspace/services/python/pyspark_metronome/main.py > /workspace/logs/pyspark_metronome.log 2>&1 &
