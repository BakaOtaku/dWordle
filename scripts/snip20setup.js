const {
  EnigmaUtils, Secp256k1Pen, SigningCosmWasmClient, pubkeyToAddress, encodeSecp256k1Pubkey, unmarshalTx, CosmWasmClient
} = require("secretjs");
const { Slip10RawIndex } = require("@iov/crypto");
const { fromUtf8 } = require("@iov/encoding");

const fs = require("fs");

// Load environment variables
// require('dotenv').config();

const customFees = {
  upload: {
    amount: [{ amount: "2000000", denom: "uscrt" }],
    gas: "2000000",
  },
  init: {
    amount: [{ amount: "500000", denom: "uscrt" }],
    gas: "500000",
  },
  exec: {
    amount: [{ amount: "500000", denom: "uscrt" }],
    gas: "500000",
  },
  send: {
    amount: [{ amount: "80000", denom: "uscrt" }],
    gas: "80000",
  },
}

const main = async () => {
    const httpUrl = "http://testnet.securesecrets.org:1317";

  // Use key created in tutorial #2
  const mnemonic = "file shop output chest weather twist useless fancy giant alert blame spoon tornado damage immense portion help unfold grab flush wish saddle swarm kidney";

  // A pen is the most basic tool you can think of for signing.
  // This wraps a single keypair and allows for signing.
  const signingPen = await Secp256k1Pen.fromMnemonic(mnemonic);

  // Get the public key
  const pubkey = encodeSecp256k1Pubkey(signingPen.pubkey);

  // get the wallet address
  const accAddress = pubkeyToAddress(pubkey, 'secret');

  const txEncryptionSeed = EnigmaUtils.GenerateNewSeed();

  const client = new SigningCosmWasmClient(
    httpUrl,
    accAddress,
    (signBytes) => signingPen.sign(signBytes),
    txEncryptionSeed, customFees
  );
  console.log(`Wallet address=${accAddress}`)

  // Upload the wasm of a simple contract
  const wasm = fs.readFileSync("./snip20.wasm");
  console.log('Uploading contract')
  const uploadReceipt = await client.upload(wasm, {});

  // Get the code ID from the receipt
  const codeId = uploadReceipt.codeId;
  console.log("upload receipt ", uploadReceipt);
  // Create an instance of the token contract, minting some tokens to our wallet
  const initMsg = {
    "name":"dwordleTest",
    "symbol":"DWD",
    "decimals":6,
    "prng_seed": Buffer.from("Something really random").toString('base64'),
    "admin": accAddress,
    "initial_balances": [{
      "address": accAddress,
      "amount": "1000000000"
    }
    ]
  }
  const contract = await client.instantiate(codeId, initMsg, "dwordleTest" + Math.ceil(Math.random()*10000));
  console.log('contract: ', contract);

  const contractAddress = contract.contractAddress;

  // Entropy: Secure implementation is left to the client, but it is recommended to use base-64 encoded random bytes and not predictable inputs.
  const entropy = "Another really random thing";


  let handleMsg = { create_viewing_key: {entropy: entropy} };
  console.log('Creating viewing key');
  response = await client.execute(contractAddress, handleMsg);
  console.log('response: ', response);

  // Convert the UTF8 bytes to String, before parsing the JSON for the api key.
  const apiKey = JSON.parse(fromUtf8(response.data)).create_viewing_key.key;

  // Query balance with the api key
  const balanceQuery = {
    balance: {
      key: apiKey,
      address: accAddress
    }
  };
  let balance = await client.queryContractSmart(contractAddress, balanceQuery);

  console.log('My token balance: ', balance);
  //
  // // Transfer some tokens
  handleMsg = {
    transfer:
      {
        owner: accAddress, amount: "1000", recipient: await getAddress(mnemonic, 1)
      }
  };
  console.log('Transferring tokens');
  response = await client.execute(contractAddress, handleMsg);
  console.log('Transfer response: ', response)

  balance = await client.queryContractSmart(contractAddress, balanceQuery);
  console.log('New token balance', balance)
};

