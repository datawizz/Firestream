import pulumi
import pulumi_gcp as gcp
import pulumi_random as random
from typing import Optional, Dict, Any
from dataclasses import dataclass

@dataclass
class PostgresConfig:
    """Configuration for PostgreSQL instance"""
    instance_name: str
    database_name: str
    tier: str = "db-f1-micro"
    version: str = "POSTGRES_13"
    charset: str = "UTF8"
    collation: str = "en_US.UTF8"

class PostgresOperator:
    """Manages PostgreSQL instances on Google Cloud SQL"""

    def __init__(
        self,
        postgres_config: PostgresConfig,
        opts: Optional[pulumi.ResourceOptions] = None
    ):
        self.config = pulumi.Config()
        self.postgres_config = postgres_config
        self.resource_opts = opts or pulumi.ResourceOptions()

        # Get region from Pulumi config
        self.region = self.config.require('region')

        # Generate random password
        self.password = random.RandomPassword(
            f"{postgres_config.instance_name}-password",
            length=20,
            special=True,
            opts=self.resource_opts
        )

        # Create resources
        self.instance = self._create_instance()
        self.database = self._create_database()

        # Export outputs
        self._export_values()

    def _create_instance(self) -> gcp.sql.DatabaseInstance:
        """Creates a new PostgreSQL instance"""
        return gcp.sql.DatabaseInstance(
            self.postgres_config.instance_name,
            region=self.region,
            settings=gcp.sql.DatabaseInstanceSettingsArgs(
                tier=self.postgres_config.tier
            ),
            database_version=self.postgres_config.version,
            root_password=self.password.result,
            opts=self.resource_opts
        )

    def _create_database(self) -> gcp.sql.Database:
        """Creates a new database in the PostgreSQL instance"""
        return gcp.sql.Database(
            self.postgres_config.database_name,
            instance=self.instance.name,
            charset=self.postgres_config.charset,
            collation=self.postgres_config.collation,
            opts=pulumi.ResourceOptions(parent=self.instance)
        )

    def _export_values(self) -> None:
        """Exports relevant values as Pulumi outputs"""
        pulumi.export("instance_connection_name", self.instance.connection_name)
        pulumi.export("database_name", self.database.name)
        pulumi.export("database_password", self.password.result)

    def get_connection_info(self) -> pulumi.Output[Dict[str, Any]]:
        """Returns connection information for the database"""
        return pulumi.Output.all(
            instance_name=self.instance.name,
            database_name=self.database.name,
            password=self.password.result,
            connection_name=self.instance.connection_name
        ).apply(lambda args: {
            "instance_name": args["instance_name"],
            "database_name": args["database_name"],
            "password": args["password"],
            "connection_name": args["connection_name"],
            "connection_string": f"postgresql://postgres:{args['password']}@/{args['database_name']}?host=/cloudsql/{args['connection_name']}"
        })
