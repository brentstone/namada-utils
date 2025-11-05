#!/usr/bin/env python3

import json
import subprocess
import os
from datetime import datetime, timedelta
from collections import defaultdict
from typing import Dict, List, Optional, Tuple

# ============================================================================
# Configuration
# ============================================================================

# Time ranges to analyze (can edit as needed)
RANGES = ["30d", "100d", "250d"]

# Block time in seconds
BLOCK_TIME_SECONDS = 7

# Path to the filtered data
DATA_FILE = "filtered_for_analysis.json"

# Path to token configuration
TOKENS_FILE = "config/tokens.json"

# RPC endpoint (reads from environment variable)
RPC_MAINNET = os.getenv("RPC_MAINNET", "https://rpc.mainnet.siuuu.click")

# Set to a specific block height, or None to query the current block height
REFERENCE_BLOCK_HEIGHT: Optional[int] = None  # e.g., 4173600 or None

SHIELDING_FEE = 0.05
UNSHIELDING_FEE = 0.05

# ============================================================================
# Helper Functions
# ============================================================================

def parse_time_range(range_str: str) -> int:
    """
    Parse a time range string like '6h', '12h', '1d', '7d' into seconds.
    
    Args:
        range_str: Time range string (e.g., "6h", "1d", "7d")
        
    Returns:
        Number of seconds in the range
    """
    if range_str.endswith('h'):
        hours = int(range_str[:-1])
        return hours * 3600
    elif range_str.endswith('d'):
        days = int(range_str[:-1])
        return days * 86400
    else:
        raise ValueError(f"Unknown time range format: {range_str}")


def get_current_block_height() -> Optional[int]:
    """
    Query the current block height from the network using namadac CLI.
    
    Returns:
        Current block height as an integer, or None if query fails
    """
    try:
        cmd = f"namadac block --node {RPC_MAINNET}"
        result = subprocess.run(
            cmd,
            shell=True,
            capture_output=True,
            text=True,
            check=True
        )
        # Parse output like "Last committed block ID: 4173600, height: 4173600"
        output = result.stdout.strip()
        # Extract height after "height: " and remove comma
        height_str = output.split("height: ")[1].split(",")[0].strip()
        return int(height_str)
    except Exception as e:
        print(f"Warning: Failed to get current block height: {e}")
        print("You can set REFERENCE_BLOCK_HEIGHT in the script or the script will use the newest block in the data.")
        return None


def blocks_to_seconds(blocks: int) -> int:
    """Convert number of blocks to seconds."""
    return blocks * BLOCK_TIME_SECONDS


def seconds_to_blocks(seconds: int) -> int:
    """Convert seconds to number of blocks."""
    return seconds // BLOCK_TIME_SECONDS


def format_amount(amount: str, decimals: int) -> float:
    """
    Format an amount string with the given decimals.
    
    Args:
        amount: Amount as string (raw value)
        decimals: Number of decimals for the token
        
    Returns:
        Float value with decimals applied
    """
    return float(amount) / (10 ** decimals)


def format_large_number(num: float) -> str:
    """Format large numbers with commas and 2 decimal places."""
    return f"{num:,.2f}"


# ============================================================================
# Analysis Functions
# ============================================================================

def load_data() -> List[Dict]:
    """Load the filtered transaction data."""
    print(f"Loading data from {DATA_FILE}...")
    with open(DATA_FILE, 'r') as f:
        data = json.load(f)
    print(f"Loaded {len(data)} transactions")
    return data


def load_tokens() -> Dict:
    """Load token configuration."""
    with open(TOKENS_FILE, 'r') as f:
        return json.load(f)


