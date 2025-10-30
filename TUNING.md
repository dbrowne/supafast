# High connection counts

# Check current limits
```bash
 ulimit -n
# Increase to 65535
 ulimit -n 65535
```


## Or edit /etc/security/limits.conf:
```ini
* soft nofile 65535
* hard nofile 65535
```

# Linux TCP Tuning
```bash

# Increase TCP buffers
sudo sysctl -w net.core.rmem_max=16777216
sudo sysctl -w net.core.wmem_max=16777216
sudo sysctl -w net.ipv4.tcp_rmem='4096 87380 16777216'
sudo sysctl -w net.ipv4.tcp_wmem='4096 65536 16777216'

# Enable TCP fast open
sudo sysctl -w net.ipv4.tcp_fastopen=3
```


# Postgres settings

postgresql.conf
```ini
# Connection pooling
max_connections = 200              # Ensure pool_size < this

# Performance
shared_buffers = 4GB              # 25% of RAM
effective_cache_size = 12GB       # 75% of RAM
work_mem = 64MB                   # Per-operation memory
maintenance_work_mem = 512MB

# Write performance
wal_buffers = 16MB
checkpoint_completion_target = 0.9
max_wal_size = 4GB

# Query optimization
random_page_cost = 1.1            # For SSDs
effective_io_concurrency = 200    # For SSDs
```


# Bottleneck identification

1. CPU Profiling
```bash 

# Install cargo-flamegraph
cargo install flamegraph

# Profile release build
cargo flamegraph --release

# Open flamegraph.svg
```

2. Memory profiling
```bash 
 # Use valgrind
valgrind --tool=massif ./target/release/supafast

# Or heaptrack (Linux)
heaptrack ./target/release/supafast
```

3. DB query analysis
4. ```sql

-- Enable query logging in PostgreSQL
ALTER SYSTEM SET log_min_duration_statement = 100;  -- Log queries > 100ms
SELECT pg_reload_conf();

-- Monitor slow queries
SELECT query, calls, total_time, mean_time
FROM pg_stat_statements
ORDER BY mean_time DESC
LIMIT 10;
```
