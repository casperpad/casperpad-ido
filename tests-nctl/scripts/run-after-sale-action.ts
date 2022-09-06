import { config } from "dotenv";
// config();
config({ path: ".env.test.local" });
import { CasperClient, Keys } from "casper-js-sdk";
import { BigNumberish, parseFixed } from "@ethersproject/bignumber";

import IDOClient from "./client/IDOClient";
import { getAccountNamedKeyValue, getDeploy } from "./utils";

import { investors } from "./tiers/casper/converted.json";

const {
  NODE_ADDRESS,
  EVENT_STREAM_ADDRESS,
  CHAIN_NAME,
  MASTER_KEY_PAIR_PATH,
  DEFAULT_RUN_ENTRYPOINT_PAYMENT,
} = process.env;

const KEYS = Keys.Ed25519.loadKeyPairFromPrivateFile(
  `${MASTER_KEY_PAIR_PATH}/secret_key.pem`
);

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
  const orders: Map<string, BigNumberish> = new Map();
  const account =
    "account-hash-2642243a3ca1abc6f1b5ad3c9f53114955533ffe1a9e76055d1f987370d1d8e0";
  const amount = parseFixed("100", 9);
  investors.forEach((investor) => {
    orders.set(`account-hash-${investor.accountHash}`, investor.amount);
  });
  orders.set(account, amount);
  const deployHash = await idoContract.addOrders(
    KEYS,
    orders,
    DEFAULT_RUN_ENTRYPOINT_PAYMENT!
  );
  console.log({ deployHash });
  await getDeploy(NODE_ADDRESS!, deployHash);
};

addOrders();
