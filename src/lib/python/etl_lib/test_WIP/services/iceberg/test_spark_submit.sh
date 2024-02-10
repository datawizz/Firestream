# add Iceberg dependency
ICEBERG_VERSION=1.2.1
DEPENDENCIES="org.apache.iceberg:iceberg-spark-runtime-3.3_2.12:$ICEBERG_VERSION"

# add AWS dependnecy
AWS_SDK_VERSION=2.20.18
AWS_MAVEN_GROUP=software.amazon.awssdk
AWS_PACKAGES=(
    "bundle"
)
for pkg in "${AWS_PACKAGES[@]}"; do
    DEPENDENCIES+=",$AWS_MAVEN_GROUP:$pkg:$AWS_SDK_VERSION"
done

# start Spark SQL client shell
spark-sql --packages $DEPENDENCIES \
    --conf spark.sql.defaultCatalog=my_catalog \
    --conf spark.sql.catalog.my_catalog=org.apache.iceberg.spark.SparkCatalog \
    --conf spark.sql.catalog.my_catalog.warehouse=s3://my-bucket/my/key/prefix \
    --conf spark.sql.catalog.my_catalog.catalog-impl=org.apache.iceberg.aws.glue.GlueCatalog \
    --conf spark.sql.catalog.my_catalog.io-impl=org.apache.iceberg.aws.s3.S3FileIO