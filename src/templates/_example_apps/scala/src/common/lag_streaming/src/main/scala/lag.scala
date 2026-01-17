
// Implement a function in Scala which when packaged into a Jar can be called from PySpark or Spark Scala

import org.apache.spark.sql.{Encoder, Encoders, Row}
import org.apache.spark.sql.functions._
import org.apache.spark.sql.streaming.{GroupStateTimeout, OutputMode, StreamingQuery}
import org.apache.spark.sql.types.{StructType, StructField, StringType, IntegerType}

def lagFunction(df: DataFrame, lagValue: Int, stateTimeout: String): DataFrame = {
  val schema = df.schema

  def mappingFunction(key: String, values: Iterator[Row], state: GroupState[Seq[Row]]): Iterator[Row] = {
    val currentBatch = values.toSeq
    val previousBatch = state.getOption.getOrElse(Seq.empty[Row])
    val completeBatch = previousBatch ++ currentBatch

    if (completeBatch.size > lagValue) {
      val lagRows = completeBatch.takeRight(lagValue)
      state.update(lagRows)

      val structFields = (0 until lagValue).map { i =>
        StructField(s"lag_$i", schema, nullable = true)
      }
      val structType = StructType(structFields)
      val lagSchema = Encoders.product[LagRow].schema

      val lagData = lagRows.zipWithIndex.map { case (row, index) =>
        LagRow(row, index)
      }

      val rowWithLag = currentBatch.last.toSeq ++ Seq(create_struct(lagData.map(row => row.row)))
      Iterator(Row.fromSeq(rowWithLag))
    } else {
      state.update(completeBatch)
      Iterator.empty
    }
  }

  case class LagRow(row: Row, index: Long)

  df.groupByKey(row => row.getString(0))
    .flatMapGroupsWithState(OutputMode.Append, GroupStateTimeout.ProcessingTimeTimeout)(mappingFunction)
    .toDF("key", "value", "lag_data")
}
