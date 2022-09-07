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

  const idoContractHash =
    "hash-f86ccc2fb216ac512ddb44c42208cb86b7f75bf260f4505246def86d014a3c11";

  await idoContract.setContractHash(idoContractHash.slice(5));
  const orders: Map<string, BigNumberish> = new Map();
  const account =
    "account-hash-f2af240a5aa234d6e295ff65b011126dc002f655b1034f869f38b7b2ba60e450";
  const amount = parseFixed("100", 9);
  orders.set(account, amount);
  const deployHash = await idoContract.addOrders(KEYS, orders, "1500000000");
  await getDeploy(NODE_ADDRESS!, deployHash);
  // let i = 0,
  //   j = 0;
  // const SIZE = 10;
  // for (i = 0; ; i += SIZE) {
  //   const orders: Map<string, BigNumberish> = new Map();
  //   for (j = 0; j < SIZE; j++) {
  //     if (j === investors.length - 1) break;
  //     orders.set(
  //       `account-hash-${investors[j].accountHash}`,
  //       investors[j].amount
  //     );
  //   }
  //   const deployHash = await idoContract.addOrders(KEYS, orders, "1500000000");
  //   console.log(`----${i / SIZE}-----`);
  //   console.log({ nth: i / SIZE, deployHash });
  //   await getDeploy(NODE_ADDRESS!, deployHash);
  //   console.log(`----------------`);
  // }
};

addOrders();
