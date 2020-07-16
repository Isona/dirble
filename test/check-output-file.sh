#!/bin/bash

# Check the output file against a couple of parameters

# Dirble Scan Report for http://localhost:5000/:
# + http://localhost:5000/302.html (CODE:302|SIZE:209|DEST:http://localhost:5000/)
# + http://localhost:5000/401.html (CODE:401|SIZE:338)
# + http://localhost:5000/403.html (CODE:403|SIZE:234)
# + http://localhost:5000/429.html (CODE:429|SIZE:194)
# + http://localhost:5000/console (CODE:200|SIZE:1985)
#
# Dirble Scan Report for http://localhost:5001/:
# + http://localhost:5001/chunked.html (CODE:200|SIZE:180)

FILE=$1

if [[ "$FILE" == "" ]] ; then
	echo "Usage: ./$0 output-file.txt"
	exit 1
fi

function mismatch() {
	echo "Test failed, see earlier output".
	exit 1
}

# The chunked server should be processed correctly
grep "http://localhost:5001/chunked.html (CODE:200|SIZE:180)" "$FILE" || mismatch

# The tested response codes should be rendered correctly
grep \
	"http://localhost:5000/302.html (CODE:302|SIZE:209|DEST:http://localhost:5000/)" \
	"$FILE" \
	|| mismatch
grep "ttp://localhost:5000/401.html (CODE:401|SIZE:338)" "$FILE" || mismatch
grep "http://localhost:5000/403.html (CODE:403|SIZE:234)" "$FILE" || mismatch
grep "http://localhost:5000/429.html (CODE:429|SIZE:194)" "$FILE" || mismatch

# We don't care about the werkzeug console; that doesn't count as a test case

# If it got this far then all the output should be correct. Print a nice
# message and exit normally
echo "All tests passed! :)"