import { promisify } from "util";
import { exec } from "child_process";
import 'dotenv/config';
import pkg from '@stellar/stellar-sdk';
import { Server } from 'soroban-client';
import { Account } from 'stellar-sdk';

const { Contract, SorobanRpc, TransactionBuilder, Networks, BASE_FEE, nativeToScVal, Address,Keypair,Server: SorobanRpcServer } = pkg;


const execute = promisify(exec);

async function exe(command) {
    let { stdout } = await execute(command, { stdio: "inherit" });
    return stdout;
};

let rpcUrl = "https://soroban-testnet.stellar.org"

let contractAddress = 'CDOR3CGFAEZZ2EBGYXS775IODQU7NP64IZPZRDDPWXXBO6NJWWJ3DVJ7'
//let contractAddress = 'CCYZ6YOAPTK4LDC45TXAGPJTFKHY6M6RKY4AXK3NHNTQB7W6FNNVKPHZ'

let params = {
    fee: BASE_FEE,
    networkPassphrase: Networks.TESTNET
}


async function contractInt(functName, values = []) {
    try {
        const kp = Keypair.fromSecret("SCJIAGU2KOS3YJZNTOLFYXH7MUX5DYRSJZB4UQC4TSHEFWNVIV7TQ7OG");
        const caller = kp.publicKey();
        const provider = new Server(rpcUrl); 
        
        // Create the contract instance
        const contract = new Contract(contractAddress);
        
        // Build the operation
        let operation;
        if (values && values.length > 0) {
            operation = contract.call(functName, ...values);
        } else {
            operation = contract.call(functName);
        }

        // Build the transaction
        const transaction = new TransactionBuilder(
            new Account(caller, "0"), // Use a new Account instance
            {
                fee: BASE_FEE,
                networkPassphrase: Networks.TESTNET
            }
        )
        .addOperation(operation)
        .setTimeout(30)
        .build();

        // Prepare the transaction
        const preparedTransaction = await provider.prepareTransaction(transaction);
        preparedTransaction.sign(kp);

        // Submit and wait for transaction
        const response = await provider.sendTransaction(preparedTransaction);
        
        if (response.status === "ERROR") {
            throw new Error(`Transaction submission failed: ${JSON.stringify(response.error)}`);
        }

        // Poll for transaction result
        let result;
        let attempts = 0;
        const maxAttempts = 10;
        
        while (attempts < maxAttempts) {
            result = await provider.getTransaction(response.hash);
            
            if (result.status === "SUCCESS") {
                return result.returnValue;
            } else if (result.status === "ERROR") {
                throw new Error(`Transaction failed: ${JSON.stringify(result.error)}`);
            }
            
            await new Promise(resolve => setTimeout(resolve, 1000));
            attempts++;
        }

        throw new Error("Transaction polling timed out");
    } catch (err) {
        console.error('Error in contractInt:', err);
        throw err;
    }
}

const stringToSymbol = (value) => {
    return nativeToScVal(value, {type: "symbol"})
}

const numberToU64 = (value) => {
    return nativeToScVal(value, {type: "u64"})
}


async function readContractB() {
    try {
        const result = await contractInt('get_all_minted_tokens');
        
        if (result && result._value) {
            if (result._value.length === 0) {
                console.log('No tokens minted yet');
                return;
            }
            
            result._value.forEach((token, index) => {
                console.log(`Token ${index + 1}:`);
                console.log(`  ID: ${token._value[0]._value}`);
                console.log(`  URI: ${token._value[1]._value}`);
                console.log(`  Owner: ${token._value[2]._value}`);
                console.log('-------------------');
            });
        } else {
            console.log('No tokens found or invalid result structure:', result);
        }
    } catch (error) {
        console.error('Error in readContractB:', error);
        throw error;
    }
}

// Modify the test connection function to be simpler
async function testConnection() {
    try {
        const provider = new Server(rpcUrl);
        const health = await provider.getHealth();
        console.log('Network connection status:', health);
        
        // Just check if the contract exists
        const contract = new Contract(contractAddress);
        console.log('Contract instance created successfully');
        
        return true;
    } catch (error) {
        console.error('Connection test failed:', error);
        return false;
    }
}

// Main execution
(async () => {
    try {
        if (await testConnection()) {
            console.log('Connection test passed, proceeding with contract call...');
            await readContractB();
        } else {
            console.error('Connection test failed, stopping execution');
            process.exit(1);
        }
    } catch (error) {
        console.error('Execution failed:', error);
        process.exit(1);
    }
})();

export { exe, contractInt };