def analyze_time_range(
    transactions: List[Dict],
    tokens: Dict,
    reference_height: int,
    range_str: str
) -> Dict:
    """
    Analyze transactions within a specific time range.
    
    Args:
        transactions: List of transaction dictionaries
        tokens: Token configuration dictionary
        reference_height: Reference block height (usually current height)
        range_str: Time range string (e.g., "6h", "1d")
        
    Returns:
        Dictionary with analysis results
    """
    range_seconds = parse_time_range(range_str)
    range_blocks = seconds_to_blocks(range_seconds)
    min_height = reference_height - range_blocks
    
    # Filter transactions in range
    txs_in_range = [
        tx for tx in transactions
        if tx['block_height'] >= min_height and tx['block_height'] <= reference_height
    ]
    
    # Aggregate by token and transaction type
    token_amounts = defaultdict(float)  # token -> total amount
    token_counts = defaultdict(int)      # token -> count
    type_counts = defaultdict(int)       # tx_type -> count
    type_amounts = defaultdict(lambda: defaultdict(float))  # tx_type -> token -> amount
    type_fees = defaultdict(lambda: defaultdict(float))  # tx_type -> token -> fees
    
    for tx in txs_in_range:
        token_addr = tx['token']
        tx_type = tx['tx_type']
        amount_str = tx['amount']
        
        # Get token info
        token_info = tokens.get(token_addr, {'name': 'UNKNOWN', 'decimals': 6, 'price': 0.00})
        decimals = token_info['decimals']
        token_name = token_info['name']
        token_price = token_info['price']

        if token_name == 'UNKNOWN':
            print(f"Warning: Unknown token {token_addr} in transaction {tx['inner_tx_id']}")
        
        # Format amount
        amount = format_amount(amount_str, decimals)
        
        # Update aggregates
        token_amounts[token_name] += amount
        token_counts[token_name] += 1
        type_counts[tx_type] += 1
        type_amounts[tx_type][token_name] += amount

        if tx_type == 'shielding' or tx_type == 'ibc_shielding':
            type_amounts['shield'][token_name] += amount
            type_fees['shield'][token_name] += SHIELDING_FEE * amount
        elif tx_type == 'unshielding' or tx_type == 'ibc_unshielding':
            type_amounts['unshield'][token_name] += amount
            type_fees['unshield'][token_name] += UNSHIELDING_FEE * amount
    
    return {
        'range': range_str,
        'range_seconds': range_seconds,
        'range_blocks': range_blocks,
        'min_height': min_height,
        'max_height': reference_height,
        'total_txs': len(txs_in_range),
        'token_amounts': dict(token_amounts),
        'token_counts': dict(token_counts),
        'type_counts': dict(type_counts),
        'type_amounts': {k: dict(v) for k, v in type_amounts.items()},
        'type_fees': {k: dict(v) for k, v in type_fees.items()},
    }


def print_analysis_results(results: Dict):
    """Print analysis results in a readable format."""
    print("\n" + "=" * 80)
    print(f"Time Range: {results['range']}")
    print(f"Block Range: {results['min_height']:,} to {results['max_height']:,} ({results['range_blocks']:,} blocks)")
    print(f"Duration: ~{results['range_seconds'] // 3600} hours ({results['range_seconds'] // 86400} days)")
    print("=" * 80)
    
    print(f"\nTotal Transactions: {results['total_txs']:,}")
    print(f"\nASSUME a shielding fee of {100*SHIELDING_FEE:.2f}% and an unshielding fee of {100*UNSHIELDING_FEE:.2f}%")

    if results['total_txs'] == 0:
        print("\nNo transactions in this time range.")
        return
    
    # Transaction types breakdown
    print("\nTransaction Types:")
    for tx_type, count in sorted(results['type_counts'].items(), key=lambda x: x[1], reverse=True):
        percentage = (count / results['total_txs']) * 100
        print(f"  {tx_type:20s}: {count:6,} ({percentage:5.1f}%)")
    
    # Token amounts breakdown
    print("\nTotal Absolute Amounts Transacted by Token:")
    for token, amount in sorted(results['token_amounts'].items(), key=lambda x: x[1], reverse=True):
        # Calculate total fees (shield + unshield)
        shield_fees = results['type_fees'].get('shield', {}).get(token, 0)
        unshield_fees = results['type_fees'].get('unshield', {}).get(token, 0)
        total_fees = shield_fees + unshield_fees
        print(f"  {token:8s}: {format_large_number(amount):>20s} (fees: {format_large_number(total_fees):>15s})")
    
    # Detailed breakdown by transaction type and token
    print("\nAmounts by Transaction Type and Token:")
    # for tx_type in sorted(results['type_amounts'].keys()):
    for tx_type in ["shield", "unshield"]:
        print(f"\n  {tx_type}:")
        for token, amount in sorted(results['type_amounts'][tx_type].items(), key=lambda x: x[1], reverse=True):
            print(f"    {token:8s}: {format_large_number(amount):>20s}")


# ============================================================================
# Main
# ============================================================================

