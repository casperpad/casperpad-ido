import { config } from "dotenv";
config();
import {
  Keys,
  CasperClient,
} from "casper-js-sdk";
import { ERC20Client } from "casper-erc20-js-client";
import { BigNumberish } from '@ethersproject/bignumber';
import FactoryClient from "./client/FactoryClient";
import { getAccountInfo, getAccountNamedKeyValue, getDeploy } from "./utils";
import IDOClient from "./client/IDOClient";
import { BiddingToken, CLBiddingToken, CLBiddingTokenBytesParser } from "./clvalue";

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

const KEYS = Keys.Ed25519.parseKeyFiles(
  `${MASTER_KEY_PAIR_PATH}/public_key.pem`,
  `${MASTER_KEY_PAIR_PATH}/secret_key.pem`
);

const testFactory = async () => {
  const factoryContract = new FactoryClient(
    NODE_ADDRESS!,
    CHAIN_NAME!,
    EVENT_STREAM_ADDRESS!
  );
  const installDeployHash = await factoryContract.install(KEYS, "ido_factory", KEYS.publicKey.toAccountHashStr(), 10000, INSTALL_PAYMENT_AMOUNT!, FACTORY_CONTRACT);


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

  const contractName = "ACME Token";

  const installDeployHash = await erc20.install(
    KEYS, // Key pair used for signing 
    contractName, // Name of the token
    "ACME", // Token Symbol
    "11", // Token decimals
    "1000000000000000", // Token supply
    "200000000000", // Payment amount
    ERC20_CONTRACT // Path to WASM file
  );


  console.log(`... Contract installation deployHash: ${installDeployHash}`);

  await getDeploy(NODE_ADDRESS!, installDeployHash);


  console.log(`... Contract installed successfully.`);

  const casperClient = new CasperClient(NODE_ADDRESS!);

  let accountInfo = await getAccountInfo(casperClient, KEYS.publicKey);

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

  const factory_contract = `contract-7e40f5ab6320be2ca33e016aa6077bc06b112f65e24a22a9537367abf8bba61a`;

  const info =
    "{\n  \"name\":\"The Swappery\",\n  \"info\":\"The Coolest DEX on Casper Network\"\n}";

  const auctionStartTime = Date.now();
  const auctionEndTime = Date.now() + 5000000;
  const launchTime = Date.now() + 5000000 * 2;

  const auctionToken = "contract-c2402c3d88b13f14390ff46fde9c06b8590c9e45a9802f7fb8a2674ff9c1e5b1";

  const auctionTokenPrice = 50000;

  const auctionTokenCapacity = 20000000;

  let schedules = new Map<number, BigNumberish>([[Date.now(), 120]]);
  let biddingToken: BiddingToken = { price: 2000 };

  const installDeployHash = await IDOContract.install(KEYS, "casper_ido", factory_contract, info, auctionStartTime, auctionEndTime, launchTime, auctionTokenPrice, auctionTokenCapacity, biddingToken, schedules, INSTALL_PAYMENT_AMOUNT!, IDO_CONTRACT);


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


testIDO();

// testFactory();