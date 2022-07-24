import { config } from "dotenv";
// config();
config({ path: ".env.test.local" });
// config({ path: ".env.production.local" });
import {
  CasperClient,
  CLValueBuilder,
  decodeBase16,
  Keys,
} from "casper-js-sdk";
import { parseFixed } from "@ethersproject/bignumber";
import { ERC20Client } from "casper-erc20-js-client";

// import kunft from "./tiers/casper/kunft.json";
import kunft from "./tiers/casper-test/kunft.json";

import IDOClient from "./client/IDOClient";
import { getAccountNamedKeyValue, getDeploy } from "./utils";

const {
  NODE_ADDRESS,
  EVENT_STREAM_ADDRESS,
  CHAIN_NAME,
  MASTER_KEY_PAIR_PATH,
  DEFAULT_RUN_ENTRYPOINT_PAYMENT,
} = process.env;

const private_key = Keys.Ed25519.parsePrivateKeyFile(
  `${MASTER_KEY_PAIR_PATH}/secret_key.pem`
);
const public_key = Keys.Ed25519.privateToPublicKey(private_key);

const KEYS = Keys.Ed25519.parseKeyPair(public_key, private_key);

const setAuctionToken = async () => {
  const idoContract = new IDOClient(
    NODE_ADDRESS!,
    CHAIN_NAME!,
    EVENT_STREAM_ADDRESS!
  );
  const casperClient = new CasperClient(NODE_ADDRESS!);

  const { name: idoName } = kunft.info;

  const idoContractHash = await getAccountNamedKeyValue(
    casperClient,
    KEYS.publicKey,
    `${idoName}_ido_contract_hash`
  );

  console.log({ idoContractHash });

  await idoContract.setContractHash(idoContractHash.slice(5));

  const erc20 = new ERC20Client(
    NODE_ADDRESS!,
    CHAIN_NAME!,
    EVENT_STREAM_ADDRESS!
  );
  const { name, capacity, decimals } = kunft.info.token;
  const erc20ContractHash = await getAccountNamedKeyValue(
    casperClient,
    KEYS.publicKey,
    `${name}_contract_hash`
  );

  console.log({ erc20ContractHash });

  await erc20.setContractHash(erc20ContractHash.slice(5));

  let idoContractPackageHash = await getAccountNamedKeyValue(
    casperClient,
    KEYS.publicKey,
    `KUNFT Marketplace_ido_contract_package_hash`
  );

  console.log({ idoContractPackageHash });

  const auctionTokenCapacity = parseFixed(capacity.toString(), decimals);
  let deployHash = await erc20.approve(
    KEYS,
    CLValueBuilder.byteArray(decodeBase16(idoContractPackageHash.slice(5))),
    auctionTokenCapacity.toString(),
    DEFAULT_RUN_ENTRYPOINT_PAYMENT!
  );
  console.log(`ERC20 Approve deploy hash: ${deployHash}`);
  await getDeploy(NODE_ADDRESS!, deployHash);
  console.log("Approved to contract.");

  deployHash = await idoContract.setAuctionToken(
    KEYS,
    `contract-${erc20ContractHash.slice(5)}`,
    auctionTokenCapacity,
    DEFAULT_RUN_ENTRYPOINT_PAYMENT!
  );

  console.log(`setAuctionToken deploy hash: ${deployHash}`);
  await getDeploy(NODE_ADDRESS!, deployHash);
  console.log("setAuctionToken done");
};

const runPresaleActions = async () => {
  await setAuctionToken();
};

runPresaleActions();
