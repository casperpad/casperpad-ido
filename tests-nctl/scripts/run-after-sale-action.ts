import { config } from "dotenv";
// config();
config({ path: ".env.test.local" });
import { CasperClient, Keys } from "casper-js-sdk";
import { BigNumber, parseFixed } from "@ethersproject/bignumber";

import IDOClient from "./client/IDOClient";
import { getAccountInfo, getAccountNamedKeyValue, getDeploy } from "./utils";

const { NODE_ADDRESS, EVENT_STREAM_ADDRESS, CHAIN_NAME, MASTER_KEY_PAIR_PATH } =
  process.env;

const private_key = Keys.Ed25519.parsePrivateKeyFile(
  `${MASTER_KEY_PAIR_PATH}/secret_key.pem`
);
const public_key = Keys.Ed25519.privateToPublicKey(private_key);

const KEYS = Keys.Ed25519.parseKeyPair(public_key, private_key);

const DEFAULT_RUN_ENTRYPOINT_PAYMENT = "50000000000";

const addOrders = async () => {
  const idoContract = new IDOClient(
    NODE_ADDRESS!,
    CHAIN_NAME!,
    EVENT_STREAM_ADDRESS!
  );
  const casperClient = new CasperClient(NODE_ADDRESS!);

  const idoContractHash = await getAccountNamedKeyValue(
    casperClient,
    KEYS.publicKey,
    `casper_ido_contract_hash`
  );

  await idoContract.setContractHash(idoContractHash.slice(5));
  const orders: Map<string, BigNumber> = new Map();
  const account =
    "account-hash-f2af240a5aa234d6e295ff65b011126dc002f655b1034f869f38b7b2ba60e450";
  const amount = parseFixed("100", 9);
  orders.set(account, amount);
  const deployHash = await idoContract.addOrders(
    KEYS,
    orders,
    DEFAULT_RUN_ENTRYPOINT_PAYMENT
  );
  console.log({ deployHash });
  await getDeploy(NODE_ADDRESS!, deployHash);
};

addOrders();
