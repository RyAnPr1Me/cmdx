#!/bin/bash
# Example Linux shell script
echo "Starting Linux script execution..."
echo

# List directory contents
echo "Listing current directory:"
ls -l

# Create a test file
echo "Hello from Linux" > test_output.txt

# Display file contents
echo
echo "File contents:"
cat test_output.txt

# Cleanup
rm test_output.txt

echo
echo "Script completed successfully!"
