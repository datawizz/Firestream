# Dockerfile for the PostgreSQL image used by the PostgreSQL Helm chart

# Step 1: Pull the official PostgreSQL image
FROM postgres:14 as deployment

# Step 2: Install necessary build tools and libraries
USER root

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    build-essential \
    git \
    postgresql-server-dev-all \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Step 3: Download the PGvector source code from the correct repository
RUN git clone https://github.com/pgvector/pgvector.git

# Step 4: Compile and install the PGvector C extension
WORKDIR /pgvector
RUN make && \
    make install

# Step 5: Clean up unnecessary build tools and cache
RUN rm -rf /pgvector
RUN apt-get purge -y --auto-remove \
    build-essential \
    git \
    postgresql-server-dev-all \
    ca-certificates

# Step 6: Configure PostgreSQL to load the PGvector extension and enable logical replication for Debezium
USER postgres
RUN echo "shared_preload_libraries = 'pgvector,pgoutput'" >> /var/lib/postgresql/data/postgresql.conf && \
    echo "wal_level = logical" >> /var/lib/postgresql/data/postgresql.conf && \
    echo "max_wal_senders = 8" >> /var/lib/postgresql/data/postgresql.conf && \
    echo "max_replication_slots = 8" >> /var/lib/postgresql/data/postgresql.conf

# Expose the PostgreSQL port and set the default command
EXPOSE 5432
CMD ["postgres"]
