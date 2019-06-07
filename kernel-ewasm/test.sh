pushd example_contract_2
sh ./build.sh
popd
mocha tests/**/**.js
