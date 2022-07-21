# UZH-Group7 Code Running Guide

The following explains the two ways to run our smart contract, the easy local way, or the difficult way of deploying to the blockchain. Each section has installable files that are necessary, with section 2 requiring the installable from section 1.


1. Running the contract locally (easy) (This follows the steps from https://developer.concordium.software/en/mainnet/smart-contracts/guides/contract-dev-guides.html which is the official guide, if you are struggling here please refer to it)

    1. Installables
        a. Install rustup (rust and cargo) https://rustup.rs/

        b. set the rustup target using "rustup target add wasm32-unknown-unknown"

        c. Install cargo concordium from "https://developer.concordium.software/en/mainnet/net/installation/downloads.html#downloads" and rename it to "cargo-concordium.exe" For windows, or "cargo-concordium" for linux/mac. for mac you must also make it executable using "chmod +x path/to/cargo-concordium"

        d. add this file to your PATH. Easiest place is the .cargo directory "%HOMEPATH%\.cargo\bin\ for Windows, and $HOME/.cargo/bin for MacOS/Linux."

    2. Running
        a. You should now be able to compile the contract with "cargo concordium build"

        b. running "cargo test" will execute the tests created in the /src/lib.rs file

        c. By default in the contract initialization area of the code, 
            
                - Valid Nationalies are "CH" (Switzerland)
                - Valid Date of Birth is between "19000101u64" and "20040101u64"
                - No previous votes have occured

        d. If you make your way to the test area of the code (the bottom of the lib.rs file), you will notice there are two areas to change the input parameters to force an error/invalid ID. They are labelled with comments "INPUT TESTING 1" and "INPUT TESTING 2" and follow the instructions to change the inputs

2. Running the Smart Contracts on the TestNet (Hard, takes a long time for setup)

    1. Installables
        a. Installing and Running a node is a long installation process, so I will redirect you instead to link of the official documentation describing it "https://developer.concordium.software/en/mainnet/net/nodes/node-requirements.html". Run the node at this point so it can catchup to the most recent block, otherwise you cant do any of the deployment steps later.

        b. Install the concordium-client from "https://developer.concordium.software/en/mainnet/net/installation/downloads-testnet.html#concordium-node-and-client-download-testnet" and similarly rename it to "concordium-client.exe" for windows and the same steps as for cargo concordium follow.

        c. You can run the concordium-client locally in a folder with explicit path paramters, or you can add it to a folder on the PATH like for cargo-concordium

        d. Follow the steps here "https://developer.concordium.software/en/mainnet/net/installation/downloads-testnet.html" to install the testnet mobile wallet, and then follow the steps here "https://developer.concordium.software/en/mainnet/net/mobile-wallet/setup-mobile-wallet.html#setup-mobile-wallet" to create a testnet ID. Just like when you ran it locally, ensure that your ID is for what you want to test. A.K.A put in a valid Nationality/residence/Age etc.

        e. After your ID is verified, you can create a new Account on your wallet and choose which information to make public to be read by the smart contract, ensure you have revealed the necessary data for the contract

    2. Setup 

        a. If you are running your node on a separate instance "https://developer.concordium.software/en/mainnet/smart-contracts/tutorials/piggy-bank/preparing.html" go to the "Syncing a testnet node" section to ensure you concordium-client can talk to your node. If you are running it locally like me you can continue

        b. if you added concordium-client to your path you can run commands with "concordium-client" without a ./ prefix.

        c. Run "concordium-client consensus status --grpc-port 10001" to show the testnet node data, ensure that the last finalized block height has reached the the recent node

        d. On your mobile app choose the "backup" option and export your wallet backup to the same directory as your concordium-client, or if it is on your path, put it in any folder on your computer.

        e. Run "concordium-client config account import ./concordium-backup.concordiumwallet" to import your account data to the concordium-client, you must use the password from the backup creation to confirm this and all future transactions

    3. Deploying and Running

        a. IMPORTANT - Concordium has let us know that the testnet wallet does not easily allow checking date of birth fields, it is possible but out of scope for this project. Please follow these steps to make sure the contract is runnable on the testnet:

            -Open the lib.rs folder and navigate to both the "vote_no" and "vote_yes" functions of the contract
            -Comment out the following agepolicy check in both functions:

                   // Age Policy check, COMMENT THIS OUT FOR THE TESTNET DEPLOYMENT
                    ensure!(
                        host.state().age_policy.is_satisfied::<RC>(ctx.policies()),
                        ReceiveError::AgePolicyViolation
                    );
            -Now you can continue witht he next steps

        1. Using an already deployed contract:

            I am not sure how the testnet works but there is a chance that you can use the instances of the contracts we already deployed, if this does not work you will sadly have to do the whole process below. If it does work, skip to step e. below using "voting_ageless_instance" as the <NameOfNewInstanceOfContract>. A.K.A this is the instance of the contract we already deployed

        
        2. Deploying the contract:

            a. First you must compile the contract into wasm format using: 
                "cargo concordium build --out <Name>.wasm.v1"

            b. Put the wasm file into the directory that can use the concordium-client commands

            c. To deploy the contract onto the testnet use 
                "concordium-client module deploy <Name>.wasm.v1 --sender <accountName> --name <moduleName> --grpc-port 10001"

                You will get a refernce to the module if successful. e.g "Module reference 1d1bcf48f5ce836c4deb574eb63cb353eb83fd38948650f47b7f7fa16979bdea was successfully named <moduleName>."

            d. Now the contract must be initialized with an initial transaction called the state init part of the contract
                "concordium-client contract init <moduleName> --sender <accountName> --contract <NameofContractinlib.rs> --name <NameOfNewInstanceOfContract> --energy 5000 --grpc-port 10001"

            e. Now we can begin interacting with the contract with the following commands

                e1. Updating a Contract (Voting Process)
                    To vote you must run the following command:

                        "concordium-client contract update <NameOfNewInstanceOfContract> --entrypoint vote_yes --energy 5000 --sender <account-name> --grpc-port 10001" (replace "vote_yes" with "vote_no" for a no vote)

                        If successful you will see "Successfully updated contract instance {"index":729,"subindex":0} ('voting_ageless_instance') using the function 'vote_yes'."

                e2. Viewing a Contract's Results (Total Vote Checking)
        
                    To view the total votes run 
                    
                        "concordium-client contract invoke <NameOfNewInstanceOfContract> --entrypoint check_votes --energy 5000 --grpc-port 10001"

                        You will see something like this
                        "Invocation resulted in success:
                            - Energy used: 765 NRG
                            - Return value (raw):
                            [2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]"

                        The 2 there represents 2 different accounts voting for yes.
                        "[1,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0]" would represent one yes and no vote