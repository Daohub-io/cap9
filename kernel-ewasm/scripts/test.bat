call scripts\build.bat && ^
cargo test && ^
call .\node_modules\.bin\mocha -r ts-node\register --recursive tests\**.ts tests\**\**.ts
