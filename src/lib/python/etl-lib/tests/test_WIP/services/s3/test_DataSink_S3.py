from etl_lib import S3_DataSink, DataModel, DataIndex



class test_model(DataModel):
    """
    A simple model that holds known good data and posts it to S3
    in delta lake format
    """

    _index = DataIndex
    name = str


def test_write():

    test_model.make(
        
    )

if __name__ == "__main__":

