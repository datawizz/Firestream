

from confluent_kafka import Consumer
from collections import deque
import json
from multiprocessing import Process, Queue
import multiprocessing
import pandas as pd

BOOTSTRAP_SERVERS = "kafka.default.svc.cluster.local:9092"
SOURCE_TOPIC = "metronome"
DESTINATION_TOPIC = "wiener_process"
DEVICES = 5

"""
Consume messages from Kafka until 100 are in memory.
Then emit messages to Kafka mimicking a lag(1) function from Spark SQL
"""

# created a size limited queue that is shared between processes
queue = Queue(maxsize=10 * DEVICES)


class StatefulStreamProcessing():
    """
    Use a queue in memory to keep track of a subsample of the stream from Kafka

    When conditions are met append the new record to Kafka
    """

    def __init__(self) -> None:

        # Set the internal queue to limit the state kept in memory
        self.read_queue = deque([], 100)
        self.write_queue = deque([], 100)




    def consume_readings(self):
        """
        Consume messages from a topic and return a iterator that yields 
        each new message

        """

        c = Consumer(
            {
                "bootstrap.servers": BOOTSTRAP_SERVERS,
                "group.id": "mygroup9",
                "auto.offset.reset": "latest",
                "enable.auto.commit": True
            }
        )

        c.subscribe([SOURCE_TOPIC])

        while True:
            msg = c.poll(1.0)

            if msg is None:
                continue
            if msg.error():
                print("Consumer error: {}".format(msg.error()))
                continue

            msg = json.loads(msg.value().decode("utf-8"))

            yield msg

    def process_message(self, message:dict):
        """
        Add it to the queue, sort the queue

        if there is a new tuple created upon this update emit it.

        "device_id" : [
        {
        timestamp: "",
        magnitude: "",
        direction: ""
        },
        {
        timestamp: "",
        magnitude: "",
        direction: ""
        }
        ]
        """
        # Append the message to the read queue and then figure out what it updated
        self.read_queue.append(message)

        #_sorted_queue = sorted(self.q, key=lambda d: d['timestamp'])
        df = pd.DataFrame(self.read_queue)

        # Keep the last 2 messages for each "key" and "timestamp" in the queue, ordered by timestamp desc
        df1 = df.sort_values("event_time").groupby("device_id", as_index=True, sort=True).tail(2)
        df1 = df1.sort_values("device_id")

        print(df1)

        # If the key,timestamp is not in the write queue add it, otherwise pass
        _new_index = df1.index.to_list()

        new_outputs = [x for x in _new_index if x not in self.write_queue]

        if new_outputs:
            # Write the new outputs to Kafka and keep an internal state of the messages written
            #self.write_queue.append(x)
            

            print(new_outputs)





    def run(self):

        """
        Run the class logic
        """

        for event in self.consume_readings():
            self.process_message(event)





if __name__ == "__main__":
    c = StatefulStreamProcessing()
    c.run()