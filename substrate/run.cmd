@echo Off
setlocal
pushd substrate
:: abort if pushd fails
if %errorlevel% NEQ 0 goto:eof

:: Run substrate node
cargo build --release && ^
cargo run --release -- purge-chain --dev -y && ^
cargo run --release -- --dev > ..\substrate.txt 2> ..\substrate.err

set run_result=%errorlevel%

:: Pop the directory from the stack regardless of exit status
popd
:: Exit with the appropriate error code
exit /b %run_result%
