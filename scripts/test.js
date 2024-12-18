const StellarSdk = require("stellar-sdk");

// Set up Stellar SDK to use the testnet
const server = new StellarSdk.Server("https://horizon-testnet.stellar.org");
const networkPassphrase = StellarSdk.Networks.TESTNET;

(async () => {
  try {
    // Step 1: Create Issuer and Distribution Accounts
    const issuerKeypair = StellarSdk.Keypair.fromSecret('SCAOJ6XACDYMB7CPRS7DU6Z5WBT3YURKX6XS4RJUQP5QKPKRCB75LTQ5');
    const distributionKeypair = StellarSdk.Keypair.fromSecret('SBE56ND7QNL635XFQFEQBVAUTCQRQJEYJVI3NAS4TYO3KRJL4JRZVAAI');


    const maxSupply = 10000; // Limit to 10,000 NFTs

    console.log("Issuer Public Key:", issuerKeypair.publicKey());
    console.log("Issuer Secret Key:", issuerKeypair.secret());
    console.log("Distribution Public Key:", distributionKeypair.publicKey());
    console.log("Distribution Secret Key:", distributionKeypair.secret());

    
    // Fund the accounts using Friendbot

    /*
    await Promise.all([
      fetch(`https://friendbot.stellar.org?addr=${issuerKeypair.publicKey()}`),
      fetch(`https://friendbot.stellar.org?addr=${distributionKeypair.publicKey()}`),
    ]);

    */

    console.log("Issuer and Distribution accounts funded.");

    // Step 2: Define the NFT Asset
    const nftAsset = new StellarSdk.Asset("NFT001", issuerKeypair.publicKey());

    const baseFee = await server.fetchBaseFee();
    const metadataURI = "ipfs://Qme5BpPrC3JGWepxkuh4SHft2MvU7R4YF4RpP8CmNEUWvF"; // Use a shortened URI or hash

    
    /*
    // Step 3: Create a Trustline for the Distribution Account
    const distributionAccount = await server.loadAccount(distributionKeypair.publicKey());
    

    const trustlineTransaction = new StellarSdk.TransactionBuilder(distributionAccount, {
      fee: baseFee,
      networkPassphrase,
    })
      .addOperation(
        StellarSdk.Operation.changeTrust({
            destination: distributionKeypair.publicKey(),
            asset: nftAsset,
            amount: "10", // Issue 10,000 NFTs upfront
        })
      )
      .setTimeout(30)
      .build();

    trustlineTransaction.sign(distributionKeypair);

    try {
        const trustlineResponse = await server.submitTransaction(trustlineTransaction);
        console.log("Trustline created successfully for 10 NFTs. Hash:", trustlineResponse.hash);
      } catch (error) {
        if (error.response?.data?.extras?.result_codes?.operations?.[0] === 'op_low_reserve') {
          console.error("Error: Account needs more XLM to create trustline");
        } else {
          console.error("Error creating trustline:", error.response?.data || error);
        }
        throw error;
      }

      */
   
  

    // Step 4: Mint the NFT
    
    // Define the minting fee
const mintingFeeXLM = "1"; // 10 XLM fee per NFT

// Step 4: Mint the NFT with fee
const issuerAccount = await server.loadAccount(issuerKeypair.publicKey());

const mintTransaction = new StellarSdk.TransactionBuilder(issuerAccount, {
  fee: baseFee,
  networkPassphrase,
})
  
  .addOperation(
    StellarSdk.Operation.payment({
      destination: distributionKeypair.publicKey(),
      asset: nftAsset,
      amount: "1", // Mint 1 NFT
    })
  )
  .addMemo(StellarSdk.Memo.text(metadataURI))
  .setTimeout(30)
  .build();

mintTransaction.sign(issuerKeypair);

try {
  const mintResponse = await server.submitTransaction(mintTransaction);
  console.log("NFT minted with fee payment. Transaction hash:", mintResponse.hash);
  console.log(`Minting fee paid: ${mintingFeeXLM} XLM`);
} catch (error) {
  if (error.response?.data?.extras?.result_codes?.operations?.[0] === 'op_underfunded') {
    console.error("Error: Account doesn't have enough XLM to pay the minting fee");
  } else {
    console.error("Error minting NFT:", error.response?.data || error);
  }
  throw error;
}

// Optional: Add a function to check the issuer's balance
async function checkIssuerBalance() {
  const account = await server.loadAccount(issuerKeypair.publicKey());
  const balance = account.balances.find(b => b.asset_type === 'native');
  console.log(`Issuer account balance: ${balance.balance} XLM`);
}

// Check balance after minting
await checkIssuerBalance();

// ... existing code ...

// Add function to check NFT balances
async function checkNFTBalances() {
    try {
        // Check Distribution Account Balance
        const distributionAccount = await server.loadAccount(distributionKeypair.publicKey());
        const distributionNFTBalance = distributionAccount.balances.find(b => 
            b.asset_code === nftAsset.code && 
            b.asset_issuer === nftAsset.issuer
        );
        
        console.log("\nNFT Balance Summary:");
        console.log("--------------------");
        console.log("Distribution Account:");
        console.log(`Public Key: ${distributionKeypair.publicKey()}`);
        if (distributionNFTBalance) {
            console.log(`NFT Balance: ${distributionNFTBalance.balance}`);
        } else {
            console.log("No NFTs held");
        }

        // Check Issuer Account Balance
        const issuerAccount = await server.loadAccount(issuerKeypair.publicKey());
        const issuerNFTBalance = issuerAccount.balances.find(b => 
            b.asset_code === nftAsset.code && 
            b.asset_issuer === nftAsset.issuer
        );

        console.log("\nIssuer Account:");
        console.log(`Public Key: ${issuerKeypair.publicKey()}`);
        if (issuerNFTBalance) {
            console.log(`NFT Balance: ${issuerNFTBalance.balance}`);
        } else {
            console.log("No NFTs held");
        }

        // Show XLM balances as well
        const distributionXLMBalance = distributionAccount.balances.find(b => b.asset_type === 'native');
        const issuerXLMBalance = issuerAccount.balances.find(b => b.asset_type === 'native');

        console.log("\nXLM Balances:");
        console.log("-------------");
        console.log(`Distribution Account XLM: ${distributionXLMBalance.balance}`);
        console.log(`Issuer Account XLM: ${issuerXLMBalance.balance}`);

        // Show Trustline Limit
        if (distributionNFTBalance) {
            console.log("\nTrustline Info:");
            console.log("--------------");
            console.log(`Maximum NFT Limit: ${distributionNFTBalance.limit}`);
            console.log(`Available to receive: ${
                distributionNFTBalance.limit - distributionNFTBalance.balance
            }`);
        }

    } catch (error) {
        console.error("Error checking balances:", error.response?.data || error);
    }
}

// Call this function after important operations
// After minting:


// You can also add it after the trustline creation and after any transfers
try {
    // Your existing mint or transfer operation here
    // ...

    // Check balances after the operation
    console.log("\nChecking updated balances after operation:");
    await checkNFTBalances();
} catch (error) {
    console.error("Operation failed:", error.response?.data || error);
}




    /*
    // Step 5: Create a Sell Offer for the NFT
    const updatedDistributionAccount = await server.loadAccount(distributionKeypair.publicKey());

    const sellOfferTransaction = new StellarSdk.TransactionBuilder(updatedDistributionAccount, {
      fee: baseFee,
      networkPassphrase,
    })
      .addOperation(
        StellarSdk.Operation.manageSellOffer({
          selling: nftAsset,
          buying: StellarSdk.Asset.native(), // Sell NFT for XLM
          amount: "1",
          price: "100", // Price: 100 XLM
        })
      )
      .setTimeout(30)
      .build();

    sellOfferTransaction.sign(distributionKeypair);

    const sellOfferResponse = await server.submitTransaction(sellOfferTransaction);
    console.log("Sell offer created:", sellOfferResponse);
*/

  } catch (error) {
    console.error("Error:", error.response?.data || error.message);
  }
})();