// Util to generate another address to send to
async function getAddress(mnemonic, index) {
  const signingPen = await Secp256k1Pen.fromMnemonic(mnemonic, [Slip10RawIndex.normal(index)]);
  const pubkey = encodeSecp256k1Pubkey(signingPen.pubkey);
  return pubkeyToAddress(pubkey, 'secret');
}

main();

/*

Wallet address=secret1hg7lzjrmfaljqedgau87apzdj5ms4kfma4fwyy
Uploading contract
upload receipt  {
  originalSize: 276462,
  originalChecksum: '71403cd955f73ae7bec0f23e5b0ef802050a5d04a5fe63debc3a4537ea1799ad',
  compressedSize: 88225,
  compressedChecksum: '1a12425c17f7ebaf3f0ade36dc8c4980d4bf0519530e1e647aa6d0c76494d69b',
  codeId: 8747,
  logs: [ { msg_index: 0, log: '', events: [Array] } ],
  transactionHash: '640D4639C088A407D41E9E5FF6A918B301F7F4A983E0E37E629EDD397D803247'
}
contract:  {
  contractAddress: 'secret1q7meekm058y0rjj8l2x3tswpfw89an087cml7n',
  logs: [ { msg_index: 0, log: '', events: [Array] } ],
  transactionHash: '12BBB8180C4835E3665B00C1D860AD511AEC183E19AB8C6F17542F484CEFF142',
  data: '0A460A2E2F7365637265742E636F6D707574652E763162657461312E4D7367496E7374616E7469617465436F6E7472616374121407B79CDB6FA1C8F1CA47FA8D15C1C14B8E5ECDE7'
}
Creating viewing key
response:  {
  logs: [ { msg_index: 0, log: '', events: [Array] } ],
  transactionHash: '6917A5A2954F0352D62BEC8E75A5D43C5EC54D8556C8129FC014F2F89EAEA039',
  data: Uint8Array(256) [
    123,  34,  99, 114, 101,  97, 116, 101,  95, 118, 105, 101,
    119, 105, 110, 103,  95, 107, 101, 121,  34,  58, 123,  34,
    107, 101, 121,  34,  58,  34,  97, 112, 105,  95, 107, 101,
    121,  95, 115,  89,  75,  47,  55, 100, 104,  65,  56,  54,
     65,  70,  90, 107,  66,  43,  43,  47,  89,  86,  97,  75,
     97, 105,  83,  81,  87,  56,  86, 115,  53,  43, 118, 109,
     86, 122, 102,  98, 122,  76,  86, 103,  69,  61,  34, 125,
    125,  32,  32,  32,  32,  32,  32,  32,  32,  32,  32,  32,
     32,  32,  32,  32,
    ... 156 more items
  ]
}
My token balance:  { balance: { amount: '1000000000' } }
Transferring tokens
Transfer response:  {
  logs: [ { msg_index: 0, log: '', events: [Array] } ],
  transactionHash: '7A951FF3AD40C1284D69FACF61956DD7C1D3EC523F44E4A36987ABD1CE70207C',
  data: Uint8Array(256) [
    123, 34, 116, 114,  97, 110, 115, 102, 101, 114, 34,  58,
    123, 34, 115, 116,  97, 116, 117, 115,  34,  58, 34, 115,
    117, 99,  99, 101, 115, 115,  34, 125, 125,  32, 32,  32,
     32, 32,  32,  32,  32,  32,  32,  32,  32,  32, 32,  32,
     32, 32,  32,  32,  32,  32,  32,  32,  32,  32, 32,  32,
     32, 32,  32,  32,  32,  32,  32,  32,  32,  32, 32,  32,
     32, 32,  32,  32,  32,  32,  32,  32,  32,  32, 32,  32,
     32, 32,  32,  32,  32,  32,  32,  32,  32,  32, 32,  32,
     32, 32,  32,  32,
    ... 156 more items
  ]
}
New token balance { balance: { amount: '999999000' } }


 */