def main():
    """Main analysis function."""
    print("MASP Transaction Analysis")
    print("=" * 80)
    
    # Load data first (we may need it to determine reference height)
    transactions = load_data()
    tokens = load_tokens()
    
    # Get oldest and newest transaction heights
    if transactions:
        oldest_height = min(tx['block_height'] for tx in transactions)
        newest_height = max(tx['block_height'] for tx in transactions)
        print(f"Data range: blocks {oldest_height:,} to {newest_height:,}")
        print(f"Data span: ~{(newest_height - oldest_height) * BLOCK_TIME_SECONDS / 86400:.1f} days")
    else:
        print("Error: No transactions found in data!")
        return
    
    # Determine reference block height
    if REFERENCE_BLOCK_HEIGHT is not None:
        reference_height = REFERENCE_BLOCK_HEIGHT
        print(f"Using provided reference block height: {reference_height:,}")
    else:
        print("Querying current block height...")
        reference_height = get_current_block_height()
        if reference_height is None:
            reference_height = newest_height
            print(f"Using newest block in data as reference: {reference_height:,}")
        else:
            print(f"Current block height: {reference_height:,}")
    
    # Convert reference height to approximate date
    # Note: This is approximate based on current time and block time
    print(f"Block time: {BLOCK_TIME_SECONDS} seconds")
    
    # Analyze each time range
    all_results = []
    for range_str in RANGES:
        results = analyze_time_range(transactions, tokens, reference_height, range_str)
        all_results.append(results)
        print_analysis_results(results)
    
    # Summary comparison
    print("\n" + "=" * 80)
    print("Summary Comparison")
    print("=" * 80)
    
    # Collect all unique tokens across all results
    all_tokens = set()
    for result in all_results:
        all_tokens.update(result['token_amounts'].keys())
    
    # Sort tokens for consistent display (NAM first if present, then alphabetically)
    sorted_tokens = sorted(all_tokens, key=lambda x: (x != 'NAM', x))
    
    # Display summary for each token
    for token in sorted_tokens:
        print(f"\n{token}:")
        print(f"{'Range':>10s} {'Total Txs':>12s} {'Shielded':>20s} {'Unshielded':>20s} {'Fee Value ($)':>20s}")
        print("-" * 85)
        
        for result in all_results:
            total_txs = result['total_txs']
            shielded = result['type_amounts'].get('shield', {}).get(token, 0)
            unshielded = result['type_amounts'].get('unshield', {}).get(token, 0)
            
            # Calculate fee value
            shield_fees = result['type_fees'].get('shield', {}).get(token, 0)
            unshield_fees = result['type_fees'].get('unshield', {}).get(token, 0)
            total_fees = shield_fees + unshield_fees
            
            # Get token price
            token_price = 0.0
            for token_addr, token_info in tokens.items():
                if token_info['name'] == token:
                    token_price = token_info.get('price', 0.0)
                    break
            
            fee_value = total_fees * token_price
            
            print(f"{result['range']:>10s} {total_txs:>12,} "
                  f"{format_large_number(shielded):>20s} "
                  f"{format_large_number(unshielded):>20s} "
                  f"{format_large_number(fee_value):>20s}")
    
    # Calculate total fee value across all tokens and ranges
    print("\n" + "=" * 80)
    print("Total Fee Value Collected (USD)")
    print("=" * 80)
    print(f"\n{'Range':>10s} {'Total Fee Value ($)':>25s} {'Excl. USDC ($)':>25s}")
    print("-" * 65)
    
    for result in all_results:
        total_fee_value = 0.0
        total_fee_value_excl_usdc = 0.0
        
        for token in sorted_tokens:
            # Calculate fee value for this token
            shield_fees = result['type_fees'].get('shield', {}).get(token, 0)
            unshield_fees = result['type_fees'].get('unshield', {}).get(token, 0)
            total_fees = shield_fees + unshield_fees
            
            # Get token price
            token_price = 0.0
            for token_addr, token_info in tokens.items():
                if token_info['name'] == token:
                    token_price = token_info.get('price', 0.0)
                    break
            
            fee_value = total_fees * token_price
            total_fee_value += fee_value
            
            # Exclude USDC and USDN from the alternative calculation
            if token not in ['USDC', 'USDN']:
                total_fee_value_excl_usdc += fee_value
        
        print(f"{result['range']:>10s} {format_large_number(total_fee_value):>25s} {format_large_number(total_fee_value_excl_usdc):>25s}")
    
    print("\n" + "=" * 80)
    print("Analysis complete!")
    print("=" * 80)


if __name__ == "__main__":
    main()

