#!/bin/zsh

# Start time tracking
start_time=$(date +%s)
echo "Starting pipeline at $(date)"

./scrape_indexer.sh
python3 transform_masp_txs.py
python3 filter_masp_txs.py

# End time tracking
end_time=$(date +%s)
duration=$((end_time - start_time))
minutes=$((duration / 60))
seconds=$((duration % 60))

echo ""
echo "================================================"
echo "Data processing pipeline completed at $(date)"
echo "Total time: ${minutes}m ${seconds}s (${duration} seconds)"
echo "Now analyzing data..."
echo "================================================"

python3 analyze.py
