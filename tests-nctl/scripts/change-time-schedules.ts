import { config } from "dotenv";
// config();
// config({ path: ".env.test.local" });
config({ path: ".env.production.local" });
import { CasperClient, Keys } from "casper-js-sdk";
import { BigNumberish } from "@ethersproject/bignumber";

import kunft from "./tiers/casper/kunft.json";

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

const changeTimeSchedules = async () => {
  const idoContract = new IDOClient(
    NODE_ADDRESS!,
    CHAIN_NAME!,
    EVENT_STREAM_ADDRESS!
  );
  const casperClient = new CasperClient(NODE_ADDRESS!);

  const idoContractHash = await getAccountNamedKeyValue(
    casperClient,
    KEYS.publicKey,
    `KUNFT Marketplace_ido_contract_hash`
  );

  await idoContract.setContractHash(idoContractHash.slice(5));

  const { startTime, endTime, schedules: schedulesInfo } = kunft.info;

  const schedules = new Map<number, BigNumberish>([]);
  schedulesInfo.forEach((schedule) => {
    schedules.set(schedule.time, schedule.percent * 10 ** 2);
  });

  const deployHash = await idoContract.changeTimeSchedules(
    KEYS,
    startTime,
    endTime,
    schedules,
    DEFAULT_RUN_ENTRYPOINT_PAYMENT!
  );

  console.log(`changeTimeSchedules deploy hash: ${deployHash}`);
  await getDeploy(NODE_ADDRESS!, deployHash);
  console.log("changeTimeSchedules done");
};
changeTimeSchedules();
