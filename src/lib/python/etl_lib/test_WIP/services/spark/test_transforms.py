if __name__ == "__main__":

    interval = [timedelta(seconds=60 * 60)]
    start = pd.to_datetime("2022-01-03 01:00", utc=True).astimezone(pytz.utc)
    end = pd.to_datetime("2022-01-03 17:00", utc=True).astimezone(pytz.utc)

    context = DataContext(model=DataModel, intervals=interval, start=start, end=end)

    df1 = context.spark_client.session.createDataFrame(
        [
            Row(event_time="2022/01/03 03:00:36", value=5.0),
            Row(event_time="2022/01/03 03:40:12", value=10.0),
            Row(event_time="2022/01/03 05:25:30", value=111.0),
        ]
    )
    df2 = context.spark_client.session.createDataFrame(
        [
            Row(event_time="2022/01/03 03:00:36", value=2.0),
            Row(event_time="2022/01/03 03:40:12", value=3.0),
            Row(event_time="2022/01/03 05:25:30", value=1.0),
        ]
    )

    stream_stream_bucket_join(left_df=df1, right_df=df2, context=context)
