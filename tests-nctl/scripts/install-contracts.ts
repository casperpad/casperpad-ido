import { config } from "dotenv";
// config();
// config({ path: ".env.test.local" });
config({ path: ".env.production.local" });
import { Keys, CasperClient } from "casper-js-sdk";
import { ERC20Client } from "casper-erc20-js-client";
import { BigNumberish, parseFixed } from "@ethersproject/bignumber";
import { getAccountNamedKeyValue, getDeploy } from "./utils";
import IDOClient from "./client/IDOClient";
import kunft from "./tiers/casper/kunft.json";

const {
  NODE_ADDRESS,
  EVENT_STREAM_ADDRESS,
  CHAIN_NAME,
  MASTER_KEY_PAIR_PATH,
  INSTALL_PAYMENT_AMOUNT,
  IDO_CONTRACT,
  ERC20_CONTRACT,
} = process.env;

const private_key = Keys.Ed25519.parsePrivateKeyFile(
  `${MASTER_KEY_PAIR_PATH}/secret_key.pem`
);
const public_key = Keys.Ed25519.privateToPublicKey(private_key);

const KEYS = Keys.Ed25519.parseKeyPair(public_key, private_key);

const deployERC20 = async () => {
  const erc20 = new ERC20Client(
    NODE_ADDRESS!,
    CHAIN_NAME!,
    EVENT_STREAM_ADDRESS!
  );

  const { name, symbol, decimals, capacity } = kunft.info.token;
  const totalSupply = parseFixed(capacity.toString(), decimals);
  console.log(`Deploying ${name} ...`);
  const installDeployHash = await erc20.install(
    KEYS, // Key pair used for signing
    name, // Name of the token
    symbol, // Token Symbol
    decimals.toString(), // Token decimals
    totalSupply.toString(), // Token supply
    "80000000000", // Payment amount
    ERC20_CONTRACT! // Path to WASM file
  );

  console.log({ installDeployHash });

  await getDeploy(NODE_ADDRESS!, installDeployHash);

  console.log(`... ${name} installed successfully.`);

  const casperClient = new CasperClient(NODE_ADDRESS!);

  const contractHash = await getAccountNamedKeyValue(
    casperClient,
    KEYS.publicKey,
    `${name}_contract_hash`
  );

  console.log({ contractHash });

  await erc20.setContractHash(contractHash.slice(5));
  const storedName = await erc20.name();
  const storedTotalSupply = await erc20.totalSupply();
  console.log({ storedName, storedTotalSupply });
};

const deployIDO = async () => {
  console.log("Deploying IDO Contract...");
  const IDOContract = new IDOClient(
    NODE_ADDRESS!,
    CHAIN_NAME!,
    EVENT_STREAM_ADDRESS!
  );
  const casperClient = new CasperClient(NODE_ADDRESS!);

  const {
    token,
    schedules: schedulesInfo,
    startTime,
    endTime,
    name,
  } = kunft.info;

  const auctionTokenPrice = parseFixed(token.price.toString(), 9);
  const auctionTokenCapacity = parseFixed(
    token.capacity.toString(),
    token.decimals
  );

  const schedules = new Map<number, BigNumberish>([]);
  schedulesInfo.forEach((schedule) => {
    schedules.set(schedule.time, schedule.percent * 10 ** 2);
  });

  const treasuryWallet = `account-hash-c3f7b56fcf432bd759c9f81ed32d34a46b9639175cf54192d97db11ddfc0b040`;

  const payToken = undefined; // payment is CSPR

  const contractName = `${name}_ido`;

  const installDeployHash = await IDOContract.install(
    KEYS,
    contractName,
    startTime,
    endTime,
    auctionTokenPrice,
    auctionTokenCapacity,
    schedules,
    treasuryWallet,
    INSTALL_PAYMENT_AMOUNT!,
    IDO_CONTRACT!,
    payToken
  );

  console.log({ installDeployHash });

  await getDeploy(NODE_ADDRESS!, installDeployHash);

  console.log(`... Contract installed successfully.`);

  const contractHash = await getAccountNamedKeyValue(
    casperClient,
    KEYS.publicKey,
    `${contractName}_contract_hash`
  );

  console.log({ contractHash });
};

const deploy = async () => {
  await deployERC20();
  await deployIDO();
};

// deployERC20();

deployIDO();

// deploy();
