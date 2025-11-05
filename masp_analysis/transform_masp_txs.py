#!/usr/bin/env python3
"""
Transform MASP transaction data by flattening wrapper and inner transactions.

This script reads masp_txs.json and creates a flattened output where each entry
corresponds to an inner transaction with selected fields from its wrapper transaction.
"""

import json
import sys


def transform_masp_txs(input_file='masp_txs.json', output_file='transformed_masp_txs.json'):
    """
    Transform MASP transaction data.
    
    For each inner transaction, create an entry containing:
    - wrapper_tx_id: the wrapper transaction ID
    - fee_token_trace: the wrapper's feeToken trace string (if exists)
    - inner_tx_id: the inner transaction ID
    - inner_tx_kind: the inner transaction kind
    - sources: list of sources without 'type' field
    - targets: list of targets without 'type' field
    """
    print(f"Reading {input_file}...")
    
    try:
        with open(input_file, 'r') as f:
            wrapper_txs = json.load(f)
    except FileNotFoundError:
        print(f"Error: {input_file} not found")
        sys.exit(1)
    except json.JSONDecodeError as e:
        print(f"Error parsing JSON: {e}")
        sys.exit(1)
    
    transformed_data = []
    
    print(f"Processing {len(wrapper_txs)} wrapper transactions...")
    
    for wrapper_tx in wrapper_txs:
        wrapper_id = wrapper_tx.get('id')
        wrapper_height = wrapper_tx.get('blockHeight')
        fee_payer = wrapper_tx.get('feePayer')
        fee_token = wrapper_tx.get('feeToken', {})
        fee_token_address = fee_token.get('address')  # May be None if not present
        
        inner_txs = wrapper_tx.get('innerTransactions', [])
        
        for inner_tx in inner_txs:
            inner_tx_id = inner_tx.get('id')
            inner_tx_kind = inner_tx.get('kind')
            data_str = inner_tx.get('data', '{}')
            
            # Parse the data field (it's a JSON string)
            try:
                data = json.loads(data_str)
            except json.JSONDecodeError:
                print(f"Warning: Could not parse data for inner tx {inner_tx_id}")
                data = {}
            
            # Get sources and targets, removing 'type' field from each
            sources = data.get('sources', [])
            targets = data.get('targets', [])
            
            # Remove 'type' field from sources
            filtered_sources = [
                {k: v for k, v in source.items() if k != 'type'}
                for source in sources
            ]
            
            # Remove 'type' field from targets
            filtered_targets = [
                {k: v for k, v in target.items() if k != 'type'}
                for target in targets
            ]
            
            # Create flattened entry
            entry = {
                'wrapper_tx_id': wrapper_id,
                'block_height': wrapper_height,
                'fee_payer': fee_payer,
                'fee_token_address': fee_token_address,
                'inner_tx_id': inner_tx_id,
                'inner_tx_kind': inner_tx_kind,
                'sources': filtered_sources,
                'targets': filtered_targets
            }
            
            transformed_data.append(entry)
    
    print(f"Writing {len(transformed_data)} inner transactions to {output_file}...")
    
    with open(output_file, 'w') as f:
        json.dump(transformed_data, f, indent=2)
    
    print(f"Done! Transformed {len(transformed_data)} inner transactions.")


if __name__ == '__main__':
    transform_masp_txs()

