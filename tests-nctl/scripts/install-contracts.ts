import { config } from "dotenv";
config();
// config({ path: '.env.development.local' });
import {
  Keys,
  CasperClient,
} from "casper-js-sdk";
import { ERC20Client } from "casper-erc20-js-client";
import { BigNumberish } from '@ethersproject/bignumber';
import FactoryClient from "./client/FactoryClient";
import { getAccountInfo, getAccountNamedKeyValue, getDeploy } from "./utils";
import IDOClient from "./client/IDOClient";
import { BiddingToken } from "./clvalue";
import { Info, Convert } from "./types";
// Path to contract to be installed.
const IDO_CONTRACT = "/home/master/workspace/casperpad-ido/target/wasm32-unknown-unknown/release/casper_ido_contract.wasm";
const FACTORY_CONTRACT = "/home/master/workspace/casperpad-ido/target/wasm32-unknown-unknown/release/factory_contract.wasm";
const ERC20_CONTRACT = "/home/master/workspace/casperpad-ido/tests/wasm/erc20_token.wasm";

const {
  NODE_ADDRESS,
  EVENT_STREAM_ADDRESS,
  CHAIN_NAME,
  MASTER_KEY_PAIR_PATH,
  INSTALL_PAYMENT_AMOUNT
} = process.env;

const private_key = Keys.Ed25519.parsePrivateKeyFile(`${MASTER_KEY_PAIR_PATH}/secret_key.pem`);
const public_key = Keys.Ed25519.privateToPublicKey(private_key);

const KEYS = Keys.Ed25519.parseKeyPair(public_key, private_key);

const testFactory = async () => {
  const factoryContract = new FactoryClient(
    NODE_ADDRESS!,
    CHAIN_NAME!,
    EVENT_STREAM_ADDRESS!
  );
  const installDeployHash = await factoryContract.install(KEYS, "ido_factory", KEYS.publicKey.toAccountHashStr(), 10000, "80000000000", FACTORY_CONTRACT);


  console.log(`... Contract installation deployHash: ${installDeployHash}`);

  await getDeploy(NODE_ADDRESS!, installDeployHash);


  console.log(`... Contract installed successfully.`);

  const casperClient = new CasperClient(NODE_ADDRESS!);

  let accountInfo = await getAccountInfo(casperClient, KEYS.publicKey);

  console.log(`... Account Info: `);
  console.dir(accountInfo, { depth: null });

  const contractHash = await getAccountNamedKeyValue(casperClient,
    KEYS.publicKey,
    `ido_factory_contract_hash`
  );

  console.log(`... Contract Hash: ${contractHash}`);

};

const testERC20 = async () => {
  const erc20 = new ERC20Client(
    NODE_ADDRESS!,
    CHAIN_NAME!,
    EVENT_STREAM_ADDRESS!
  );

  const contractName = "Test Swappery Token";

  const installDeployHash = await erc20.install(
    KEYS, // Key pair used for signing 
    contractName, // Name of the token
    "tSWPR", // Token Symbol
    "9", // Token decimals
    "1000000000000000", // Token supply
    "80000000000", // Payment amount
    ERC20_CONTRACT // Path to WASM file
  );


  console.log(`... Contract installation deployHash: ${installDeployHash}`);

  await getDeploy(NODE_ADDRESS!, installDeployHash);


  console.log(`... Contract installed successfully.`);

  const casperClient = new CasperClient(NODE_ADDRESS!);

  let accountInfo = await getAccountInfo(casperClient, KEYS.publicKey);

  console.dir(accountInfo, { depth: null });
  const contractHash = await getAccountNamedKeyValue(casperClient,
    KEYS.publicKey,
    `${contractName}_contract_hash`
  );

  console.log(`... Contract Hash: ${contractHash}`);

  await erc20.setContractHash(contractHash);
}

const testIDO = async () => {
  const IDOContract = new IDOClient(
    NODE_ADDRESS!,
    CHAIN_NAME!,
    EVENT_STREAM_ADDRESS!
  );
  const casperClient = new CasperClient(NODE_ADDRESS!);

  const factoryContractHash = await getAccountNamedKeyValue(casperClient,
    KEYS.publicKey,
    `ido_factory_contract_hash`
  );
  const factory_contract = `contract-${factoryContractHash.slice(5)}`;

  const info2: Info = {
    "name": "The Swappery",
    "description": "The Coolest DEX on Casper Network",
    "token": {
      "name": "Test Swappery Token",
      "symbol": "tSWPR",
      "decimals": 9,
      "price": 0.01
    },
    "links": {
      "logo": "https://avatars.githubusercontent.com/u/49560738?s=200&v=4",
      "webpack": "https://",
      "twitter": "",
      "outline": "",
      "telegram": ""
    }
  };

  const info = Convert.infoToJson(info2);

  const auctionStartTime = Date.now() + 7200;
  const auctionEndTime = Date.now() + 18000;
  const launchTime = Date.now() + 432000;

  const auctionToken = "contract-9965d53100f27b29b45ebd4ea70d0bb4fd3715a9df91bf9e85adbd96dda68869";

  const auctionTokenPrice = "100000000";

  const auctionTokenCapacity = "2000000000000";

  const schedules = new Map<number, BigNumberish>([[Date.now(), 1250], [Date.now() + 20000, 8750]]);
  const biddingToken: BiddingToken = { price: undefined };

  const installDeployHash = await IDOContract.install(KEYS,
    "casper_ido",
    factory_contract,
    info,
    auctionStartTime,
    auctionEndTime,
    launchTime,
    auctionTokenPrice,
    auctionTokenCapacity,
    biddingToken,
    schedules,
    INSTALL_PAYMENT_AMOUNT!,
    IDO_CONTRACT);

  console.log(`... Contract installation deployHash: ${installDeployHash}`);

  await getDeploy(NODE_ADDRESS!, installDeployHash);

  console.log(`... Contract installed successfully.`);

  let accountInfo = await getAccountInfo(casperClient, KEYS.publicKey);

  console.log(`... Account Info: `);
  console.dir(accountInfo, { depth: null });

  const contractHash = await getAccountNamedKeyValue(casperClient,
    KEYS.publicKey,
    `ido_factory_contract_hash`
  );

  console.log(`... Contract Hash: ${contractHash}`);

};

const test = async () => {
  await testERC20();
  await testFactory();
  await testIDO();
}

// testERC20();

testIDO();

// testFactory();

// test();