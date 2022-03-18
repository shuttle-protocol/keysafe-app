# keysafe-sgx
# env setup
+ follow https://github.com/intel/linux-sgx#build-and-install-the-intelr-sgx-driver
+ install linux-sgx 
# build
+ download baidu sgx-sdk with the name incubator-teaclave-sgx-sdk
+ copy the code into incubator-teaclave-sgx-sdk/samplecode
+ change Makefile SGX_SDK and TOP_DIR according to your environment
+ make SGX_MODE=SW
# build with ks-sgx
+ clone ks-sgx in another directory,
+ cd ks-sgx; make
+ go back to keysafe-sgx
+ change Makefile SGX_SDK and TOP_DIR according to your environment
+ ln -s path-to-ks-sgx/APP ./APP_KS
+ ln -s path-to-ks-sgx/Enclave_KS ./
+ cp path-to-ks-sgx/.so files to ./
+ make clean
+ make
# execute
+ cd bin
+ ln -s ../certs .
+ ./app
# test
+ download postman
+ click "My Workspace", "New", "Websocket Request (Beta)"
+ connect to "wss://127.0.0.1:12345/save"
+ use postman as a web front-end to communicate with enclave.
