

from typing import Iterator
import pandas as pd
import numpy as np
from pyspark.sql.functions import pandas_udf




class BrownianMotion():
    """
    A class for generating Brownian Motion one step at a time
    """

    def __init__(self):
        """
        Init class
        """
        self.x0 = float(100)
    

    def next_point(self):
        """
        Use the state saved in self to generate the following point in a stochastic series
        Before returning update the state in self with this value
        """

        # The probability of upward or downward motion is 1/2
        direction = np.random.choice([1,-1])

        # The magnitude of change is sampled from a Normal distribution
        yi = np.random.normal()

        # Weiner process
        w = self.x0 + (yi / np.sqrt(self.x0)) * direction

        # Set state to be previous value (to prepare for next iteration)
        self.x0 = w

        return w



@pandas_udf("long")
def brownian_motion(iterator: Iterator[pd.Series]) -> Iterator[pd.Series]:

    # Wrap the internal state to ensure proper discarding
    try:

        # Initialize a class (once per worker) which will hold the previous state
        state = BrownianMotion()
        # Iterate over the incoming data
        for x in iterator:
            # Use that state for whole iterator.
            yield state.next_point()
    finally:
        pass # Clean up state

df = df.withColumn("brownian_motion", brownian_motion(F.col("value")))

# df.select(brownian_motion("value")).show()


b = BrownianMotion()
for i in range(1000):
    print(b.next_point())