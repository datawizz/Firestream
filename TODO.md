
1. Setup Kyuubi to build a Spark Driver/Worker container with all dependencies (from the build.sh)

2. Setup Superset to have a default Database in Hive / Kyuubi

3. Setup Spark Catalog to load the SPIDER dataset and BIRD dataset

4. THE MVP - Develop CTEs for Abstract Tools over the database.
    1. Review the database table by table.
        Use the "Contraint" to find the forgien keys
        Map this into DataModel definition?

Benchmark it!
<https://bird-bench.github.io>
The SOTA is 43% accuracy on text to SQL commands???

Run these SQL commands one at a time through GPT4
/workspace/tmp/data_plugins/bird/train/train_gold.sql

Each and every Table in any database has the following

class Column(DataModel):
    """
    Defines the abstraction of a Column within a Table, with all the assumptions of ANSII SQL.

    A column is the primary abstraction of a table.
    """

    id: UUID
    name: str
    datatype: UNION[*ALLOWED_TYPES]
    count: Int # The number of elements in this column.

    partition_by: bool # If this column should be used in the partitioning logic
    index: bol
    index_with: list[str] # An array of strings representing columns in this Coulumn's Table that should be co-located with this column when partitioning

    
    # Statistical Properties
    # Uncertainty in Statistical Properties # This value should be updated via the function library

    # A 10 word description
    # A 100 word description
    # A 1000 word description

    def sample(self):
        """
        Randomly sample a segment of data from the column to understand its properties.
        Update the certainty according to how much of the dataset has been explored this way.
        Make extension use of partitioned columns

        """


    def sample_statistics(self):
        """
        Find the mean, median, standard deviation, etc for the distribution of the columns actual values by sampling them
        """

class Table(DataModel):
    """
    Defines the abstraction of a Table with all the assumptions of ANSII SQL.

    Extends the definition to include metadata required for various operations.

    """

    id: UUID # A universally unique ID
    name: str # The name to refer to in SQL statements
    columns: array[str] # An array of undeleted columns that are queriable.

The demand for the deployment of AI will outpace the supply of execution environments when the model size is beyond a few hundred billion wegihts.

Langchain implements Spark SQL.

Need to setup DataContext to create and manage the branch of the Agent.
The DataContext should intercept all the SQL that is run, send it to Kafka as a log,
and then discard any agent supplied SQL that contains a modififction to the branch that is being accessed.

Therefore whatever is returned and run in SQL will be limited to that instance of the agent which is created at runtime.

Benchmark it!
<https://bird-bench.github.io>
The SOTA is 43% accuracy on text to SQL commands???

Run these SQL commands one at a time through GPT4
/workspace/tmp/data_plugins/bird/train/train_gold.sql

1. Submodules must be cloned to be used in "make demo"
    This depends on Git being able to access the private repos in the repo...

2. Superset requires "helm dependency build" before it will run. Should this be glued to the FireStream implementation?

3. The machine that the repo is run on requires access to git "inside" the contianer, which is not setup from the local environment automatically.
