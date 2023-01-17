


    # # windowSpec = Window.partitionBy("timestamp").orderBy("timestamp")
    # windowSpec = Window.orderBy(F.col("timestamp").cast("long")).rowsBetween(Window.unboundedPreceding, Window.currentRow)

    # df = df.withColumn("price", F.lag(F.col("randn"), 1).over(windowSpec))


    # # w = Window.orderBy(F.window('timestamp', '1 seconds'))
    # windowSpec = Window.partitionBy(F.session_window('timestamp', '1 seconds')).orderBy("timestamp")
    # # df = df \
    # # .withWatermark('date_format', '5 seconds') \
    # # .groupBy(w).agg(F.lag(F.col("randn"), 1))
    # df = df.withColumn("price", F.lag(F.col("randn"), 1).over(windowSpec))


    # windowSpec = Window.partitionBy(F.window("timestamp", "5 seconds"))

    # df = df.withColumn("row_number",row_number().over(windowSpec))

    # df = df.withWatermark("timestamp", '5 seconds').groupBy("timestamp", F.window("timestamp", '1 seconds')).count()





    # class BrownianMotion():
    #     """
    #     A class for generating Brownian Motion one step at a time
    #     """

    #     def __init__(self):
    #         """
    #         Init class
    #         """
    #         self.x0 = float(100)
        

    #     def gen_point(self):
    #         """
    #         Use the state saved in self to generate the following point in a stochastic series
    #         Before returning update the state in self with this value
    #         """

    #         # The probability of upward or downward motion is 1/2
    #         direction = np.random.choice([1,-1])

    #         # The magnitude of change is sampled from a Normal distribution
    #         yi = np.random.normal()

    #         # Weiner process
    #         w = self.x0 + (yi / np.sqrt(self.x0)) * direction

    #         # Set state to be previous value (to prepare for next iteration)
    #         self.x0 = w

    #         return w

    #     def next_point(self, x):
    #         """
    #         Generate a coherent series of outputs matching the length of the number of inputs
    #         """
    #         return pd.Series([self.gen_point() for y in x])



    # @pandas_udf("long")
    # def brownian_motion(iterator: Iterator[pd.Series]) -> Iterator[pd.Series]:

    #     # Wrap the internal state to ensure proper discarding
    #     try:

    #         # Initialize a class (once per worker) which will hold the previous state
    #         state = BrownianMotion()
    #         # Iterate over the incoming data
    #         for x in iterator:
    #             # Use that state for whole iterator.
    #             yield state.next_point(x)
    #     finally:
    #         pass # Clean up state

    # df = df.withColumn("brownian_motion", brownian_motion(F.col("value")))

