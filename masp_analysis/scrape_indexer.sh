#!/bin/zsh

# --- Configuration ---
# Set the starting offset and page size
offset=0
size=1000
file="masp_txs.json" # Using .json since it's JSON data

# API endpoint
base_url="https://indexer.mainnet.siuuu.click/api/v1/chain/wrapper/recent"

# --- Initialization ---
# Create the output file if it doesn't exist
touch "${file}"
echo "Starting scrape... output will be in ${file}"
echo "[" > "${file}" # Start with an opening bracket for a valid JSON array

first_entry=true

# --- Main Loop ---
while true; do
    echo "Fetching ${size} items from offset ${offset}..."

    # Construct the full URL
    # We assume the parameter is named 'offset'
    url="${base_url}?offset=${offset}&size=${size}&kind=shieldedTransfer&kind=unshieldingTransfer&kind=shieldingTransfer"
    
    # Make the request
    # -s for silent (no progress meter)
    # -f for fail-fast (don't output error pages if the server fails)
    #
    # We pipe to 'jq .' to validate and format the JSON.
    # 'jq -c .[]' might be better if the API returns an array of objects
    # and you want to store them one-per-line.
    # Let's check what 'jq .' does with an empty array.
    # `echo "[]" | jq .` outputs "[]".
    #
    # We store the response in a variable to check it.
    new_data=$(curl -sf "${url}" | jq '.')
    
    # Check if curl or jq failed, or if the API returned no data (null or empty array [])
    if [ -z "${new_data}" ] || [ "${new_data}" = "null" ] || [ "${new_data}" = "[]" ]; then
        echo "No more data received from the API (or API error)."
        break
    fi

    # Append the new, valid JSON data to the file
    # We pipe through `jq -c '.[]'` to extract each item from the
    # returned array and print it on its own line.
    # For each object, prepend a comma if it's not the first entry
    if [ "$first_entry" = true ]; then
        # For the first batch, output first item without comma, rest with comma
        echo "${new_data}" | jq -c '.[]' | head -n1 >> "${file}"
        echo "${new_data}" | jq -c '.[]' | tail -n+2 | sed 's/^/,/' >> "${file}"
        first_entry=false
    else
        # For subsequent batches, prepend comma to all items
        echo "${new_data}" | jq -c '.[]' | sed 's/^/,/' >> "${file}"
    fi

    # --- THIS IS THE CRITICAL PAGINATION STEP ---
    # Increment the offset for the next page
    ((offset += size))
    
    # Optional: Add a small delay to be polite to the API
    sleep 1
done

echo "]" >> "${file}" # Close the JSON array
echo "Scraping complete. Final data is in ${file}."
echo "Note: The file is a JSON array, with each object on a new line."

