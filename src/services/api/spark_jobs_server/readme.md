# Spark Jobs API

Spark Jobs API is a REST API to accept remote procedure calls to update running spark jobs.

The API should attend to the following:

1. Estimate the amount of additional processing load to implement the submitted job
2. Parse the provided SQL received in the payload for safety and sanity. DONT RUN IT
3. Submit the job to the kubernetes cluster as a streaming job.
4. Provide a method to retrieve the streaming jobs on the server
    That this user is permissioned to see
5. Provide a method to cancel the streaming job.
6. Implement the remote procedure calls via pydantic classes over ETL lib