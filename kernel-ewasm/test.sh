pushd example_contract_2
./build.sh
popd
mocha tests/**/**.js
