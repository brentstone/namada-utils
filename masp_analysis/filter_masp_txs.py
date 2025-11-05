#!/usr/bin/env python3
"""
Filter and validate MASP transaction data for analysis.

This script reads transformed_masp_txs.json and creates filtered_for_analysis.json
with validated and processed transaction data.
"""

import json
import sys


def filter_masp_txs(input_file='transformed_masp_txs.json', output_file='filtered_for_analysis.json'):
    """
    Filter and validate MASP transactions based on their kind.
    
    Processes shielding and unshielding transfers with validation.
    """
    print(f"Reading {input_file}...")
    
    try:
        with open(input_file, 'r') as f:
            transactions = json.load(f)
    except FileNotFoundError:
        print(f"Error: {input_file} not found")
        sys.exit(1)
    except json.JSONDecodeError as e:
        print(f"Error parsing JSON: {e}")
        sys.exit(1)
    
    filtered_data = []
    stats = {
        'total': len(transactions),
        'shielding': 0,
        'unshielding': 0,
        'shielded_transfer': 0,
        'shielded_transfer_pure': 0,
        'shielded_transfer_fake_unshield': 0,
        'ibc_shielding': 0,
        'ibc_unshielding': 0,
        'skipped': 0,
        'warnings': 0,
        'errors': 0
    }
    
    print(f"Processing {len(transactions)} transactions...")
    
    for tx in transactions:
        wrapper_tx_id = tx.get('wrapper_tx_id')
        block_height = tx.get('block_height')
        fee_payer = tx.get('fee_payer')
        fee_token_address = tx.get('fee_token_address')
        inner_tx_id = tx.get('inner_tx_id')
        inner_tx_kind = tx.get('inner_tx_kind')
        sources = tx.get('sources', [])
        targets = tx.get('targets', [])
        
        # Process based on transaction kind
        if inner_tx_kind == 'shieldedTransfer':
            stats['shielded_transfer'] += 1
            stats['shielded_transfer_pure'] += 1
            continue
        
        elif inner_tx_kind == 'shieldingTransfer' or inner_tx_kind == 'ibcShieldingTransfer' or inner_tx_kind == 'ibcUnshieldingTransfer':
            # Validate structure
            if len(sources) != 1 or len(targets) != 1:
                print(f"WARNING: Unconventional {inner_tx_kind} with {len(sources)} sources "
                      f"and {len(targets)} targets. Inner tx ID: {inner_tx_id}")
                stats['warnings'] += 1
                stats['skipped'] += 1
                continue
            
            source = sources[0]
            target = targets[0]
            
            # Validate amounts match
            if source.get('amount') != target.get('amount'):
                print(f"ERROR: Amount mismatch in shielding tx. Inner tx ID: {inner_tx_id}")
                print(f"  Source amount: {source.get('amount')}, Target amount: {target.get('amount')}")
                stats['errors'] += 1
                stats['skipped'] += 1
                continue
            
            # Validate tokens match
            if source.get('token') != target.get('token'):
                print(f"ERROR: Token mismatch in shielding tx. Inner tx ID: {inner_tx_id}")
                print(f"  Source token: {source.get('token')}, Target token: {target.get('token')}")
                stats['errors'] += 1
                stats['skipped'] += 1
                continue
            
            # Create output entry
            if inner_tx_kind == 'shieldingTransfer':
                tx_type = 'shielding'
            elif inner_tx_kind == 'ibcShieldingTransfer':
                tx_type = 'ibc_shielding'
            else:
                tx_type = 'ibc_unshielding'
            
            entry = {
                'wrapper_tx_id': wrapper_tx_id,
                'block_height': block_height,
                'fee_token_address': fee_token_address,
                'inner_tx_id': inner_tx_id,
                'tx_type': tx_type,
                'address': source.get('owner'),
                'amount': source.get('amount'),
                'token': source.get('token')
            }
            filtered_data.append(entry)
            stats[tx_type] += 1
        
        elif inner_tx_kind == 'unshieldingTransfer':
            # Case 1: Single source and single target. Can either be a shielded transfer with MASP fee payment (skip) or a real unshielding with fee paid by currently transparent tokens.
            if len(sources) == 1 and len(targets) == 1:
                source = sources[0]
                target = targets[0]
                
                # Check if this is a fee payment tx for an actual shielded transfer (skip it)
                if (target.get('owner') == fee_payer and 
                    source.get('amount') == target.get('amount')):
                    stats['shielded_transfer'] += 1
                    stats['shielded_transfer_fake_unshield'] += 1
                # Check if this is a real unshielding with fee paid by currently transparent tokens.
                elif (target.get('owner') != fee_payer and source.get('amount') == target.get('amount')):
                    filtered_data.append(entry)
                    stats['unshielding'] += 1
                else:
                    # Unconventional single-to-single that doesn't match fee payment pattern
                    print(f"WARNING: Unconventional 1-to-1 unshielding tx. Inner tx ID: {inner_tx_id}")
                    stats['warnings'] += 1
                    stats['skipped'] += 1
                continue

            # Case 2: Two sources and two targets, where the fees are paid in a token different from the primary transacted token.
            if len(sources) == 2 and len(targets) == 2:
                source1 = sources[0]
                source2 = sources[1]
                target1 = targets[0]
                target2 = targets[1]
                
                if fee_token_address == source1.get('token'):
                    is_valid = (source1.get('amount') == target1.get('amount') and target1.get('token') == fee_token_address) or (source1.get('amount') == target2.get('amount') and target2.get('token') == fee_token_address)
                elif fee_token_address == source2.get('token'):
                    is_valid = (source2.get('amount') == target1.get('amount') and target1.get('token') == fee_token_address) or (source2.get('amount') == target2.get('amount') and target2.get('token') == fee_token_address)
                else:
                    is_valid = False
                
                if is_valid:
                    filtered_data.append(entry)
                    stats['unshielding'] += 1
                else:
                    print(f"WARNING: Unconventional unshielding tx form with {len(sources)} sources "
                          f"and {len(targets)} targets. Inner tx ID: {inner_tx_id}")
                    stats['warnings'] += 1
                    stats['skipped'] += 1
                
                continue
            
            # Case 3: Other combinations - validate 1 source, 2 targets
            if len(sources) != 1 or len(targets) != 2:
                print(f"WARNING: Unconventional unshielding tx form with {len(sources)} sources "
                      f"and {len(targets)} targets. Inner tx ID: {inner_tx_id}")
                stats['warnings'] += 1
                stats['skipped'] += 1
                continue
            
            source = sources[0]
            
            # Find which target has owner == fee_payer
            fee_target = None
            recipient_target = None
            
            for target in targets:
                if target.get('owner') == fee_payer:
                    fee_target = target
                else:
                    recipient_target = target
            
            if fee_target is None:
                print(f"WARNING: No target with owner == fee_payer in unshielding tx. "
                      f"Inner tx ID: {inner_tx_id}")
                stats['warnings'] += 1
                stats['skipped'] += 1
                continue
            
            if recipient_target is None:
                print(f"WARNING: Both targets have owner == fee_payer in unshielding tx. "
                      f"Inner tx ID: {inner_tx_id}")
                stats['warnings'] += 1
                stats['skipped'] += 1
                continue
            
            # Calculate expected recipient amount
            try:
                source_amount = int(source.get('amount', '0'))
                fee_amount = int(fee_target.get('amount', '0'))
                recipient_amount_expected = source_amount - fee_amount
                recipient_amount_actual = int(recipient_target.get('amount', '0'))
                
                # Validate
                if recipient_amount_expected != recipient_amount_actual:
                    print(f"ERROR: Amount calculation mismatch in unshielding tx. "
                          f"Inner tx ID: {inner_tx_id}")
                    print(f"  Expected: {recipient_amount_expected}, "
                          f"Actual: {recipient_amount_actual}")
                    stats['errors'] += 1
                    stats['skipped'] += 1
                    continue
                
                # Create output entry
                entry = {
                    'wrapper_tx_id': wrapper_tx_id,
                    'block_height': block_height,
                    'fee_token_address': fee_token_address,
                    'inner_tx_id': inner_tx_id,
                    'tx_type': 'unshielding',
                    'address': recipient_target.get('owner'),
                    'amount': str(recipient_amount_actual),
                    'token': source.get('token')
                }
                filtered_data.append(entry)
                stats['unshielding'] += 1
                
            except (ValueError, TypeError) as e:
                print(f"ERROR: Could not parse amounts in unshielding tx. "
                      f"Inner tx ID: {inner_tx_id}. Error: {e}")
                stats['errors'] += 1
                stats['skipped'] += 1
                continue
        
        else:
            print(f"WARNING: Unknown transaction kind '{inner_tx_kind}'. "
                  f"Inner tx ID: {inner_tx_id}")
            stats['warnings'] += 1
            stats['skipped'] += 1
            continue
    
    print(f"\nWriting {len(filtered_data)} filtered transactions to {output_file}...")
    
    with open(output_file, 'w') as f:
        json.dump(filtered_data, f, indent=2)
    
    print(f"\n=== Filtering Complete ===\n")
    print(f"Total MASPtransactions processed: {stats['total']}")
    print(f"Native shielding transactions: {stats['shielding']}")
    print(f"Native unshielding transactions: {stats['unshielding']}")
    print(f"Shielded transfers (total): {stats['shielded_transfer']}")
    print(f"Shielded transfers (transparent gas payment): {stats['shielded_transfer_pure']}")
    print(f"Shielded transfers (shielded gas payment): {stats['shielded_transfer_fake_unshield']}")
    print(f"IBC shielding transfers: {stats['ibc_shielding']}")
    print(f"IBC unshielding transfers: {stats['ibc_unshielding']}")
    print(f"Other skipped transactions: {stats['skipped']}")
    print(f"Warnings: {stats['warnings']}")
    print(f"Errors: {stats['errors']}")
    print(f"\nOutput: {len(filtered_data)} transactions in {output_file}")


if __name__ == '__main__':
    filter_masp_txs()

