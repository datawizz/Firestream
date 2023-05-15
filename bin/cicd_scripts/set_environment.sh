#!/bin/bash

# Used for setting complex environment variables


echo 'export PYTHONPATH=$(ZIPS=("$SPARK_HOME"/python/lib/*.zip); IFS=:; echo "${ZIPS[*]}"):$PYTHONPATH' >> ~/.bashrc
