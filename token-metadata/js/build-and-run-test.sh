cd ../../
build.sh token-metadata
cd token-metadata/js
yarn api:gen
amman stop
amman start &
sleep 5
run-test.sh $1